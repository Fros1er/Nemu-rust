use crate::isa::{CPUState, Isa};
use crate::isa::riscv64::inst::{init_patterns, Pattern};
use crate::isa::riscv64::logo::RISCV_LOGO;
use crate::isa::riscv64::reg::Registers;
use crate::memory::paddr::{memcpy_to_pmem, PAddr};
use crate::memory::vaddr::VAddr;
use crate::utils::configs::{CONFIG_MBASE, CONFIG_PC_RESET_OFFSET};

mod inst;
mod reg;
mod logo;

pub(crate) struct RISCV64 {
    state: RISCV64CpuState,
    instruction_patterns: Vec<Pattern>,
}

impl RISCV64 {
    pub(crate) fn new() -> RISCV64 {
        RISCV64 {
            state: RISCV64CpuState::new(),
            instruction_patterns: init_patterns(),
        }
    }
}

impl RISCV64CpuState {
    fn new() -> RISCV64CpuState {
        RISCV64CpuState {
            regs: Registers::new(),
            pc: PAddr::new(0),
        }
    }
}

pub(crate) struct RISCV64CpuState {
    regs: Registers,
    pc: PAddr,
}

impl CPUState for RISCV64CpuState {}

const IMG: [u32; 5] = [
    0x00000297,  // auipc t0,0
    0x00028823,  // sb  zero,16(t0)
    0x0102c503,  // lbu a0,16(t0)
    0x00100073,  // ebreak (used as nemu_trap)
    0xdeadbeef,  // some data]
];

impl Isa<RISCV64CpuState> for RISCV64 {
    fn isa_logo() -> &'static [u8] {
        RISCV_LOGO
    }

    fn init_isa(&mut self) {
        // restart
        let reset_addr: PAddr = CONFIG_MBASE + CONFIG_PC_RESET_OFFSET;
        memcpy_to_pmem(&IMG, &reset_addr, IMG.len());
        self.state.pc = reset_addr;
    }

    fn cpu_state() -> Box<RISCV64CpuState> {
        todo!()
    }

    fn isa_reg_display() {
        // todo!()
    }

    fn isa_reg_str2val(name: &str) -> Result<u64, ()> {
        // todo!()
        Ok(0)
    }

    fn isa_exec_once(&mut self) {
        // ifetch
        // todo!()
        let inst = 0u64;
        // decode exec
        for pat in self.instruction_patterns.iter() {
            if pat.match_inst(&inst) {
                pat.exec(&inst, &mut self.state);
                return
            }
        }
        // no match
        todo!("invalid inst")
    }

    fn isa_raise_interrupt(no: u64, epc: VAddr) -> VAddr {
        todo!()
    }

    fn isa_query_interrupt() -> u64 {
        todo!()
    }

    fn isa_difftest_check_regs(ref_r: RISCV64CpuState, pc: VAddr) -> bool {
        todo!()
    }

    fn isa_difftest_attach() {
        todo!()
    }
}