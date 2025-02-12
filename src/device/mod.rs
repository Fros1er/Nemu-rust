use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::device::keyboard::{Keyboard, KEYBOARD_MMIO_START};
use crate::device::liteuart::{LiteUART, LITEUART_MMIO_START};
use crate::device::plic::{PLIC, PLIC_MMIO_START};
use crate::device::rtc::{RTC, RTC_MMIO_START};
use crate::device::serial::{Serial, SERIAL_MMIO_START};
use crate::device::timer::{Timer, TIMER_MMIO_START};
use crate::device::vga::{
    VGACtrl, SCREEN_H, SCREEN_W, VGA, VGA_CTL_MMIO_START, VGA_FRAME_BUF_MMIO_START,
};
use crate::memory::Memory;

mod keyboard;
mod liteuart;
mod plic;
mod rtc;
mod serial;
mod timer;
mod vga;

pub struct Devices {
    stopped: Arc<AtomicBool>,
    update_thread: JoinHandle<()>,
}

lazy_static! {
    pub static ref glob_timer: Arc<Timer> = Arc::new(Timer::new());
}

impl Devices {
    pub fn new(memory: &mut Memory, _no_sdl: bool) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_tmp = stopped.clone();

        let vga = Arc::new(VGA::new());
        let vga_ctrl = Arc::new(VGACtrl::new());
        let keyboard = Arc::new(Keyboard::new());
        let serial = Arc::new(Serial::new());
        let liteuart = Arc::new(LiteUART::new());
        // let timer = Arc::new(Timer::new(stopped.clone()));
        // let timer = Arc::new(Timer::new());
        let rtc = Arc::new(RTC::new());
        let plic = Arc::new(PLIC::new());

        memory.add_mmio(VGA_FRAME_BUF_MMIO_START, vga.clone());
        memory.add_mmio(VGA_CTL_MMIO_START, vga_ctrl.clone());
        memory.add_mmio(KEYBOARD_MMIO_START, keyboard.clone());
        memory.add_mmio(SERIAL_MMIO_START, serial.clone());
        memory.add_mmio(LITEUART_MMIO_START, liteuart.clone());
        // memory.add_mmio(SERIAL_MMIO_START_QEMU, serial.clone());
        // memory.add_mmio(TIMER_MMIO_START, timer.clone());
        memory.add_mmio(TIMER_MMIO_START, glob_timer.clone());
        memory.add_mmio(RTC_MMIO_START, rtc.clone());
        memory.add_mmio(PLIC_MMIO_START, plic.clone());

        // let rvtest_serial = Arc::new(RVTestSerial::new());
        // memory.add_mmio(SERIAL_MMIO_START_RVTEST, rvtest_serial.clone());

        let update_thread = thread::spawn(move || {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();

            let window = video_subsystem
                .window("Emulator", SCREEN_W, SCREEN_H)
                .position_centered()
                .resizable()
                .build()
                .unwrap();
            let mut canvas = window.into_canvas().build().unwrap();
            let texture_creator = canvas.texture_creator();
            let mut texture = texture_creator
                .create_texture_static(PixelFormatEnum::ARGB8888, SCREEN_W, SCREEN_H)
                .unwrap();
            let mut event_pump = sdl_context.event_pump().unwrap();

            let stopped = stopped_tmp;
            'outer: while !stopped.load(Relaxed) {
                for event in event_pump.poll_iter() {
                    // println!("{}", event.type_id());
                    match event {
                        Event::Quit { .. } => {
                            break 'outer;
                        }
                        // Event::Window { win_event, .. } => {
                        //     if win_event == Close {
                        //         break 'outer;
                        //     }
                        // }
                        Event::KeyDown { keycode, .. } => {
                            keyboard.send_key(keycode.unwrap(), true);
                        }
                        Event::KeyUp { keycode, .. } => {
                            keyboard.send_key(keycode.unwrap(), false);
                        }
                        _ => {}
                    }
                }

                keyboard.update();
                {
                    let vga_mem = vga.mem.lock().unwrap();
                    texture
                        .update(None, vga_mem.deref(), (SCREEN_W * 4) as usize)
                        .unwrap();
                }
                canvas.clear();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
                thread::sleep(Duration::from_millis(10));
            }
            stopped.store(true, Relaxed);
        });

        Self {
            // vga,
            // keyboard,
            // timer,
            stopped,
            update_thread,
        }
    }

    pub fn stop(self) {
        self.stopped.store(true, Relaxed);
        self.update_thread.join().unwrap();
    }

    pub fn has_stopped(&self) -> bool {
        self.stopped.load(Relaxed)
    }
}
