use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use sdl2::keyboard::Keycode;
use strum_macros::IntoStaticStr;

use crate::memory::IOMap;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize;

pub const KEYBOARD_MMIO_START: PAddr = PAddr::new(0xa0000060);

macro_rules! build_keymap {
    ($($body:ident),*) => {
        #[repr(u32)]
        #[derive(Clone, IntoStaticStr, PartialEq)]
        enum NemuKeycode {
            None,
            $($body,)*
        }
        fn build_keymap() -> HashMap<Keycode, NemuKeycode> {
            let mut map = HashMap::new();
            $(
               map.insert(Keycode::$body, NemuKeycode::$body);
            )*
            map
        }
    };
}

build_keymap! {
    Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Backquote, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0, Minus, Equals, Backspace,
    Tab, Q, W, E, R, T, Y, U, I, O, P, LeftBracket, RightBracket, Backslash,
    CapsLock, A, S, D, F, G, H, J, K, L, Semicolon, Quote, Return,
    LShift, Z, X, C, V, B, N, M, Comma, Period, Slash, RShift,
    LCtrl, Application, LAlt, Space, RAlt, RCtrl,
    Up, Down, Left, Right, Insert, Delete, Home, End, PageUp, PageDown
}

pub struct Keyboard {
    keycode_map: HashMap<Keycode, NemuKeycode>,
    key_queue: Mutex<VecDeque<u32>>,
    mem: Mutex<[u8; 8]>,
}

impl Keyboard {
    pub fn new() -> Self {
        let keycode_map = build_keymap();
        Self {
            keycode_map,
            key_queue: Mutex::new(VecDeque::new()),
            mem: Mutex::new([0; 8]), // mem[3:0]: keycode, mem[7:4]: write anything to set keycode to none
        }
    }

    fn write_key(&self, keycode: u32) {
        unsafe {
            let addr = self.mem.lock().unwrap().get_unchecked_mut(0) as *mut u8;
            (addr as *mut u32).write(keycode);
        }
    }

    pub fn send_key(&self, keycode: Keycode, is_down: bool) {
        if let Some(keycode) = self.keycode_map.get(&keycode) {
            if keycode == &NemuKeycode::P {
                if !is_down {
                    return;
                }
                let mut key_queue = self.key_queue.lock().unwrap();
                let v = vec![
                    NemuKeycode::S as u32 | 0x8000,
                    NemuKeycode::A as u32 | 0x8000,
                    NemuKeycode::S as u32,
                    NemuKeycode::J as u32 | 0x8000,
                    NemuKeycode::A as u32,
                    NemuKeycode::J as u32,
                ];
                if key_queue.is_empty() {
                    self.write_key(v[0]);
                }
                for i in v {
                    key_queue.push_back(i);
                    println!("Macro: {}", i);
                }
                return
            }
            let s: &'static str = keycode.into();
            println!("{} {}", s, is_down);
            let mut keycode = keycode.clone() as u32;
            if is_down {
                keycode |= 0x8000;
            }
            let mut key_queue = self.key_queue.lock().unwrap();
            if key_queue.is_empty() {
                self.write_key(keycode)
            }
            key_queue.push_back(keycode)
        };
    }
}

impl IOMap for Keyboard {
    fn len(&self) -> usize {
        8
    }
    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        unsafe { len.read_sized(self.mem.lock().unwrap().get_unchecked(offset)) }
    }

    fn write(&self, offset: usize, data: u64, _size: MemOperationSize) {
        if offset < 4 {
            panic!("Write to keyboard keycode is not allowed")
        }
        if data != 0 {
            let mut key_queue = self.key_queue.lock().unwrap();
            if !key_queue.is_empty() {
                key_queue.pop_front();
            }
            let next = key_queue.front().unwrap_or(&(NemuKeycode::None as u32));
            self.write_key(next.clone());
        }
    }
}