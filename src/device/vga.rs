use std::cell::RefCell;
use std::ptr::{addr_of, addr_of_mut};
use std::sync::Mutex;

use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::isa::riscv64::vaddr::MemOperationSize::WORD;

pub const SCREEN_W: u32 = 320;
pub const SCREEN_H: u32 = 200;
pub const VGA_FRAME_BUF_MMIO_START: PAddr = PAddr::new(0xa1000000);
pub const VGA_CTL_MMIO_START: PAddr = PAddr::new(0xa0000100);

pub struct VGA {
    pub(super) mem: Mutex<Box<[u8]>>,
}

pub struct VGACtrl {
    mem: RefCell<[u8; 8]>, // width: 2, height: 2, sync: 4 (but works as bool)
}

impl VGA {
    pub fn new() -> Self {
        Self {
            mem: Mutex::new(vec![0u8; (SCREEN_W * SCREEN_H * 4) as usize].into_boxed_slice())
        }
    }
}

impl VGACtrl {
    pub fn new() -> Self {
        let vga_ctrl = VGACtrl {
            mem: RefCell::new([0u8; 8]),
        };
        WORD.write_sized(SCREEN_W as u64, addr_of_mut!(vga_ctrl.mem.borrow_mut()[0]));
        WORD.write_sized(SCREEN_H as u64, addr_of_mut!(vga_ctrl.mem.borrow_mut()[2]));
        vga_ctrl
    }
}

impl IOMap for VGACtrl {
    fn len(&self) -> usize {
        8
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        if offset + len as usize > 4 {
            panic!("Read VGA control sync is not allowed")
        }
        len.read_sized(addr_of!(self.mem.borrow()[offset]))
    }

    fn write(&self, offset: usize, data: u64, len: MemOperationSize) {
        if offset < 4 {
            panic!("Write VGA control size is not allowed")
        }
        unsafe { len.write_sized(data, self.mem.borrow_mut().get_unchecked_mut(offset)) }
    }
}

impl IOMap for VGA {
    fn len(&self) -> usize {
        (SCREEN_W * SCREEN_H * 4) as usize
    }

    fn write(&self, offset: usize, data: u64, len: MemOperationSize) {
        unsafe { len.write_sized(data, self.mem.lock().unwrap().get_unchecked_mut(offset)) }
    }
}