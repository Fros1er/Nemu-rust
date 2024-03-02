pub mod eval;

use crate::isa::Isa;
use crate::monitor::sdb::eval::{eval, eval_expr, parse, Expr};
use crate::Emulator;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;

fn unknown_sdb_command(cmd: &str) {
    println!(
        "Unknown command: {}",
        cmd.split_once(' ').unwrap_or((cmd, "")).0
    );
}

struct WatchPoint {
    expr: Expr,
    expr_str: String,
    prev_val: i64,
}

fn exec_once<T: Isa>(
    emulator: &mut Emulator<T>,
    watchpoints: &mut HashMap<u32, WatchPoint>,
) -> (bool, bool) {
    let not_halt = emulator.cpu.isa_exec_once();
    let mut watchpoint_matched = false;
    for (idx, watchpoint) in watchpoints.iter_mut() {
        let eval_res = eval_expr(&watchpoint.expr, emulator);
        if eval_res != Ok(watchpoint.prev_val) {
            match eval_res {
                Ok(res) => {
                    println!("Watchpoint {}: {}", idx, watchpoint.expr_str);
                    println!("Old value = {}", watchpoint.prev_val);
                    println!("New value = {}", res);
                    watchpoint.prev_val = res;
                }
                Err(err) => println!("{}", err),
            }
            watchpoint_matched = true;
        }
    }
    (not_halt, watchpoint_matched)
}

pub fn sdb_loop<T: Isa>(emulator: &mut Emulator<T>) {
    let mut rl = DefaultEditor::new().unwrap();
    let mut watchpoints: HashMap<u32, WatchPoint> = HashMap::new();
    let mut next_watchpoint_idx = 0;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).expect("Rustyline err");
                if line.len() == 0 {
                    continue;
                }
                match line.trim_start().as_bytes()[0] as char {
                    'h' => {
                        if line.starts_with("help") {
                            println!("help: Display information about all supported commands");
                            println!("c: Continue the execution of the program");
                            println!("s: step once");
                            println!("i: display reg");
                            println!("p expr: eval(expr)");
                            println!("w expr: set watchpoint expr");
                            println!("d N: del watchpoint N");
                            println!("q: Exit");
                        } else {
                            unknown_sdb_command(line.as_str());
                        }
                    }
                    'c' => loop {
                        let (not_halt, wp_matched) = exec_once(emulator, &mut watchpoints);
                        if !not_halt {
                            return;
                        }
                        if wp_matched {
                            break;
                        }
                    },
                    'q' => return,
                    's' => {
                        // si
                        if !exec_once(emulator, &mut watchpoints).0 {
                            return;
                        }
                    }
                    'i' => emulator.cpu.isa_reg_display(), // info r(reg) / info w(watchpoint)
                    'x' => {}                              // x N expr: mem[eval(expr)..N*4]
                    'p' => match eval(&line[1..], emulator) {
                        Ok(val) => println!("result: {}", val),
                        Err(err) => println!("{}", err),
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
                            Ok(watchpoint) => {
                                watchpoints.insert(next_watchpoint_idx, watchpoint);
                                println!("watchpoint {}: {}", next_watchpoint_idx, raw_expr);
                                next_watchpoint_idx += 1;
                            }
                            Err(err) => println!("{}", err),
                        }
                    } // w expr: pause when mem[eval(expr)] changes
                    'd' => match line[1..].parse::<u32>() {
                        Ok(num) => match watchpoints.remove(&num) {
                            Some(watchpoint) => println!(
                                "Watchpoint number {} deleted, expr: {}",
                                num, watchpoint.expr_str
                            ),
                            None => println!("No watchpoint number {}", num),
                        },
                        Err(err) => println!("{}", err),
                    }, // d N: delete watchpoint N

                    _ => unknown_sdb_command(line.as_str()),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
