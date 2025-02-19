use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::Memory;
use crate::monitor::sdb::difftest_qemu::DifftestInfo;
use crate::monitor::Args;
use riscv64::vaddr::VAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub(crate) mod riscv64;

pub trait Isa {
    fn new(stopped: Arc<AtomicBool>, memory: Memory, args: &Args) -> Self;

    // monitor
    fn isa_logo() -> &'static [u8];
    // reg
    // fn cpu_state() -> Box<T>;
    fn isa_reg_display(&self);
    fn isa_get_reg_by_name(&self, name: &str) -> Result<u64, String>;
    fn isa_get_pc(&self) -> u64;
    // exec, true if not terminate
    fn isa_exec_once(&mut self) -> bool;

    fn isa_get_exit_code(&self) -> u8;

    fn isa_print_icache_info(&self) {
        println!("ICache not implemented")
    }

    // fn isa_get_prev_inst_info(&mut self, prev_pc: &VAddr) -> Result<InstInfo, ()>;

    fn isa_disassemble_inst(&mut self, addr: &VAddr) -> String;
    // mmu
    fn read_vaddr(&mut self, addr: &VAddr, len: MemOperationSize) -> Result<u64, String>;
    // todo
    // interrupt/exception
    // fn isa_raise_interrupt(no: u64, epc: VAddr) -> VAddr;
    // fn isa_query_interrupt() -> u64;
    fn isa_get_backtrace(&self) -> String;

    // difftest
    // fn isa_difftest_check_regs(ref_r: T, pc: VAddr) -> bool;
    fn isa_difftest_init(&mut self) -> DifftestInfo;
    fn isa_difftest_check_regs(&self, difftest_regs: &Vec<u64>) -> Result<(), String>;
}
