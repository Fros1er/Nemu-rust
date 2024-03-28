use std::cell::RefCell;
use std::ptr::{addr_of, addr_of_mut};
use std::rc::Rc;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::Sdl;
use crate::device::Device;

use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize;
use crate::memory::vaddr::MemOperationSize::{DWORD, WORD};

const SCREEN_W: u32 = 400;
const SCREEN_H: u32 = 300;
pub const VGA_FRAME_BUF_MMIO_START: PAddr = PAddr::new(0xa1000000);
pub const VGA_CTL_MMIO_START: PAddr = PAddr::new(0xa0000100);


pub struct VGA {
    canvas: WindowCanvas,
    texture: Texture,
    mem: Box<[u8]>,
}

pub struct VGAControl {
    vga: Rc<RefCell<VGA>>,
    mem: [u8; 8], // width: 2, height: 2, sync: 4 (but works as bool)
}

impl VGA {
    pub fn new(sdl_context: &Sdl) -> (VGAControl, Rc<RefCell<VGA>>) {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Emulator", SCREEN_W, SCREEN_H)
            .position_centered()
            .resizable()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_static(PixelFormatEnum::ARGB8888, SCREEN_W, SCREEN_H).unwrap();
        let vga = VGA {
            canvas,
            texture,
            mem: vec![0u8; (SCREEN_W * SCREEN_H * 4) as usize].into_boxed_slice(),
        };
        let vga = Rc::new(RefCell::new(vga));
        let mut vga_ctrl = VGAControl {
            vga: vga.clone(),
            mem: [0u8; 8],
        };
        WORD.write_sized(SCREEN_W as u64, addr_of_mut!(vga_ctrl.mem[0]));
        WORD.write_sized(SCREEN_H as u64, addr_of_mut!(vga_ctrl.mem[2]));
        (vga_ctrl, vga)
    }

    fn update(&mut self) {
        self.texture.update(None, &self.mem, (SCREEN_W * 4) as usize).unwrap();
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }
}

impl Device for VGAControl {
    fn update(&mut self) {
        if DWORD.read_sized(addr_of!(self.mem[4])) != 0 {
            self.vga.borrow_mut().update();
        }
    }
}

impl IOMap for VGAControl {
    fn data_for_default_read(&self) -> &[u8] {
        &self.mem
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        if offset + len as usize > 4 {
            panic!("Read VGA control sync is not allowed")
        }
        len.read_sized(addr_of!(self.mem[offset]))
    }

    fn write(&mut self, offset: usize, data: u64, len: MemOperationSize) {
        if offset < 4 {
            panic!("Write VGA control size is not allowed")
        }
        unsafe { len.write_sized(data, self.mem.get_unchecked_mut(offset)) }
    }
}

impl IOMap for VGA {
    fn data_for_default_read(&self) -> &[u8] {
        &self.mem
    }

    fn read(&self, _offset: usize, _len: MemOperationSize) -> u64 {
        panic!("Read VGA memory is not allowed")
    }

    fn write(&mut self, offset: usize, data: u64, len: MemOperationSize) {
        unsafe { len.write_sized(data, self.mem.get_unchecked_mut(offset)) }
    }
}