use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::Sdl;
use crate::device::Device;
use crate::device_impl_iomap;

use crate::memory::IOMap;
use crate::memory::paddr::PAddr;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;
pub const VGA_MMIO_START: PAddr = PAddr::new(0xa0000100);

pub struct VGA {
    canvas: WindowCanvas,
    texture: Texture,
    mem: [u8; (SCREEN_W * SCREEN_H * 4) as usize],
}

impl VGA {
    pub fn new(sdl_context: &Sdl) -> VGA {
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
        VGA {
            canvas,
            texture,
            mem: [0u8; (SCREEN_W * SCREEN_H * 4) as usize],
        }
    }
}

impl Device for VGA {
    fn update(&mut self, dirty: bool) {
        if dirty {
            self.texture.update(None, &self.mem, (SCREEN_W * 4) as usize).unwrap();
            self.canvas.clear();
            self.canvas.copy(&self.texture, None, None).unwrap();
            self.canvas.present();
        }
    }
}

device_impl_iomap!(VGA);