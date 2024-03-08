use crate::device::Device;
use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize;

pub const SERIAL_MMIO_START: PAddr = PAddr::new(0xa00003f8);

pub struct Serial {
    // queue: VecDeque<i32>,
    mem: [u8; 1],
}

impl Serial {
    pub fn new() -> Self {
        Self {
            mem: [0; 1],
        }
    }
}

impl Device for Serial {
    fn update(&mut self) {}
}

impl IOMap for Serial {
    fn data(&self) -> &[u8] {
        &self.mem
    }

    fn read(&self, _offset: usize, _len: MemOperationSize) -> u64 {
        panic!("Read Serial memory is not allowed")
    }

    fn write(&mut self, _offset: usize, data: u64, _len: MemOperationSize) {
        // len.write_sized(data, addr_of_mut!(self.mem[offset]))
        print!("{}", data as u8 as char);
    }
}