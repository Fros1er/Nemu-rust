use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize;

pub const SERIAL_MMIO_START: PAddr = PAddr::new(0xa00003f8);

pub struct Serial {
}

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
    }
}