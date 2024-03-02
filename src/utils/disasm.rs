#[cfg(test)]
mod tests {
    use llvm_sys::disassembler::{LLVMCreateDisasm, LLVMDisasmInstruction};
    use llvm_sys::target::{
        LLVM_InitializeAllDisassemblers, LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs,
    };
    use std::ffi::CString;
    use std::os::raw::c_char;
    use std::ptr;
    use std::ptr::addr_of_mut;

    #[test]
    fn it_works() {
        let c_str = CString::new("x86_64-unknown-linux-gnu").unwrap();
        let triple: *const c_char = c_str.as_ptr() as *const c_char;
        let dc;
        unsafe {
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllDisassemblers();
            dc = LLVMCreateDisasm(triple, *ptr::null(), 0, *ptr::null(), *ptr::null());
        }

        let mut inst: u32 = 0x00000297;
        let ptr_inst: *mut u8 = addr_of_mut!(inst) as *mut u8;
        let mut v = vec![0; 101];
        let ptr = v.as_mut_ptr() as *mut i8;
        let len;
        unsafe {
            len = LLVMDisasmInstruction(dc, ptr_inst, 4, 0, ptr, 100);
        }
        v.truncate(len);
        // Reference it as a `&str`
        let s = str::from_utf8(&v).expect("TODO: Handle invalid UTF-8");
        println!("{}", s);
        ()
    }
}
