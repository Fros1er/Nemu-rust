use std::ptr::addr_of_mut;
use sdl2::keyboard::Keycode;
use crate::device::Device;
use crate::device_impl_iomap;
use crate::memory::IOMap;
use crate::memory::paddr::PAddr;

pub const KEYBOARD_MMIO_START: PAddr = PAddr::new(0xa0000060);

pub struct Keyboard {
    // queue: VecDeque<i32>,
    mem: [u8; 4],
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            // queue: VecDeque::new(),
            mem: [0; 4],
        }
    }

    pub fn send_key(&mut self, keycode: Keycode, is_down: bool) {
        let mut keycode = keycode as i32;
        if is_down {
            keycode |= 0x8000;
        }
        unsafe {
            (addr_of_mut!(self.mem[0]) as *mut i32).write(keycode);
        }
    }
}

impl Device for Keyboard {
    fn update(&mut self, dirty: bool) {
        if dirty {
            panic!("Can't write to keyboard mem")
        }
    }
}

device_impl_iomap!(Keyboard);