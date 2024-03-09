use crate::device::Devices;
use crate::isa::riscv64::RISCV64;
use crate::isa::Isa;
use crate::memory::Memory;
use crate::monitor::init_log;
use crate::monitor::sdb::{exec_once, sdb_loop};
use clap::Parser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{DerefMut};
use std::process::ExitCode;
use std::rc::Rc;
use log::info;
use crate::monitor::sdb::difftest_qemu::DifftestContext;

mod device;
mod engine;
mod isa;
mod memory;
mod monitor;
mod utils;

pub struct Emulator<T: Isa> {
    cpu: T,
    memory: Rc<RefCell<Memory>>,
    device: Devices,
    difftest_ctx: Option<DifftestContext>,
    batch: bool
}

impl<T: Isa> Emulator<T> {
    pub fn new() -> Self {
        let args = crate::monitor::Args::parse();
        init_log(args.log.as_ref());

        let memory = Rc::new(RefCell::new(Memory::new())); // init mem
        let device = Devices::new(&mut *memory.borrow_mut()); // init device
        let mut cpu = T::new(memory.clone());
        let _img_size = monitor::load_img(args.image.as_ref(), memory.borrow_mut().deref_mut());


        let difftest_ctx = if args.difftest { Some(DifftestContext::init(cpu.isa_difftest_init(), args.image)) } else { None };

        Emulator {
            cpu,
            memory,
            device,
            difftest_ctx,
            batch: args.batch
        }
    }

    pub fn run(&mut self) {
        let cnt = if !self.batch {
            sdb_loop(self)
        } else {
            let mut inst_count= 0;
            loop {
               inst_count += 1;
                let (not_halt, _) = exec_once(self, &mut HashMap::new(), &HashMap::new(), inst_count);
                if !not_halt {
                    break;
                }
            }
            inst_count
        };
        info!("Instruction executed: {}", cnt);
    }

    pub fn exit(mut self) -> ExitCode {
        if let Some(ctx) = &mut self.difftest_ctx {
            ctx.exit();
        }
        self.device.stop();
        ExitCode::from(self.cpu.isa_get_exit_code())
    }
}

fn main() -> ExitCode {
    let mut emulator = Emulator::<RISCV64>::new();
    emulator.run();
    emulator.exit()
}
