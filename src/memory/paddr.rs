use std::io::Read;
use std::ops::{Add, Sub};
use std::ptr::{addr_of, addr_of_mut};
use lazy_static::lazy_static;
use log::info;
use num::Num;
use crate::memory::vaddr::VAddr;
use crate::utils::configs::{CONFIG_MBASE, CONFIG_MSIZE, CONFIG_PC_RESET_OFFSET};

//noinspection RsStructNaming
pub struct PAddr(usize);

impl PAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }
    pub fn to_host_arr_index(&self) -> usize {
        self.0 - CONFIG_MBASE.0
    }
    pub unsafe fn get_ptr<T: Num>(&self) -> *const T {
        addr_of!(PMEM[self.to_host_arr_index()]) as *const T
    }

    pub unsafe fn get_ptr_mut<T: Num>(&self) -> *mut T {
        addr_of_mut!(PMEM[self.to_host_arr_index()]) as *mut T
    }

    pub fn read<T: Num>(&self) -> T {
        unsafe {
            let ptr: *const T = self.get_ptr();
            ptr.read()
        }
    }

    pub fn write<T: Num>(&self, num: T) {
        unsafe {
            let ptr = addr_of_mut!(PMEM[self.0]);
            let ptr = ptr as *mut T;
            ptr.write(num);
        }
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

impl From<&VAddr> for PAddr {
    fn from(value: &VAddr) -> Self {
        Self(value.value())
    }
}

pub struct PAddrDiff(usize);

impl PAddrDiff {
    pub const fn new(addr: usize) -> PAddrDiff {
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
    pub static ref PMEM_RIGHT: PAddr = PMEM_LEFT + CONFIG_MSIZE - PAddrDiff::new(1);
}

pub static mut PMEM: [u8; CONFIG_MSIZE.0] = [0u8; CONFIG_MSIZE.0];

/// len: n elements, not n bytes!
pub fn memcpy_to_pmem<T: Num>(src: &[T], dst: &PAddr, len: usize) {
    PAddrDiff::new(10);
    unsafe {
        let src = src.as_ptr();
        let dst = dst.get_ptr_mut::<T>();
        std::ptr::copy_nonoverlapping(src, dst, len);
    }
}

pub fn init_mem() {
    info!("physical memory area [{:#x}, {:#x}]", PMEM_LEFT.0, PMEM_RIGHT.0)
}