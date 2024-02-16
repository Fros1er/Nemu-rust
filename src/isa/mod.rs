use crate::memory::vaddr::VAddr;

pub(crate) mod riscv64;

pub struct Instruction(u64);

pub struct AbstractContext {}

pub trait CPUState {}

pub trait Isa<T: CPUState> {
    // monitor
    fn isa_logo() -> &'static [u8];
    fn init_isa(&mut self);
    // reg
    fn cpu_state() -> Box<T>;
    fn isa_reg_display();
    fn isa_reg_str2val(name: &str) -> Result<u64, ()>;
    // exec
    fn isa_exec_once(&mut self);
    // mmu
    // todo
    // interrupt/exception
    fn isa_raise_interrupt(no: u64, epc: VAddr) -> VAddr;
    fn isa_query_interrupt() -> u64;
    // difftest
    fn isa_difftest_check_regs(ref_r: T, pc: VAddr) -> bool;
    fn isa_difftest_attach();
}