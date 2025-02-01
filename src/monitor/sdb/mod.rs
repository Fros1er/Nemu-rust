pub mod difftest_qemu;
pub mod eval;
mod gdb_interface;

use crate::isa::Isa;
use crate::memory::vaddr::VAddr;
use crate::monitor::sdb::eval::{eval, eval_expr, parse, Expr};
use crate::utils::cfg_if_feat;
use crate::Emulator;
use cfg_if::cfg_if;
use log::{error, info};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;

fn unknown_sdb_command(cmd: &str) {
    error!(
        "Unknown command: {}",
        cmd.split_once(' ').unwrap_or((cmd, "")).0
    );
}

pub struct WatchPoint {
    expr: Expr,
    expr_str: String,
    prev_val: i64,
}

#[inline]
pub fn exec_once<T: Isa>(emulator: &mut Emulator<T>) -> (bool, bool, bool) {
    let sdl_quit = emulator.device.has_stopped();
    let not_halt = emulator.cpu.isa_exec_once();
    (not_halt, false, sdl_quit)
}

struct DbgContext {
    watchpoints: HashMap<u32, WatchPoint>,
    breakpoints: HashMap<u32, u64>,
    inst_count: u64,
    next_watchpoint_idx: u32,
    next_breakpoint_idx: u32,
    fn_trace_enable: bool,
    prev_pc: u64,
}

impl DbgContext {
    fn new() -> Self {
        Self {
            watchpoints: HashMap::new(),
            breakpoints: HashMap::new(),
            inst_count: 0,
            next_watchpoint_idx: 0,
            next_breakpoint_idx: 0,
            fn_trace_enable: false,
            prev_pc: 0,
        }
    }
    fn exec_once_dbg<T: Isa>(&mut self, emulator: &mut Emulator<T>) -> (bool, bool, bool) {
        cfg_if_feat!("difftest", {
            if emulator.difftest_ctx.is_some() {
                emulator.difftest_ctx.as_mut().unwrap().gdb_ctx.step();
            }
        });

        let pc = emulator.cpu.isa_get_pc();

        let (not_halt, _, sdl_quit) = exec_once(emulator);

        let inst = emulator.cpu.isa_get_prev_inst_info(&VAddr::new(pc));
        if self.fn_trace_enable && inst.is_ok_and(|i| i.is_branch) {
            info!("Function call at {:#x}", pc)
        }
        self.prev_pc = pc;

        let mut pause = false;

        cfg_if_feat!("difftest", {
            if emulator.difftest_ctx.is_some() {
                let difftest_regs = emulator
                    .difftest_ctx
                    .as_mut()
                    .unwrap()
                    .gdb_ctx
                    .read_regs_64();
                let difftest_res = emulator.cpu.isa_difftest_check_regs(&difftest_regs);
                if difftest_res.is_err() {
                    info!("{}", difftest_res.err().unwrap());
                    return (false, false, false);
                }
                info!(
                    "identical at pc {:#x}, {} inst in total",
                    emulator.cpu.isa_get_pc(),
                    self.inst_count
                );
            }
        });

        for (idx, watchpoint) in self.watchpoints.iter_mut() {
            let eval_res = eval_expr(&watchpoint.expr, emulator);
            if eval_res != Ok(watchpoint.prev_val) {
                match eval_res {
                    Ok(res) => {
                        info!("Watchpoint {}: {}", idx, watchpoint.expr_str);
                        info!("Old value = {}", watchpoint.prev_val);
                        info!("New value = {}", res);
                        watchpoint.prev_val = res;
                    }
                    Err(err) => info!("{}", err),
                }
                pause = true;
            }
        }
        for (idx, breakpoint) in self.breakpoints.iter() {
            if emulator.cpu.isa_get_pc() == *breakpoint {
                info!("Breakpoint {}: {}", idx, breakpoint);
                pause = true;
                break;
            }
        }
        (not_halt, pause, sdl_quit)
    }

    fn insert_watchpoint(&mut self, wp: WatchPoint, raw_expr: &str) {
        self.watchpoints.insert(self.next_watchpoint_idx, wp);
        info!("watchpoint {}: {}", self.next_watchpoint_idx, raw_expr);
        self.next_watchpoint_idx += 1;
    }

    fn insert_breakpoint(&mut self, addr: u64) {
        self.breakpoints.insert(self.next_breakpoint_idx, addr);
        info!("breakpoint {}: {:#x}", self.next_breakpoint_idx, addr);
        self.next_breakpoint_idx += 1;
    }
}

