use std::sync::{Arc, Mutex};

use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::{init_mem, PAddr, PMEM_LEFT, PMEM_RIGHT};
use crate::utils::configs::CONFIG_MEM_SIZE;

pub mod paddr;

pub trait IOMap {
    fn len(&self) -> usize;

    /// guarantee ofs is inside self.mem
    fn read(&self, _offset: usize, _len: MemOperationSize) -> u64 {
        panic!("Read should not happen");
    }

    /// guarantee ofs is inside self.mem
    ///
    /// Use interior mutability pattern for flexibility
    fn write(&mut self, _offset: usize, _data: u64, _len: MemOperationSize) {
        panic!("Write should not happen");
    }
}

pub struct IOMapEntry {
    left: PAddr,
    right: PAddr,
    device: Arc<Mutex<dyn IOMap>>,
}

impl IOMapEntry {
    fn new(left: PAddr, right: PAddr, device: Arc<Mutex<dyn IOMap>>) -> Self {
        Self {
            left,
            right,
            device,
        }
    }
    fn addr_inside(&self, paddr: &PAddr) -> bool {
        self.left <= *paddr && *paddr < self.right
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
            pmem: vec![0u8; CONFIG_MEM_SIZE].into_boxed_slice(),
            mmio: vec![],
        }
    }

    pub fn in_pmem(paddr: &PAddr) -> bool {
        PMEM_LEFT <= *paddr && *paddr <= *PMEM_RIGHT
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

    pub fn add_mmio(&mut self, left: PAddr, device: Arc<Mutex<dyn IOMap>>) {
        let right = left.clone() + device.lock().unwrap().len() as u64;
        let io_map = IOMapEntry::new(left, right, device);
        if Self::in_pmem(&io_map.left) && Self::in_pmem(&io_map.right) {
            panic!(
                "MMIO region ({:#x}, {:#x}) overlaps with pmem",
                io_map.left, io_map.right
            )
        }
        if let Some(mmap) = self
            .find_iomap(&io_map.left)
            .or(self.find_iomap(&(io_map.right.clone() - 1)))
        {
            panic!(
                "MMIO region ({:#x}, {:#x}) overlaps with other mmio region ({:#x} {:#x})",
                io_map.left, io_map.right, mmap.left, mmap.right
            )
        }
        self.mmio.push(io_map)
    }
}
