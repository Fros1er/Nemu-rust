use std::ptr::{addr_of, addr_of_mut};
use std::time::SystemTime;

use crate::device::Device;
use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize;
use crate::memory::vaddr::MemOperationSize::QWORD;

pub const TIMER_MMIO_START: PAddr = PAddr::new(0xa0000048);

pub struct Timer {
    // queue: VecDeque<i32>,
    boot_time: SystemTime,
    mem: [u8; 8],
}

impl Timer {
    pub fn new() -> Self {
        Self {
            boot_time: SystemTime::now(),
            mem: [0; 8],
        }
    }
}

impl Device for Timer {
    fn update(&mut self) {
        if let Ok(now) = SystemTime::now().duration_since(self.boot_time) {
            let us = now.as_micros() as u64;
            QWORD.write_sized(us, addr_of_mut!(self.mem[0]))
        }
    }
}

impl IOMap for Timer {
    fn data(&self) -> &[u8] {
        &self.mem
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        let res = len.read_sized(addr_of!(self.data()[offset]));
        // info!("read time: {}", res);
        res
    }

    fn write(&mut self, _offset: usize, _data: u64, _len: MemOperationSize) {
        panic!("Write to timer is not allowed")
    }
}