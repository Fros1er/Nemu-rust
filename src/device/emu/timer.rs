use std::ptr::addr_of;
use std::time::SystemTime;
// use std::sync::Arc;
// use std::sync::atomic::{AtomicBool, AtomicU64};
// use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
// use std::thread;
// use std::time::{Duration, SystemTime};

use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;

pub const TIMER_MMIO_START: PAddr = PAddr::new(0xa0000048);

pub struct Timer {
    boot_time: SystemTime,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            boot_time: SystemTime::now(),
        }
    }

    pub fn since_boot_us(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.boot_time)
            .unwrap()
            .as_micros() as u64
    }
}

impl IOMap for Timer {
    fn len(&self) -> usize {
        8
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        let time = self.since_boot_us();
        let res = len.read_sized(unsafe { (addr_of!(time) as *const u8).offset(offset as isize) });
        res
    }
}
