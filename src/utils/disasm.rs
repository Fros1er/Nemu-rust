use std::process::{Command, Stdio};

pub struct LLVMDisassembler;
impl LLVMDisassembler {
    pub fn new(_: &str, _: &str) -> Self {
        Self {}
    }
    pub fn disassemble(&mut self, inst: u32, pc: u64) -> String {
        let output = Command::new("disasm-util/target/release/disasm-util")
            .arg(inst.to_string())
            .arg(pc.to_string())
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        String::from_utf8(output.stdout).unwrap()
    }
}
