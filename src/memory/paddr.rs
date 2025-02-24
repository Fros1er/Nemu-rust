use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::Memory;
use crate::utils::configs::{
    CONFIG_FIRMWARE_BASE, CONFIG_FIRMWARE_SIZE, CONFIG_MEM_BASE, CONFIG_MEM_SIZE,
};
use lazy_static::lazy_static;
use log::{info, warn};
use std::fmt::{Display, Formatter, LowerHex};
use std::ops::{Add, Sub};

//noinspection RsStructNaming
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PAddr(u64);

impl PAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }
    pub fn to_host_mem_arr_index(&self) -> usize {
        (self.0 - CONFIG_MEM_BASE.0) as usize
    }
    pub fn to_host_firmware_arr_index(&self) -> usize {
        (self.0 - CONFIG_FIRMWARE_BASE.0) as usize
    }
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Add<u64> for PAddr {
    type Output = PAddr;
    fn add(self, rhs: u64) -> Self::Output {
        PAddr(self.0 + rhs)
    }
}

impl Sub<u64> for PAddr {
    type Output = PAddr;
    fn sub(self, rhs: u64) -> Self::Output {
        PAddr(self.0 - rhs)
    }
}

impl Display for PAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LowerHex for PAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

pub const PMEM_LEFT: PAddr = CONFIG_MEM_BASE;
pub const FIRMWARE_LEFT: PAddr = CONFIG_FIRMWARE_BASE;
lazy_static! {
    pub static ref PMEM_RIGHT: PAddr = PMEM_LEFT + CONFIG_MEM_SIZE - 1;
    pub static ref FIRMWARE_RIGHT: PAddr = FIRMWARE_LEFT + CONFIG_FIRMWARE_SIZE - 1;
}

impl Memory {
    #[inline]
    fn get_mem_ptr(&self, paddr: &PAddr) -> Option<*const u8> {
        if Memory::in_pmem(paddr) {
            Some(unsafe { self.pmem.get_unchecked(paddr.to_host_mem_arr_index()) })
        // } else if Memory::in_firmware(paddr) {
        //     Some(unsafe { self.firmware.get_unchecked(paddr.to_host_firmware_arr_index()) })
        } else {
            None
        }
    }
    #[inline]
    fn get_mem_ptr_mut(&mut self, paddr: &PAddr) -> Option<*mut u8> {
        if Memory::in_pmem(paddr) {
            Some(unsafe { self.pmem.get_unchecked_mut(paddr.to_host_mem_arr_index()) })
        // } else if Memory::in_firmware(paddr) {
        //     Some(unsafe { self.firmware.get_unchecked_mut(paddr.to_host_firmware_arr_index()) })
        } else {
            None
        }
    }

    pub fn read_mem(&self, paddr: &PAddr, len: MemOperationSize) -> Option<u64> {
        if let Some(ptr) = self.get_mem_ptr(paddr) {
            return Some(len.read_sized(ptr));
        }
        warn!("MEM READ ERR: {:#x}", paddr.0);
        None
    }

    pub fn read(&self, paddr: &PAddr, len: MemOperationSize) -> Option<u64> {
        if let Some(ptr) = self.get_mem_ptr(paddr) {
            return Some(len.read_sized(ptr));
        }
        if let Some(iomap) = self.find_iomap(paddr) {
            return Some(
                iomap
                    .device
                    .lock()
                    .unwrap()
                    .read(iomap.paddr_to_device_mem_idx(paddr), len),
            );
        }
        warn!("MEM & IO READ ERR: {:#x}", paddr.0);
        None
    }

    pub fn write(&mut self, paddr: &PAddr, data: u64, len: MemOperationSize) -> Result<(), ()> {
        if let Some(ptr) = self.get_mem_ptr_mut(paddr) {
            len.write_sized(data, ptr);
            return Ok(());
        }

        match self.find_iomap_mut(paddr) {
            Some(iomap) => {
                iomap
                    .device
                    .lock()
                    .unwrap()
                    .write(iomap.paddr_to_device_mem_idx(paddr), data, len);
                Ok(())
            }
            None => {
                warn!("MEM WRITE ERR: {:#x}", paddr.0);
                Err(())
            }
        }
    }
    // len: n elements, not n bytes!
    // pub fn pmem_memcpy<T: Num>(&mut self, src: &[T], dst: &PAddr, len: usize) {
    //     assert!(Memory::in_pmem(dst));
    //     assert!(Memory::in_pmem(&PAddr::new(dst.value() + len as u64)));
    //     let src = src.as_ptr();
    //     let dst = addr_of_mut!(self.pmem[dst.to_host_arr_index()]) as *mut T;
    //     unsafe {
    //         std::ptr::copy_nonoverlapping(src, dst, len);
    //     }
    // }
}

pub fn init_mem() {
    info!(
        "physical memory area [{:#x}, {:#x}]",
        PMEM_LEFT.0, PMEM_RIGHT.0
    )
}
