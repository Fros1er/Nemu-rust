use crate::device::emu::clint::{CLINT, CLINT_MMIO_START};
use crate::device::emu::keyboard::{Keyboard, KEYBOARD_MMIO_START};
use crate::device::emu::plic::{PLIC, PLIC_MMIO_START};
use crate::device::emu::rtc::{RTC, RTC_MMIO_START};
use crate::device::emu::serial::{Serial, SERIAL_MMIO_START};
use crate::device::emu::timer::{Timer, TIMER_MMIO_START};
use crate::device::emu::uart16550::{UART16550, UART16550_MMIO_START};
use crate::device::emu::vga::{
    VGACtrl, SCREEN_H, SCREEN_W, VGA, VGA_CTL_MMIO_START, VGA_FRAME_BUF_MMIO_START,
};
use crate::memory::Memory;
use crate::monitor::Args;
use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

mod emu;

pub struct Devices {
    stopped: Arc<AtomicBool>,
    update_thread: JoinHandle<()>,
    pub cpu_interrupt_bits: Arc<AtomicU64>,
}

lazy_static! {
    pub static ref glob_timer: Arc<Mutex<Timer>> = Arc::new(Mutex::new(Timer::new()));
}

impl Devices {
    pub fn new(stopped: Arc<AtomicBool>, memory: &mut Memory, args: &Args) -> Self {
        let stopped_clone = stopped.clone();
        let cpu_interrupt_bits = Arc::new(AtomicU64::new(0));

        let vga = Arc::new(Mutex::new(VGA::new()));
        let vga_ctrl = Arc::new(Mutex::new(VGACtrl::new()));
        let keyboard = Arc::new(Mutex::new(Keyboard::new()));
        let serial = Arc::new(Mutex::new(Serial::new()));
        let rtc = Arc::new(Mutex::new(RTC::new()));
        let plic = Arc::new(Mutex::new(PLIC::new(cpu_interrupt_bits.clone())));
        let clint = Arc::new(Mutex::new(CLINT::new(
            cpu_interrupt_bits.clone(),
            stopped.clone(),
        )));
        let uart16550 = Arc::new(Mutex::new(UART16550::new(
            plic.clone(),
            stopped.clone(),
            args.term_timeout,
        )));

        memory.add_mmio(VGA_FRAME_BUF_MMIO_START, vga.clone());
        memory.add_mmio(VGA_CTL_MMIO_START, vga_ctrl.clone());
        memory.add_mmio(KEYBOARD_MMIO_START, keyboard.clone());
        memory.add_mmio(SERIAL_MMIO_START, serial.clone());
        memory.add_mmio(UART16550_MMIO_START, uart16550.clone());
        memory.add_mmio(TIMER_MMIO_START, glob_timer.clone());
        memory.add_mmio(RTC_MMIO_START, rtc.clone());
        memory.add_mmio(PLIC_MMIO_START, plic.clone());
        memory.add_mmio(CLINT_MMIO_START, clint.clone());

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

            let stopped = stopped.clone();
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
                            keyboard.lock().unwrap().send_key(keycode.unwrap(), true);
                        }
                        Event::KeyUp { keycode, .. } => {
                            keyboard.lock().unwrap().send_key(keycode.unwrap(), false);
                        }
                        _ => {}
                    }
                }

                keyboard.lock().unwrap().update();
                {
                    let vga_mem = &vga.lock().unwrap().mem;
                    texture
                        .update(None, vga_mem, (SCREEN_W * 4) as usize)
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
            cpu_interrupt_bits,
            stopped: stopped_clone,
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
