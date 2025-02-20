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
use log::{info, warn};
use std::process::ExitCode;
use std::ptr::addr_of_mut;
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
        init_log(args.log.as_ref());

        let mut memory = Memory::new(); // init mem
        let _img_size = monitor::load_img(&args.image, &mut memory);
        let _firm_size = if let Some(path) = &args.firmware {
            monitor::load_firmware(path, &mut memory)
        } else {
            // if CONFIG_MEM_BASE.value() != 0x80000000 {
            //     panic!("TODO: MANUALLY CRAFTED RISCV ASM FOR DEFAULT FIRMWARE");
            // }
            warn!("TODO: MANUALLY CRAFTED RISCV ASM FOR DEFAULT FIRMWARE");
            let firm = [
                0x0810029bu32, // addw	t0,zero,1 (li 0x81000000)
                0x01f29293,    // sll	t0,t0,0x1f
                0x00028067,    // jr	t0
            ];
            let dst = addr_of_mut!(memory.firmware[0]) as *mut u32;
            unsafe {
                std::ptr::copy_nonoverlapping(firm.as_ptr(), dst, firm.len());
            }
            firm.len() * 4
        };
        let stopped = Arc::new(AtomicBool::new(false));
        let device = Devices::new(stopped.clone(), &mut memory, args.no_sdl_devices); // init device
        let mut cpu = T::new(
            stopped.clone(),
            memory,
            device.cpu_interrupt_bits.clone(),
            &args,
        );
        ctrlc::set_handler(move || {
            stopped.store(true, std::sync::atomic::Ordering::Relaxed);
        })
        .unwrap();

        let difftest_ctx = if args.difftest {
            Some(DifftestContext::init(
                cpu.isa_difftest_init(),
                &args.firmware.unwrap(),
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
