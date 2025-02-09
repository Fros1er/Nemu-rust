use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;
use std::io;
use std::io::Write;

pub const SERIAL_MMIO_START: PAddr = PAddr::new(0xa00003f8);
// pub const SERIAL_MMIO_START_QEMU: PAddr = PAddr::new(0x10000000);

// pub const SERIAL_MMIO_START_RVTEST: PAddr = PAddr::new(0x80001000);

// pub struct RVTestSerial {}
// impl RVTestSerial { pub fn new() -> Self { Self {} } }

// impl IOMap for RVTestSerial {
//     fn len(&self) -> usize {
//         8
//     }
//
//     fn write(&self, _offset: usize, data: u64, _len: MemOperationSize) {
//         if _offset == 0 {
//             print!("{}", (data as u8 + '0' as u8) as char);
//             let _ = io::stdout().flush();
//         }
//     }
// }

pub struct Serial {}

impl Serial {
    pub fn new() -> Self {
        Self {}
    }
}

impl IOMap for Serial {
    fn len(&self) -> usize {
        1
    }

    fn write(&self, _offset: usize, data: u64, _len: MemOperationSize) {
        print!("{}", data as u8 as char);
        let _ = io::stdout().flush();
    }
}
