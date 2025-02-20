use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;
use log::info;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub const PLIC_MMIO_START: PAddr = PAddr::new(0xc000000);

pub struct PLIC {
    interrupt_bits: Arc<AtomicU64>,
}

impl PLIC {
    pub fn new() -> Self {
        Self {
            interrupt_bits: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn register_interrupt(&mut self, interrupt_bits: Arc<AtomicU64>) {}
}

impl IOMap for PLIC {
    fn len(&self) -> usize {
        0x4000000
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of PLIC")
        }
        if offset < 0x1000 {
            info!("Read PLIC priority, offset {:#x} ", offset);
            // priority, not supported
            return 0;
        } else if offset == 0x1000 {
            info!("Read PLIC pending bit");
            return 0;
        }
        info!("Read PLIC offset {:#x}", offset);
        0
    }

    fn write(&mut self, offset: usize, data: u64, len: MemOperationSize) {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of PLIC")
        }
        if offset < 0x1000 {
            info!("Write PLIC priority, offset {:#x} ", offset);
            // priority, not supported
            return;
        } else if offset == 0x1000 {
            info!("Write PLIC pending bit");
            return;
        }
        info!("Write PLIC offset {:#x} data {:#x}", offset, data);
    }
}
