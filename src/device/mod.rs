use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::EventPump;

use crate::device::keyboard::{Keyboard, KEYBOARD_MMIO_START};
use crate::device::serial::{Serial, SERIAL_MMIO_START};
use crate::device::timer::{Timer, TIMER_MMIO_START};
use crate::device::vga::{VGA, VGA_CTL_MMIO_START, VGA_FRAME_BUF_MMIO_START};
use crate::memory::Memory;

mod keyboard;
mod vga;
mod serial;
mod timer;

trait Device {
    fn update(&mut self);
}

pub struct Devices {
    event_pump: EventPump,
    // vga: Rc<RefCell<VGA>>,
    keyboard: Rc<RefCell<Keyboard>>,
    // timer: Rc<RefCell<Timer>>,
    devices: Vec<Rc<RefCell<dyn Device>>>,

    device_need_update: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    update_delay_thread: JoinHandle<()>,
}

impl Devices {
    pub fn new(memory: &mut Memory) -> Self {
        let sdl_context = sdl2::init().unwrap();

        let device_need_update = Arc::new(AtomicBool::new(false));
        let device_need_update_t = device_need_update.clone();
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_t = stopped.clone();

        let (vga_ctrl, vga) = VGA::new(&sdl_context);
        let vga_ctrl = Rc::new(RefCell::new(vga_ctrl));
        let keyboard = Rc::new(RefCell::new(Keyboard::new()));
        let serial = Rc::new(RefCell::new(Serial::new()));
        let timer = Rc::new(RefCell::new(Timer::new(stopped.clone())));
        memory.add_mmio(VGA_FRAME_BUF_MMIO_START, vga.clone());
        memory.add_mmio(VGA_CTL_MMIO_START, vga_ctrl.clone());
        memory.add_mmio(KEYBOARD_MMIO_START, keyboard.clone());
        memory.add_mmio(SERIAL_MMIO_START, serial.clone());
        memory.add_mmio(TIMER_MMIO_START, timer.clone());

        let update_delay_thread = thread::spawn(move || {
            while !stopped_t.load(Relaxed) {
                device_need_update_t.store(true, Release);
                thread::sleep(Duration::from_millis(10));
            }
        });

        Self {
            event_pump: sdl_context.event_pump().unwrap(),
            devices: vec![vga_ctrl, keyboard.clone(), serial],
            // vga,
            keyboard,
            // timer,
            device_need_update,
            stopped,
            update_delay_thread,
        }
    }

    pub fn update(&mut self) -> bool {
        if !self.device_need_update.load(Acquire) {
            return false;
        }
        self.device_need_update.store(false, Release);
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return true;
                }
                Event::KeyDown { keycode, .. } => {
                    self.keyboard.borrow_mut().send_key(keycode.unwrap(), true);
                }
                Event::KeyUp { keycode, .. } => {
                    self.keyboard.borrow_mut().send_key(keycode.unwrap(), false);
                }
                _ => {}
            }
        }
        for device in &self.devices {
            device.borrow_mut().update()
        }

        false
    }

    pub fn stop(self) {
        self.stopped.store(true, Relaxed);
        self.update_delay_thread.join().unwrap();
    }
}
