use std::ops::{Add, Sub};
use std::ptr::{addr_of, addr_of_mut};
use lazy_static::lazy_static;
use log::info;
use num::Num;
use crate::memory::Memory;
use crate::memory::vaddr::VAddr;
use crate::utils::configs::{CONFIG_MBASE, CONFIG_MSIZE};

//noinspection RsStructNaming
pub struct PAddr(u64);

impl PAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }
    pub fn to_host_arr_index(&self) -> u64 {
        self.0 - CONFIG_MBASE.0
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

pub struct PAddrDiff(u64);

impl PAddrDiff {
    pub const fn new(addr: u64) -> PAddrDiff {
        PAddrDiff(addr)
    }
}

impl Add<PAddrDiff> for PAddr {
    type Output = PAddr;
    fn add(self, rhs: PAddrDiff) -> Self::Output {
        PAddr(self.0 + rhs.0)
    }
}

impl Sub<PAddrDiff> for PAddr {
    type Output = PAddr;
    fn sub(self, rhs: PAddrDiff) -> Self::Output {
        PAddr(self.0 - rhs.0)
    }
}

pub const PMEM_LEFT: PAddr = CONFIG_MBASE;
lazy_static! {
    pub static ref PMEM_RIGHT: PAddr = PMEM_LEFT + PAddrDiff::new(CONFIG_MSIZE) - PAddrDiff::new(1);
}

impl Memory {
    fn get_ptr<T: Num>(&self, paddr: &PAddr) -> *const T {
        addr_of!(self.pmem[paddr.to_host_arr_index() as usize]) as *const T
    }
    fn get_ptr_mut<T: Num>(&mut self, paddr: &PAddr) -> *mut T {
        addr_of_mut!(self.pmem[paddr.to_host_arr_index() as usize]) as *mut T
    }
    pub fn read_p<T: Num>(&self, paddr: &PAddr) -> T {
        unsafe {
            self.get_ptr::<T>(paddr).read()
        }
    }

    pub fn write_p<T: Num>(&mut self, paddr: &PAddr, num: T) {
        unsafe {
            self.get_ptr_mut::<T>(paddr).write(num)
        }
    }
    /// len: n elements, not n bytes!
    pub fn memcpy_p<T: Num>(&mut self, src: &[T], dst: &PAddr, len: usize) {
        let src = src.as_ptr();
        let dst = self.get_ptr_mut::<T>(dst);
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, len);
        }
    }
}

pub fn init_mem() {
    info!("physical memory area [{:#x}, {:#x}]", PMEM_LEFT.0, PMEM_RIGHT.0)
}