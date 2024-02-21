use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use crate::isa::{CPUState, Isa};

fn help(cmd: &str) {
    if &cmd[0..4] != "help" {
        unknown_sdb_command(cmd);
        return
    }
    println!("help: Display information about all supported commands");
    println!("c: Continue the execution of the program");
    println!("q: Exit");
}

fn unknown_sdb_command(cmd: &str) {
    println!("Unknown command: {}", cmd.split_once(' ').unwrap_or((cmd, "")).0);
}

pub fn sdb_loop<U: CPUState, T: Isa<U>>(isa: &mut T) {
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
                    'c' => isa.isa_exec_once();
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