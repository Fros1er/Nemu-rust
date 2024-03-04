use crate::device::Device;
use crate::isa::riscv64::RISCV64;
use crate::isa::Isa;
use crate::memory::Memory;
use crate::monitor::init_log;
use crate::monitor::sdb::sdb_loop;
use clap::Parser;
use std::cell::RefCell;
use std::ops::{DerefMut};
use std::process::ExitCode;
use std::rc::Rc;
use crate::monitor::sdb::difftest_qemu::DifftestContext;

mod device;
mod engine;
mod isa;
mod memory;
mod monitor;
mod utils;

fn get_exit_status() -> ExitCode {
    ExitCode::from(0)
}

pub struct Emulator<T: Isa> {
    cpu: T,
    memory: Rc<RefCell<Memory>>,
    _device: Device,
    difftest_ctx: Option<DifftestContext>,
}

impl<T: Isa> Emulator<T> {
    pub fn new() -> Self {
        let args = crate::monitor::Args::parse();
        init_log(args.log.as_ref());

        let memory = Rc::new(RefCell::new(Memory::new())); // init mem
        let device = Device::new(); // init device
        let mut cpu = T::new(memory.clone());
        let _img_size = monitor::load_img(args.image.as_ref(), memory.borrow_mut().deref_mut());


        let difftest_ctx = if args.difftest { Some(DifftestContext::init(cpu.isa_difftest_init(), args.image)) } else { None };

        Emulator {
            cpu,
            memory,
            _device: device,
            difftest_ctx,
        }
    }

    pub fn run(&mut self) {
        let cnt = sdb_loop(self);
        println!("Instruction executed: {}", cnt);
    }

    pub fn exit(&mut self) {
        if let Some(ctx) = &mut self.difftest_ctx {
            ctx.exit();
        }
    }
}

fn main() -> ExitCode {
    let mut emulator = Emulator::<RISCV64>::new();
    emulator.run();
    emulator.exit();
    get_exit_status()
}
