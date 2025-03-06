use crate::device::Devices;
use crate::isa::riscv64::RISCV64;
use crate::isa::Isa;
use crate::memory::Memory;
use crate::monitor::init_log;
use crate::monitor::sdb::difftest_qemu::DifftestContext;
use crate::monitor::sdb::{exec_once, sdb_loop};
use crate::utils::cfg_if_feat;
use cfg_if::cfg_if;
use clap::Parser;
use log::info;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

mod device;
mod engine;
mod isa;
mod memory;
mod monitor;
mod utils;

pub struct Emulator<T: Isa> {
    cpu: T,
    device: Devices,
    difftest_ctx: Option<DifftestContext>,
    batch: bool,
    exitcode: u8,
}

impl<T: Isa> Emulator<T> {
    pub fn new() -> Self {
        let args = crate::monitor::Args::parse();
        init_log(&args);

        let mut memory = Memory::new(); // init mem
        if let Some(path) = &args.image {
            monitor::load_img(path, &mut memory);
        }
        let firm_size = monitor::load_firmware(&args.firmware, args.image.is_some(), &mut memory);
        info!("Firmware size: {:#x}", firm_size);

        let stopped = Arc::new(AtomicBool::new(false));
        let device = Devices::new(stopped.clone(), &mut memory, &args); // init device
        let mut cpu = T::new(
            stopped.clone(),
            memory,
            device.cpu_interrupt_bits.clone(),
            &args,
        );

        let difftest_ctx = if args.difftest {
            Some(DifftestContext::init(
                cpu.isa_difftest_init(),
                &args.firmware,
                &"".to_string(), // &args.image.unwrap(),
            ))
        } else {
            None
        };

        Emulator {
            cpu,
            device,
            difftest_ctx,
            batch: args.batch,
            exitcode: 0,
        }
    }

    pub fn run(&mut self) {
        let cnt = if !self.batch {
            let (inst_cont, exitcode) = sdb_loop(self);
            self.exitcode = exitcode;
            inst_cont
        } else {
            #[allow(unused_mut)]
            let mut inst_count = 0;
            loop {
                cfg_if_feat!("log_inst", {
                    inst_count += 1;
                });
                let (not_halt, _, sdl_quit) = exec_once(self);
                if !not_halt {
                    self.exitcode = self.cpu.isa_get_exit_code();
                    break;
                }
                if sdl_quit {
                    break;
                }
            }
            inst_count
        };
        info!("Instruction executed: {}", cnt);
        self.cpu.isa_print_icache_info();
    }

    pub fn exit(mut self) -> ExitCode {
        if let Some(ctx) = &mut self.difftest_ctx {
            ctx.exit();
        }
        self.device.stop();
        ExitCode::from(self.exitcode)
    }
}

fn main() -> ExitCode {
    let mut emulator = Emulator::<RISCV64>::new();
    emulator.run();
    emulator.exit()
}
