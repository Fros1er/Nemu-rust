use crate::isa::riscv64::inst::PATTERNS;
use crate::isa::riscv64::logo::RISCV_LOGO;
use crate::isa::riscv64::reg::CSRName::{mcause, mepc};
use crate::isa::riscv64::reg::MCauseCode::Breakpoint;
use crate::isa::riscv64::reg::{CSRName, Reg, RegName, Registers, CSR, format_regs};
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
use crate::isa::riscv64::ibuf::SetAssociativeIBuf;
use crate::isa::riscv64::reg::RegName::{a0, a1, a2, t0};
use crate::monitor::sdb::difftest_qemu::DifftestInfo;

mod inst;
mod logo;
pub mod reg;
mod ibuf;

pub(crate) struct RISCV64 {
    state: RISCV64CpuState,
    disassembler: LLVMDisassembler,
    ibuf: SetAssociativeIBuf,
}

pub(crate) struct RISCV64CpuState {
    regs: Registers,
    csrs: CSR,
    pc: VAddr,
    dyn_pc: Option<VAddr>,
    memory: Rc<RefCell<Memory>>,
}

impl RISCV64CpuState {
    fn new(memory: Rc<RefCell<Memory>>) -> Self {
        Self {
            regs: Registers::new(),
            csrs: CSR::new(),
            pc: VAddr::new(0),
            dyn_pc: None,
            memory,
        }
    }
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
    fn new(memory: Rc<RefCell<Memory>>) -> Self {
        let reset_addr: PAddr = CONFIG_MBASE + CONFIG_PC_RESET_OFFSET;
        memory.borrow_mut().pmem_memcpy(&IMG, &reset_addr, 5);
        let mut state = RISCV64CpuState::new(memory);
        state.pc = reset_addr.into();
        Self {
            state,
            disassembler: LLVMDisassembler::new("riscv64-unknown-linux-gnu"),
            ibuf: SetAssociativeIBuf::new(CONFIG_MBASE),
        }
    }
    fn isa_logo() -> &'static [u8] {
        RISCV_LOGO
    }
    fn isa_reg_display(&self) {
        for reg in RegName::iter() {
            info!("{:?}: {:#x}", reg, self.state.regs[reg.clone()]);
        }
        info!("pc: {:#x}", self.state.pc.value());
        for reg in CSRName::iter() {
            info!("{:?}: {:#x}", reg, self.state.csrs[reg]);
        }
    }

    fn isa_get_reg_by_name(&self, name: &str) -> Result<u64, String> {
        if name == "pc" {
            return Ok(self.state.pc.value());
        }
        if let Ok(reg) = RegName::from_str(name) {
            return Ok(self.state.regs[reg]);
        }
        if let Ok(csr) = CSRName::from_str(name) {
            return Ok(self.state.csrs[csr]);
        }
        Err("Reg not found".to_string())
    }

    fn isa_get_pc(&self) -> u64 {
        self.state.pc.value() as u64
    }

    fn isa_exec_once(&mut self) -> bool {
        let pc_paddr: &PAddr = &(&self.state.pc).into();
        let (pattern, decode) = match self.ibuf.get(pc_paddr) {
            Some(content) => content,
            None => {
                let inst = self.state.memory.borrow().ifetch(&self.state.pc, DWORD);
                if inst == 0x0000006f {
                    info!("dead loop at pc {:#x}", self.state.pc.value());
                    self.state.regs[a0] = 1;
                    return false;
                }
                // decode exec
                match PATTERNS
                    .iter()
                    .find(|p| p.match_inst(&inst))
                {
                    None => {
                        error!("invalid inst: {:#x} at addr {:#x}", inst, self.state.pc.value());
                        error!("disasm as: {}", self.disassembler.disassemble(inst as u32, self.state.pc.value()));
                        self.state.regs[a0] = 1;
                        return false;
                    }
                    Some(pat) => {
                        self.ibuf.set(pc_paddr, pat, pat.decode(&inst))
                    }
                }
            }
        };
        pattern.exec(decode, &mut self.state);

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

    fn isa_get_exit_code(&self) -> u8 {
        self.state.regs[a0] as u8
    }

    fn isa_print_icache_info(&self) {
        self.ibuf.print_info()
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

    fn isa_difftest_init(&mut self) -> DifftestInfo {
        self.state.regs[t0] = 0x80000000;
        self.state.regs[a1] = 0x8fe00000;
        self.state.regs[a2] = 0x1028;
        DifftestInfo {
            qemu_bin: "/opt/qemu/bin/qemu-system-riscv64".to_string(),
            reset_vec: 0x80000000,
        }
    }

    fn isa_difftest_check_regs(&self, difftest_regs: &Vec<u64>) -> Result<(), String> {
        if difftest_regs.len() != 33 {
            return Err(format!("number of regs mismatch: local 33, difftest {}.",
                               difftest_regs.len()));
        }

        if self.state.pc.value() != difftest_regs[32] {
            return Err(format!("pc mismatch: local {:#x}, difftest {:#x}.", self.state.pc.value(), difftest_regs[32]));
        }

        for i in 1..32 {
            if difftest_regs[i] != self.state.regs[i as u64] {
                let reg_str: &str = RegName::iter().nth(i).unwrap().into();
                return Err(format!("Reg {} is different: local {:#x}, difftest {:#x}.\nfull: {}{}",
                                   reg_str, self.state.regs[i as u64], difftest_regs[i],
                                   format_regs(&(self.state.regs.0), self.state.pc.value()),
                                   format_regs(&difftest_regs[..32], difftest_regs[32]),
                ));
            }
        }

        Ok(())
    }
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