impl Drop for DbgContext {
    fn drop(&mut self) {
        eprintln!("prev_pc: {:#x}", self.prev_pc)
    }
}

pub fn sdb_loop<T: Isa>(emulator: &mut Emulator<T>) -> (u64, u8) {
    let mut ctx = DbgContext::new();
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline(format!("({:#x})>> ", emulator.cpu.isa_get_pc()).as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).expect("Rustyline err");
                if line.len() == 0 {
                    continue;
                }
                let line = line.trim_start();
                match line.as_bytes()[0] as char {
                    'h' => {
                        if line.starts_with("help") {
                            info!("help: Display information about all supported commands");
                            info!("c: Continue the execution of the program");
                            info!("s: step once");
                            info!("i: display reg");
                            info!("p expr: eval(expr)");
                            info!("w expr: set watchpoint expr");
                            info!("b addr: set breakpoint addr");
                            info!("d N: del watchpoint N");
                            info!("disasm: disassemble current instruction");
                            info!("q: Exit");
                        } else {
                            unknown_sdb_command(line);
                        }
                    }
                    'c' => loop {
                        ctx.inst_count += 1;
                        let (not_halt, wp_matched, sdl_quit) = ctx.exec_once_dbg(emulator);
                        if !not_halt {
                            return (ctx.inst_count, emulator.cpu.isa_get_exit_code());
                        }
                        if sdl_quit {
                            return (ctx.inst_count, 0);
                        }
                        if wp_matched {
                            break;
                        }
                    },
                    'q' => return (ctx.inst_count, 0),
                    's' => {
                        // si
                        ctx.inst_count += 1;
                        let (not_halt, _, sdl_quit) = ctx.exec_once_dbg(emulator);
                        if !not_halt {
                            return (ctx.inst_count, emulator.cpu.isa_get_exit_code());
                        }
                        if sdl_quit {
                            return (ctx.inst_count, 0);
                        }
                    }
                    'i' => emulator.cpu.isa_reg_display(), // info r(reg) / info w(watchpoint)
                    'x' => {}                              // x N expr: mem[eval(expr)..N*4]
                    'p' => match eval(&line[1..], emulator) {
                        Ok(val) => info!("result: {:#x}", val),
                        Err(err) => info!("{}", err),
                    }, // p expr: eval(expr)
                    'w' => {
                        let raw_expr = line[1..].trim();
                        let watchpoint = parse(raw_expr).and_then(|expr| {
                            let prev_val = eval_expr(&expr, emulator)?;
                            Ok(WatchPoint {
                                expr,
                                expr_str: raw_expr.to_string(),
                                prev_val,
                            })
                        });
                        match watchpoint {
                            Ok(watchpoint) => ctx.insert_watchpoint(watchpoint, raw_expr),
                            Err(err) => info!("{}", err),
                        }
                    } // w expr: pause when mem[eval(expr)] changes
                    'b' => match u64::from_str_radix(line[1..].trim(), 16) {
                        Ok(addr) => ctx.insert_breakpoint(addr),
                        Err(err) => info!("{}", err),
                    },
                    'd' => {
                        if line.starts_with("disasm") {
                            info!(
                                "{}",
                                emulator
                                    .cpu
                                    .isa_disassemble_inst(&VAddr::new(emulator.cpu.isa_get_pc()))
                            );
                        } else {
                            match line[1..].parse::<u32>() {
                                Ok(num) => match ctx.watchpoints.remove(&num) {
                                    Some(watchpoint) => info!(
                                        "Watchpoint number {} deleted, expr: {}",
                                        num, watchpoint.expr_str
                                    ),
                                    None => info!("No watchpoint number {}", num),
                                },
                                Err(err) => info!("{}", err),
                            }
                        }
                    } // d N: delete watchpoint N
                    't' => {
                        ctx.fn_trace_enable = !ctx.fn_trace_enable;
                        info!("Fn trace enable: {}", ctx.fn_trace_enable);
                    } // fn trace toggle

                    _ => unknown_sdb_command(line),
                }
            }
            Err(ReadlineError::Interrupted) => {
                info!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                info!("CTRL-D");
                break;
            }
            Err(err) => {
                info!("Readline Error: {:?}", err);
                break;
            }
        }
    }
    (ctx.inst_count, 0)
}
