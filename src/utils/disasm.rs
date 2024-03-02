use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr::{self, addr_of_mut};
use std::sync::Mutex;

use llvm_sys::disassembler::LLVMOpaqueDisasmContext;

static INITED: Mutex<bool> = Mutex::new(false);

pub struct LLVMDisassembler {
    dc: *mut LLVMOpaqueDisasmContext,
    buffer: Vec<u8>,
}

impl LLVMDisassembler {
    pub fn new(triple: &str) -> Self {
        use llvm_sys::target;
        let triple_cstr = CString::new(triple).unwrap();
        let triple = triple_cstr.as_ptr() as *const c_char;

        let mut inited = INITED.lock().unwrap();
        if *inited == false {
            unsafe {
                target::LLVM_InitializeAllTargetInfos();
                target::LLVM_InitializeAllTargetMCs();
                target::LLVM_InitializeAllDisassemblers();
            }
            *inited = true;
        }
        let dc;
        unsafe {
            dc = llvm_sys::disassembler::LLVMCreateDisasm(triple, ptr::null_mut(), 0, None, None);
        }
        LLVMDisassembler {
            dc,
            buffer: vec![0; 101],
        }
    }

    pub fn disassemble(&mut self, inst: u32) -> String {
        let mut inst = inst;
        let inst_ptr: *mut u8 = addr_of_mut!(inst) as *mut u8;
        let buf = self.buffer.as_mut_ptr() as *mut i8;
        unsafe {
            // let dc = llvm_sys::disassembler::LLVMCreateDisasm(CString::new("riscv64-unknown-linux-gnu").unwrap().as_ptr() as *const c_char, ptr::null_mut(), 0, None, None);
            llvm_sys::disassembler::LLVMDisasmInstruction(self.dc, inst_ptr, 4, 0, buf, 100);
        }
        let len = self
            .buffer
            .iter()
            .position(|&c| c == 0)
            .expect("a foreign function overflowed the buffer");
        let s = std::str::from_utf8(&self.buffer[..len]).expect("TODO: Handle invalid UTF-8");
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::LLVMDisassembler;

    #[test]
    fn it_works() {
        let mut disasm = LLVMDisassembler::new("riscv64-unknown-linux-gnu");
        println!("{}", disasm.disassemble(0x00000297));
    }
}
