use crate::isa::riscv64::inst::{init_patterns, Pattern};
use crate::isa::riscv64::logo::RISCV_LOGO;
use crate::isa::riscv64::reg::CSRName::{mcause, mepc};
use crate::isa::riscv64::reg::MCauseCode::Breakpoint;
use crate::isa::riscv64::reg::{CSRName, Reg, RegName, Registers, CSR};
use crate::isa::Isa;
use crate::memory::paddr::PAddr;
use crate::memory::vaddr::MemOperationSize::DWORD;
use crate::memory::vaddr::VAddr;
use crate::memory::Memory;
use crate::utils::configs::{CONFIG_MBASE, CONFIG_PC_RESET_OFFSET};
use crate::utils::disasm::LLVMDisassembler;
use log::{error, info};
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use strum::IntoEnumIterator;

mod inst;
mod logo;
mod reg;

pub(crate) struct RISCV64 {
    state: RISCV64CpuState,
    instruction_patterns: Vec<Pattern>,
    disassembler: LLVMDisassembler,
}

impl RISCV64CpuState {
    fn new(memory: Rc<RefCell<Memory>>) -> RISCV64CpuState {
        RISCV64CpuState {
            regs: Registers::new(),
            csrs: CSR::new(),
            pc: VAddr::new(0),
            dyn_pc: None,
            memory,
        }
    }
}

pub(crate) struct RISCV64CpuState {
    regs: Registers,
    csrs: CSR,
    pc: VAddr,
    dyn_pc: Option<VAddr>,
    memory: Rc<RefCell<Memory>>,
}

impl RISCV64CpuState {
    fn trap(&mut self) {
        self.csrs[mepc] = self.pc.value() as Reg;
        self.csrs[mcause] = Breakpoint as u64;
        // TODO
    }
}

const IMG: [u32; 5] = [
    0x00000297, // auipc t0,0
    0x00028823, // sb  zero,16(t0)
    0x0102c503, // lbu a0,16(t0)
    0x00100073, // ebreak (used as nemu_trap)
    0xdeadbeef, // some data]
];

impl Isa for RISCV64 {
    fn new(memory: Rc<RefCell<Memory>>) -> RISCV64 {
        let reset_addr: PAddr = CONFIG_MBASE + CONFIG_PC_RESET_OFFSET;
        memory.borrow_mut().memcpy_p(&IMG, &reset_addr, IMG.len());
        let mut state = RISCV64CpuState::new(memory);
        state.pc = reset_addr.into();
        RISCV64 {
            state,
            instruction_patterns: init_patterns(),
            disassembler: LLVMDisassembler::new("riscv64-unknown-linux-gnu"),
        }
    }
    fn isa_logo() -> &'static [u8] {
        RISCV_LOGO
    }
    fn isa_reg_display(&self) {
        for reg in RegName::iter() {
            println!("{:?}: {:#x}", reg, self.state.regs[reg.clone()]);
        }
        println!("pc: {:#x}", self.state.pc.value());
        for reg in CSRName::iter() {
            println!("{:?}: {:#x}", reg, self.state.csrs[reg]);
        }
    }

    fn isa_get_reg_by_name(&self, name: &str) -> Result<u64, String> {
        if name == "pc" {
            return Ok(self.state.pc.value() as u64);
        }
        if let Ok(reg) = RegName::from_str(name) {
            return Ok(self.state.regs[reg]);
        }
        if let Ok(csr) = CSRName::from_str(name) {
            return Ok(self.state.csrs[csr]);
        }
        Err("Reg not found".to_string())
    }

    fn isa_exec_once(&mut self) -> bool {
        // ifetch
        let inst = self.state.memory.borrow().ifetch(&self.state.pc, DWORD);
        // decode exec
        match self
            .instruction_patterns
            .iter()
            .find(|p| p.match_inst(&inst))
        {
            None => {
                error!("invalid inst: {:#x} at addr {:#x}", inst, self.state.pc.value());
                error!("disasm as: {}", self.disassembler.disassemble(inst as u32));
                return false;
            }
            Some(pat) => pat.exec(&inst, &mut self.state),
        }

        match &self.state.dyn_pc {
            Some(pc) => self.state.pc = *pc,
            None => self.state.pc.inc(DWORD),
        }
        self.state.dyn_pc = None;
        self.state.regs[0] = 0;

        if self.state.csrs[mcause] == Breakpoint as u64 {
            info!("ebreak at pc {:#x}", self.state.pc.value() - 4);
            return false;
        }
        true
    }

    // fn isa_raise_interrupt(no: u64, epc: VAddr) -> VAddr {
    //     todo!()
    // }
    //
    // fn isa_query_interrupt() -> u64 {
    //     todo!()
    // }

    // fn isa_difftest_check_regs(ref_r: RISCV64CpuState, pc: VAddr) -> bool {
    //     todo!()
    // }

    // fn isa_difftest_attach() {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    use crate::isa::riscv64::reg::RegName::a0;
    use crate::memory::vaddr::VAddr;
    use crate::monitor::sdb::eval::eval;
    use crate::utils::tests::fake_emulator;

    #[test]
    fn sdb_eval_reg_test() {
        let mut emulator = fake_emulator();
        emulator.cpu.state.regs[a0] = 114;
        emulator.cpu.state.pc = VAddr::new(514);
        let exp = "$a0 * 1000 + $pc".to_string();
        assert_eq!(eval(exp.as_str(), &emulator).unwrap(), 114514);
    }
}
