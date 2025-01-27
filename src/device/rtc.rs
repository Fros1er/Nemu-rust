use chrono::{Datelike, Timelike};

use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize;

pub const RTC_MMIO_START: PAddr = PAddr::new(0xa0000070);

pub struct RTC {}

impl RTC {
    pub fn new() -> Self {
        Self {}
    }
}

impl IOMap for RTC {
    fn len(&self) -> usize {
        32
    }
    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        let now = chrono::offset::Local::now();
        let micro =  now.timestamp_micros();
        let mem = [now.second(), now.minute(), now.hour(), now.day(), now.month(), now.year() as u32, (micro & 0xffffffff) as u32, (micro >> 32) as u32];
        unsafe {
            let mem: *const u8 = mem.get_unchecked(offset / 4) as *const u32 as *const u8;
            len.read_sized(mem)
        }
    }
}