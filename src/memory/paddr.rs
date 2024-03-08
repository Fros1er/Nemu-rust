use std::fmt::{Display, Formatter, LowerHex};
use std::ops::{Add, Sub};
use std::ptr::{addr_of, addr_of_mut};
use lazy_static::lazy_static;
use log::info;
use num::Num;
use crate::memory::Memory;
use crate::memory::vaddr::{MemOperationSize, VAddr};
use crate::utils::configs::{CONFIG_MBASE, CONFIG_MSIZE};

//noinspection RsStructNaming
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PAddr(u64);

impl PAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }
    pub fn to_host_arr_index(&self) -> usize {
        (self.0 - CONFIG_MBASE.0) as usize
    }
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<&VAddr> for PAddr {
    fn from(value: &VAddr) -> Self {
        Self(value.value())
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

pub const PMEM_LEFT: PAddr = CONFIG_MBASE;
lazy_static! {
    pub static ref PMEM_RIGHT: PAddr = PMEM_LEFT + CONFIG_MSIZE - 1;
}

impl Memory {
    pub fn read_p(&self, paddr: &PAddr, len: MemOperationSize) -> u64 {
        match self.find_iomap(paddr) {
            Some(iomap) => {
                iomap.device.borrow().read(iomap.paddr_to_device_mem_idx(paddr), len)
            }
            None => len.read_sized(addr_of!(self.pmem[paddr.to_host_arr_index()]))
        }
    }

    pub fn write_p(&mut self, paddr: &PAddr, data: u64, len: MemOperationSize) {
        match self.find_iomap_mut(paddr) {
            Some(iomap) => {
                iomap.device.borrow_mut().write(iomap.paddr_to_device_mem_idx(paddr), data, len);
            }
            None => len.write_sized(data, addr_of_mut!(self.pmem[paddr.to_host_arr_index()]))
        };
    }
    /// len: n elements, not n bytes!
    pub fn pmem_memcpy<T: Num>(&mut self, src: &[T], dst: &PAddr, len: usize) {
        assert!(Memory::in_pmem(dst));
        assert!(Memory::in_pmem(&PAddr::new(dst.value() + len as u64)));
        let src = src.as_ptr();
        let dst = addr_of_mut!(self.pmem[dst.to_host_arr_index()]) as *mut T;
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, len);
        }
    }
}

pub fn init_mem() {
    info!("physical memory area [{:#x}, {:#x}]", PMEM_LEFT.0, PMEM_RIGHT.0)
}