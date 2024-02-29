use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use crate::Emulator;
use crate::isa::Isa;

fn help(cmd: &str) {
    if !cmd.starts_with("help") {
        unknown_sdb_command(cmd);
        return;
    }
    println!("help: Display information about all supported commands");
    println!("c: Continue the execution of the program");
    println!("q: Exit");
}

fn unknown_sdb_command(cmd: &str) {
    println!("Unknown command: {}", cmd.split_once(' ').unwrap_or((cmd, "")).0);
}

pub fn sdb_loop<T: Isa>(emulator: &mut Emulator<T>) {
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).expect("Rustyline err");
                if line.len() == 0 {
                    continue;
                }
                match line.as_bytes()[0] as char {
                    'h' => help(line.as_str()),
                    'c' => {
                        loop {
                            if !emulator.cpu.isa_exec_once() {
                                return;
                            }
                        }
                    },
                    'q' => return,
                    's' => { // si
                        if !emulator.cpu.isa_exec_once() {
                            return;
                        }
                    },
                    'i' => emulator.cpu.isa_reg_display(), // info r(reg) / info w(watchpoint)
                    'x' => {}, // x N expr: mem[eval(expr)..N*4]
                    'p' => {}, // p expr: eval(expr)
                    'w' => {}, // w expr: pause when mem[eval(expr)] changes
                    'd' => {}, // d N: delete watchpoint N

                    _ => unknown_sdb_command(line.as_str())
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