use std::cell::RefCell;
use std::ops::DerefMut;
use std::process::ExitCode;
use std::rc::Rc;
use clap::Parser;
use crate::device::{Device};
use crate::isa::Isa;
use crate::isa::riscv64::RISCV64;
use crate::memory::Memory;
use crate::monitor::{init_log};
use crate::monitor::sdb::sdb_loop;

mod isa;
mod monitor;
mod utils;
mod memory;
mod device;
mod cpu;
mod engine;

fn get_exit_status() -> ExitCode {
    ExitCode::from(0)
}

pub struct Emulator<T: Isa> {
    status: EmulatorStatus,
    cpu: T,
    memory: Rc<RefCell<Memory>>,
    device: Device
}

impl<T: Isa> Emulator<T> {
    pub fn new() -> Self{
        let args = crate::monitor::Args::parse();
        init_log(args.log.as_ref());


        let memory = Rc::new(RefCell::new(Memory::new())); // init mem
        let device = Device::new(); // init device
        let cpu = T::new(memory.clone());
        let img_size = monitor::load_img(args.image.as_ref(), memory.borrow_mut().deref_mut());

        Emulator {
            status: EmulatorStatus,
            cpu,
            memory,
            device
        }
    }

    pub fn run(&mut self) {
        sdb_loop(self);
    }
}



struct EmulatorStatus;

fn main() -> ExitCode {
    let mut emulator = Emulator::<RISCV64>::new();
    emulator.run();
    get_exit_status()
}
