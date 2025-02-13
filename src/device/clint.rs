use crate::device::glob_timer;
use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;
use std::cell::RefCell;

pub const CLINT_MMIO_START: PAddr = PAddr::new(0x2000000);

const MSIP_OFFSET: usize = 0;
const MTIMECMP_OFFSET: usize = 0x4000;
const MTIME_OFFSET: usize = 0xBFF8;

pub struct CLINT {
    mtimecmp: RefCell<u64>,
}

impl CLINT {
    pub fn new() -> Self {
        Self {
            mtimecmp: RefCell::new(0),
        }
    }
}

impl IOMap for CLINT {
    fn len(&self) -> usize {
        0x10000
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of clint")
        }
        match offset {
            MSIP_OFFSET => {
                todo!("read msip");
            }
            MTIMECMP_OFFSET => {
                let ptr = &*self.mtimecmp.borrow_mut() as *const u64;
                len.read_sized(ptr as *const u8)
            }
            MTIME_OFFSET => {
                let time = glob_timer.since_boot_us();
                let ptr = &time as *const u64;
                len.read_sized(ptr as *const u8)
            }
            _ => 0,
        }
    }

    fn write(&self, offset: usize, data: u64, len: MemOperationSize) {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of clint")
        }
        match offset {
            MSIP_OFFSET => {
                if data & 1 != 0 {
                    todo!("write msip val: {}", data)
                }
            }
            MTIMECMP_OFFSET => {
                let tmp = &mut *self.mtimecmp.borrow_mut();
                let ptr = tmp as *mut u64;
                len.write_sized(data, ptr as *mut u8);
                if data != 0u64.wrapping_sub(1) {
                    todo!("write mtimecmp val: {}", data)
                }
            }
            MTIME_OFFSET => {
                todo!("write mtime val: {}", data)
            }
            _ => {}
        }
    }
}
