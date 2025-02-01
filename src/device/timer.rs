use std::ptr::addr_of;
use std::time::SystemTime;
// use std::sync::Arc;
// use std::sync::atomic::{AtomicBool, AtomicU64};
// use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
// use std::thread;
// use std::time::{Duration, SystemTime};

use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize;

pub const TIMER_MMIO_START: PAddr = PAddr::new(0xa0000048);

pub struct Timer {
    // mem: Arc<AtomicU64>,
    boot_time: SystemTime,
}

impl Timer {
    // pub fn new(stopped: Arc<AtomicBool>) -> Self {
    pub fn new() -> Self {
        // let mem = Arc::new(AtomicU64::new(0));
        // let mem_t = mem.clone();
        //
        // thread::spawn(move || {
        //     let boot_time = SystemTime::now();
        //     while !stopped.load(Relaxed) {
        //         if let Ok(now) = SystemTime::now().duration_since(boot_time) {
        //             let us = now.as_micros() as u64;
        //             mem_t.store(us, Release);
        //         }
        //         thread::sleep(Duration::from_micros(1));
        //     }
        // });

        Self {
            boot_time: SystemTime::now(),
            // mem,
            // update_thread
        }
    }
}

impl IOMap for Timer {
    fn len(&self) -> usize {
        8
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        // let time = self.mem.load(Acquire);
        let time = SystemTime::now().duration_since(self.boot_time).unwrap().as_micros() as u64;
        let res = len.read_sized(unsafe { (addr_of!(time) as *const u8).offset(offset as isize) });
        res
    }
}