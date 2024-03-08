use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::EventPump;

use crate::device::keyboard::{Keyboard, KEYBOARD_MMIO_START};
use crate::device::serial::{Serial, SERIAL_MMIO_START};
use crate::device::timer::{Timer, TIMER_MMIO_START};
use crate::device::vga::{VGA, VGA_FRAME_BUF_MMIO_START, VGA_CTL_MMIO_START};
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
    timer: Rc<RefCell<Timer>>,
    devices: Vec<Rc<RefCell<dyn Device>>>,
    last_update: Instant,
}

impl Devices {
    pub fn new(memory: &mut Memory) -> Self {
        let sdl_context = sdl2::init().unwrap();

        let (vga_ctrl, vga) = VGA::new(&sdl_context);
        let vga_ctrl = Rc::new(RefCell::new(vga_ctrl));
        let keyboard = Rc::new(RefCell::new(Keyboard::new()));
        let serial = Rc::new(RefCell::new(Serial::new()));
        let timer = Rc::new(RefCell::new(Timer::new()));
        memory.add_mmio(VGA_FRAME_BUF_MMIO_START, vga.clone());
        memory.add_mmio(VGA_CTL_MMIO_START, vga_ctrl.clone());
        memory.add_mmio(KEYBOARD_MMIO_START, keyboard.clone());
        memory.add_mmio(SERIAL_MMIO_START, serial.clone());
        memory.add_mmio(TIMER_MMIO_START, timer.clone());
        Self {
            event_pump: sdl_context.event_pump().unwrap(),
            devices: vec![vga_ctrl, keyboard.clone(), serial],
            // vga,
            keyboard,
            timer,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) -> bool {
        self.timer.borrow_mut().update();
        let now = Instant::now();
        if now.duration_since(self.last_update) < Duration::from_millis(10) {
            return false;
        }
        self.last_update = now;
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
}
