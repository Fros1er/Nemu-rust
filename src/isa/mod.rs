use crate::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) mod riscv64;

pub trait Isa {
    fn new(memory: Rc<RefCell<Memory>>) -> Self;

    // monitor
    fn isa_logo() -> &'static [u8];
    // reg
    // fn cpu_state() -> Box<T>;
    fn isa_reg_display(&self);
    fn isa_get_reg_by_name(&self, name: &str) -> Result<u64, String>;
    // exec, true if not terminate
    fn isa_exec_once(&mut self) -> bool;
    // mmu
    // todo
    // interrupt/exception
    // fn isa_raise_interrupt(no: u64, epc: VAddr) -> VAddr;
    // fn isa_query_interrupt() -> u64;
    // difftest
    // fn isa_difftest_check_regs(ref_r: T, pc: VAddr) -> bool;
    // fn isa_difftest_attach();
}
