use std::cell::RefCell;
use std::ptr::addr_of;
use std::rc::Rc;

use crate::memory::paddr::{init_mem, PAddr, PMEM_LEFT, PMEM_RIGHT};
use crate::memory::vaddr::MemOperationSize;
use crate::utils::configs::CONFIG_MSIZE;

pub mod paddr;
pub mod vaddr;

pub trait IOMap {
    fn data_for_default_read(&self) -> &[u8];
    fn len(&self) -> usize {
        self.data_for_default_read().len()
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        len.read_sized(addr_of!(self.data_for_default_read()[offset]))
    }

    // guarantee ofs is inside self.mem
    fn write(&mut self, offset: usize, data: u64, len: MemOperationSize);

    // fn data_mut(&mut self) -> &mut [u8];
}

pub struct IOMapEntry {
    left: PAddr,
    right: PAddr,
    device: Rc<RefCell<dyn IOMap>>,
}

impl IOMapEntry {
    fn new(left: PAddr, right: PAddr, device: Rc<RefCell<dyn IOMap>>) -> Self {
        Self { left, right, device }
    }
    fn addr_inside(&self, paddr: &PAddr) -> bool {
        self.left <= *paddr && *paddr <= self.right
    }

    fn paddr_to_device_mem_idx(&self, paddr: &PAddr) -> usize {
        assert!(self.addr_inside(paddr));
        (paddr.value() - self.left.value()) as usize
    }
}

pub struct Memory {
    pub pmem: Box<[u8]>,
    mmio: Vec<IOMapEntry>,
}

impl Memory {
    pub fn new() -> Self {
        init_mem();
        Self {
            pmem: vec![0u8; CONFIG_MSIZE as usize].into_boxed_slice(),
            mmio: vec!(),
        }
    }

    fn in_pmem(paddr: &PAddr) -> bool {
        PMEM_LEFT <= *paddr && PMEM_RIGHT.value() >= paddr.value()
    }

    pub fn find_iomap(&self, paddr: &PAddr) -> Option<&IOMapEntry> {
        for iomap in self.mmio.iter() {
            if iomap.addr_inside(paddr) {
                return Some(iomap);
            }
        }
        None
    }

    pub fn find_iomap_mut(&mut self, paddr: &PAddr) -> Option<&mut IOMapEntry> {
        for iomap in self.mmio.iter_mut() {
            if iomap.addr_inside(paddr) {
                return Some(iomap);
            }
        }
        None
    }

    pub fn add_mmio(&mut self, left: PAddr, device: Rc<RefCell<dyn IOMap>>) {
        let right = left.clone() + device.borrow().len() as u64;
        let io_map = IOMapEntry::new(left, right, device);
        if Self::in_pmem(&io_map.left) && Self::in_pmem(&io_map.right) {
            panic!("MMIO region ({:#x}, {:#x}) overlaps with pmem", io_map.left, io_map.right)
        }
        if let Some(mmap) = self.find_iomap(&io_map.left).or(self.find_iomap(&io_map.left)) {
            panic!("MMIO region ({:#x}, {:#x}) overlaps with other mmio region ({:#x} {:#x})",
                   io_map.left, io_map.right, mmap.left, mmap.right)
        }
        self.mmio.push(io_map)
    }
}
