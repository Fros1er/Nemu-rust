use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;
use std::io;
use std::io::Write;
use std::sync::Mutex;

pub const LITEUART_MMIO_START: PAddr = PAddr::new(0xa0000200);

// const UART_REG_RXTX: usize = 0;
// const UART_REG_TXFULL: usize = 1;
const UART_REG_RXEMPTY: usize = 2;
// const UART_REG_EV_STATUS: usize = 3;
// const UART_REG_EV_PENDING: usize = 4;
// const UART_REG_EV_ENABLE: usize = 5;

pub struct LiteUART {
    mem: Mutex<[u32; 6]>,
}

impl LiteUART {
    pub fn new() -> Self {
        let mut mem = [0; 6];
        mem[UART_REG_RXEMPTY] = 1; // TODO: UART Read
        Self {
            mem: Mutex::new(mem),
        }
    }
}

impl IOMap for LiteUART {
    fn len(&self) -> usize {
        6 * 4
    }
    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        unsafe {
            let mem: &[u8; 6 * 4] = std::mem::transmute(&*self.mem.lock().unwrap());
            len.read_sized(mem.get_unchecked(offset))
        }
    }

    fn write(&mut self, offset: usize, data: u64, _len: MemOperationSize) {
        // println!("liteuart memwrite ofs {}", offset);
        match offset {
            0 => {
                // UART_REG_RXTX
                print!("{}", data as u8 as char);
                let _ = io::stdout().flush();
            }
            16 => {
                // TODO: read ack
            }
            20 => {
                // TODO: interrupt enable/disable
            }
            _ => panic!("write to LiteUART+{:#x} should not happen", offset),
        }
    }
}
