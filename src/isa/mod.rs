use crate::memory::vaddr::VAddr;
use crate::memory::Memory;
use crate::monitor::sdb::difftest_qemu::DifftestInfo;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) mod riscv64;

pub struct InstInfo {
    pub is_branch: bool,
}

pub trait Isa {
    fn new(memory: Rc<RefCell<Memory>>) -> Self;

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

    fn isa_get_prev_inst_info(&mut self, prev_pc: &VAddr) -> Result<InstInfo, ()>;

    fn isa_disassemble_inst(&mut self, addr: &VAddr) -> String;
    // mmu
    // todo
    // interrupt/exception
    // fn isa_raise_interrupt(no: u64, epc: VAddr) -> VAddr;
    // fn isa_query_interrupt() -> u64;
    // difftest
    // fn isa_difftest_check_regs(ref_r: T, pc: VAddr) -> bool;
    fn isa_difftest_init(&mut self) -> DifftestInfo;
    fn isa_difftest_check_regs(&self, difftest_regs: &Vec<u64>) -> Result<(), String>;
}
