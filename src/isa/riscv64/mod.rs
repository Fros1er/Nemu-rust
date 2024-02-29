use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use log::info;
use strum::IntoEnumIterator;
use crate::cpu::ifetch::ifetch;
use crate::isa::Isa;
use crate::isa::riscv64::inst::{init_patterns, Pattern};
use crate::isa::riscv64::logo::RISCV_LOGO;
use crate::isa::riscv64::reg::{CSR, CSRName, Reg, Registers, RegName};
use crate::isa::riscv64::reg::CSRName::{mcause, mepc};
use crate::isa::riscv64::reg::MCauseCode::Breakpoint;
use crate::memory::Memory;
use crate::memory::paddr::{PAddr};
use crate::memory::vaddr::{MemOperationSize, VAddr};
use crate::utils::configs::{CONFIG_MBASE, CONFIG_PC_RESET_OFFSET};

mod inst;
mod reg;
mod logo;

pub(crate) struct RISCV64 {
    state: RISCV64CpuState,
    instruction_patterns: Vec<Pattern>,
}

impl RISCV64CpuState {
    fn new(memory: Rc<RefCell<Memory>>) -> RISCV64CpuState {
        RISCV64CpuState {
            regs: Registers::new(),
            csrs: CSR::new(),
            pc: VAddr::new(0),
            memory
        }
    }
}

pub(crate) struct RISCV64CpuState {
    regs: Registers,
    csrs: CSR,
    pc: VAddr,
    memory: Rc<RefCell<Memory>>
}

impl RISCV64CpuState {
    fn trap(&mut self) {
        self.csrs[mepc] = self.pc.value() as Reg;
        self.csrs[mcause] = Breakpoint as u64;
        // TODO
    }
}

const IMG: [u32; 5] = [
    0x00000297,  // auipc t0,0
    0x00028823,  // sb  zero,16(t0)
    0x0102c503,  // lbu a0,16(t0)
    0x00100073,  // ebreak (used as nemu_trap)
    0xdeadbeef,  // some data]
];

impl Isa for RISCV64 {
    fn new(memory: Rc<RefCell<Memory>>) -> RISCV64 {
        let reset_addr: PAddr = CONFIG_MBASE + CONFIG_PC_RESET_OFFSET;
        memory.borrow_mut().memcpy_p(&IMG, &reset_addr, IMG.len());
        let mut state = RISCV64CpuState::new(memory);
        state.pc = reset_addr.into();
        RISCV64 {
            state,
            instruction_patterns: init_patterns()
        }
    }
    fn isa_logo() -> &'static [u8] {
        RISCV_LOGO
    }

    // fn cpu_state() -> Box<RISCV64CpuState> {
    //     todo!()
    // }

    fn isa_reg_display(&self) {
        for reg in RegName::iter() {
            println!("{:?}: {:#x}", reg, self.state.regs[reg.clone()]);
        }
        println!("pc: {:#x}", self.state.pc.value());
        for reg in CSRName::iter() {
            println!("{:?}: {:#x}", reg, self.state.csrs[reg]);
        }
    }

    fn isa_reg_str2val(name: &str) -> Result<u64, ()> {
        // todo!()
        Ok(0)
    }

    fn isa_exec_once(&mut self) -> bool {
        // ifetch
        let inst = ifetch(&mut self.state.pc, self.state.memory.borrow().deref(), MemOperationSize::DWORD);
        // decode exec
        match self.instruction_patterns.iter().find(|p| {p.match_inst(&inst)}) {
            None => panic!("invalid inst: {:x}", inst),
            Some(pat) => pat.exec(&inst, &mut self.state)
        }
        self.state.pc.inc(MemOperationSize::DWORD); // TODO
        if self.state.csrs[mcause] == Breakpoint as u64 {
            info!("ebreak at pc {:#x}", self.state.pc.value() - 4);
            return false;
        }
        true
    }

    fn isa_raise_interrupt(no: u64, epc: VAddr) -> VAddr {
        todo!()
    }

    fn isa_query_interrupt() -> u64 {
        todo!()
    }

    // fn isa_difftest_check_regs(ref_r: RISCV64CpuState, pc: VAddr) -> bool {
    //     todo!()
    // }

    fn isa_difftest_attach() {
        todo!()
    }
}