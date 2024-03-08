use std::cell::RefCell;
use std::rc::Rc;

use sdl2::{EventPump, Sdl};
use sdl2::event::Event;
use crate::device::keyboard::{Keyboard, KEYBOARD_MMIO_START};

use crate::device::vga::{VGA, VGA_MMIO_START};
use crate::memory::{IOMap, Memory};

mod keyboard;
mod vga;

#[macro_export]
macro_rules! device_impl_iomap {
    ($device_name:ident) => {
        impl IOMap for $device_name {
            fn data(&self) -> &[u8] {
                &self.mem
            }

            fn data_mut(&mut self) -> &mut [u8] {
                &mut self.mem
            }
        }
    };
}

trait Device {
    fn update(&mut self, dirty: bool);
}

pub struct Devices {
    // sdl_context: Sdl,
    event_pump: EventPump,
    vga: Rc<RefCell<VGA>>,
    keyboard: Rc<RefCell<Keyboard>>,
    devices: Vec<Rc<RefCell<dyn Device>>>
}

impl Devices {
    pub fn new(memory: &mut Memory) -> Self {
        let sdl_context = sdl2::init().unwrap();

        let mut vga = Rc::new(RefCell::new(VGA::new(&sdl_context)));
        let mut keyboard = Rc::new(RefCell::new(Keyboard::new()));
        memory.add_mmio(VGA_MMIO_START, vga.clone());
        memory.add_mmio(KEYBOARD_MMIO_START, keyboard.clone());
        Self {
            // sdl_context,
            event_pump: sdl_context.event_pump().unwrap(),
            devices: vec![vga.clone(), keyboard.clone()],
            vga,
            keyboard,
        }
    }

    pub fn update(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return true;
                }
                Event::KeyDown { keycode, .. } => {
                    self.keyboard.borrow_mut().send_key(keycode.unwrap(), true);
                }
                Event::KeyUp { keycode, keymod, repeat, .. } => {
                    self.keyboard.borrow_mut().send_key(keycode.unwrap(), false);
                }
                _ => {}
            }
        }
        false
    }
}

//
// pub fn main() -> Result<(), String> {
//
//
//
//
//
//
//     let mut event_pump = sdl_context.event_pump()?;
//
//     println!("This example simply prints all events SDL knows about.");
//
//     'running: loop {
//         for event in event_pump.poll_iter() {
//             match event {
//                 Event::Quit { .. }
//                 | Event::KeyDown {
//                     keycode: Some(Keycode::Escape),
//                     ..
//                 } => break 'running,
//                 // skip mouse motion intentionally because of the verbose it might cause.
//                 Event::MouseMotion { .. } => {}
//                 e => {
//                     println!("{:?}", e);
//                 }
//             }
//         }
//
//         canvas.clear();
//         canvas.present();
//         ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
//         // The rest of the game loop goes here...
//     }
//     Ok(())
// }