use crate::isa::riscv64::csr::mstatus::MStatus;
use crate::isa::riscv64::csr::CSRName::{
    mcause, medeleg, mepc, mstatus, mtval, mtvec, sepc, stvec,
};
use crate::isa::riscv64::csr::{CSRName, CSRs, MCauseCode};
use crate::isa::riscv64::ibuf::SetAssociativeIBuf;
use crate::isa::riscv64::inst::PATTERNS;
use crate::isa::riscv64::logo::RISCV_LOGO;
use crate::isa::riscv64::reg::RegName::{a0, a1, a2, a7, t0};
use crate::isa::riscv64::reg::{format_regs, RegName, Registers};
use crate::isa::riscv64::vaddr::{MemOperationSize, MMU};
use crate::isa::Isa;
use crate::memory::paddr::PAddr;
use crate::memory::Memory;
use crate::monitor::sdb::difftest_qemu::DifftestInfo;
use crate::monitor::Args;
use crate::utils::configs::CONFIG_MEM_BASE;
use crate::utils::disasm::LLVMDisassembler;
use log::{error, info};
use std::fmt::Write;
use std::str::FromStr;
use std::thread;
use strum::IntoEnumIterator;
use strum_macros::FromRepr;
use vaddr::MemOperationSize::DWORD;
use vaddr::VAddr;

mod csr;
mod ibuf;
mod inst;
mod logo;
pub mod reg;
pub mod vaddr;

pub struct RISCV64 {
    state: RISCV64CpuState,
    disassembler: LLVMDisassembler,
    ibuf: SetAssociativeIBuf,
    stop_at_ebreak: bool,
}

#[derive(PartialEq, Copy, Clone, FromRepr, Debug)]
pub enum RISCV64Privilege {
    M,
    S,
    U,
}

pub struct RISCV64CpuState {
    regs: Registers,
    csrs: CSRs,
    pc: VAddr,
    dyn_pc: Option<VAddr>,
    memory: MMU,
    privilege: RISCV64Privilege,
    backtrace: Vec<u64>,
    stopping: bool,
}

impl RISCV64CpuState {
    fn new(memory: Memory, reset_vector: &PAddr) -> Self {
        let mmu = MMU::new(memory);
        Self {
            regs: Registers::new(),
            csrs: CSRs::new(),
            pc: mmu.paddr_to_vaddr(reset_vector),
            dyn_pc: None,
            memory: mmu,
            privilege: RISCV64Privilege::M,
            backtrace: Vec::new(),
            stopping: false,
        }
    }
}

impl Drop for RISCV64CpuState {
    fn drop(&mut self) {
        if thread::panicking() {
            eprintln!(
                "pc: {:#x}\nregs: {}\ncsrs: {}\npriv: {:?}",
                self.pc.value(),
                self.regs,
                self.csrs,
                self.privilege
            );
            eprintln!("Backtrace: ");
            eprint!("{}", self.get_backtrace_string());
        }
    }
}

impl RISCV64CpuState {
    fn trap(&mut self, cause: MCauseCode, mtval_val: Option<u64>) {
        let cause_name: &'static str = (&cause).into();
        info!("trap at {:#x}, caused by {}", self.pc.value(), cause_name);

        if cause == MCauseCode::ECallM && self.regs[a7] == 93 {
            info!("riscv-test passfail triggered");
            if self.regs[a0] == 0 {
                info!("test passed!");
            } else {
                info!("test case {} failed", self.regs[a0]);
            }
            self.stopping = true;
        }

        macro_rules! set_csr {
            ($csr:expr, $val:expr) => {
                let res = self.csrs.set_n($csr, $val);
                if let Some(res) = res {
                    res.call_hook(self)
                }
            };
        }

        let is_deleg = self.privilege != RISCV64Privilege::M
            && (self.csrs[medeleg] & (1u64 << cause as u64)) != 0;
        let prev_priv = self.privilege;
        let next_priv = if is_deleg {
            RISCV64Privilege::S
        } else {
            RISCV64Privilege::M
        };

        // update mstatus
        let mut mstatus_reg: MStatus = self.csrs[mstatus].into();
        mstatus_reg.update_when_trap(prev_priv, next_priv);
        self.csrs.set_n(mstatus, mstatus_reg.into());

        if next_priv == RISCV64Privilege::M {
            set_csr!(mepc, self.pc.value());
            set_csr!(mcause, cause as u64);
            self.dyn_pc = Some(VAddr::new(self.csrs[mtvec].into()));
            if let Some(val) = mtval_val {
                set_csr!(mtval, val);
            }
        } else {
            self.dyn_pc = Some(VAddr::new(self.csrs[stvec].into()));
            todo!("delegate trap to S mode")
        }

        if self.csrs[mtvec] == 0 {
            info!("mtvec unset. Stopping.");
            self.stopping = true;
        }
    }

    fn ret(&mut self, ret_inst: RISCV64Privilege) {
        // update mstatus
        let mut mstatus_reg: MStatus = self.csrs[mstatus].into();
        let next_priv = mstatus_reg.update_when_ret(ret_inst);
        self.csrs.set_n(mstatus, mstatus_reg.into());

        let xepc = if self.privilege == RISCV64Privilege::M {
            mepc
        } else {
            sepc
        };
        self.dyn_pc = Some(VAddr::new(self.csrs[xepc].into()));

        self.privilege = next_priv;
        // info!("Ret mepc: {:#x}", self.csrs[mepc]);
    }

    fn get_backtrace_string(&self) -> String {
        let mut res = String::new();
        for (i, addr) in self.backtrace.iter().rev().enumerate() {
            write!(&mut res, "#{}: {:#x}\n", i, addr - 4).unwrap();
        }
        res
    }
}

impl Isa for RISCV64 {
    fn new(memory: Memory, args: &Args) -> Self {
        // let reset_addr: PAddr = CONFIG_MBASE + CONFIG_PC_RESET_OFFSET;
        let reset_addr: PAddr = PAddr::new(CONFIG_MEM_BASE.value());
        // let reset_addr: PAddr = PAddr::new(CONFIG_FIRMWARE_BASE.value());
        let mut state = RISCV64CpuState::new(memory, &reset_addr);
        Self {
            state,
            disassembler: LLVMDisassembler::new(
                "riscv64-unknown-linux-gnu",
                "rv64imafd_zicsr_zifencei",
            ),
            ibuf: SetAssociativeIBuf::new(),
            stop_at_ebreak: !args.ignore_isa_breakpoint,
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
        self.state.pc.value()
    }

    #[inline]
    fn isa_exec_once(&mut self) -> bool {
        if self.state.stopping {
            return false;
        }
        let pc_paddr: &PAddr = &(&self.state.pc).into();
        let inst = self.state.memory.read(&self.state.pc, DWORD);
        if inst.is_none() {
            self.state
                .trap(MCauseCode::InstAccessFault, Some(self.state.pc.value()));
        } else {
            let inst = inst.unwrap();
            let (pattern, decode) = match self.ibuf.get(pc_paddr, inst) {
                Some(content) => content,
                None => {
                    if inst == 0x0000006f {
                        info!("dead loop at pc {:#x}", self.state.pc.value());
                        self.state.regs[a0] = 1;
                        return false;
                    }
                    // decode exec
                    match PATTERNS.iter().find(|p| p.match_inst(&inst)) {
                        None => {
                            error!(
                                "invalid inst: {:#x} at addr {:#x}",
                                inst,
                                self.state.pc.value()
                            );
                            error!(
                                "disasm as: {}",
                                self.disassembler
                                    .disassemble(inst as u32, self.state.pc.value())
                            );
                            info!("bt:\n{}", self.isa_get_backtrace());
                            self.state.regs[a0] = 1;
                            return false;
                        }
                        Some(pat) => self.ibuf.set(pc_paddr, inst, pat, pat.decode(&inst)),
                    }
                }
            };
            pattern.exec(decode, &mut self.state);
        }
        match &self.state.dyn_pc {
            Some(pc) => {
                self.state.pc = *pc;
                self.state.dyn_pc = None;
            }
            None => self.state.pc.inc(DWORD),
        }

        if self.stop_at_ebreak && self.state.csrs[mcause] == MCauseCode::Breakpoint as u64 {
            info!("ebreak at pc {:#x}", self.state.csrs[mepc]);
            info!("a0: {:#x}", self.state.regs[a0]);
            return false;
        }
        true
    }

    fn isa_get_exit_code(&self) -> u8 {
        self.state.regs[a0] as u8
    }

    fn isa_print_icache_info(&self) {
        self.ibuf.print_info();
    }

    // fn isa_get_prev_inst_info(&mut self, prev_pc: &VAddr) -> Result<InstInfo, ()> {
    //     let inst = self.state.memory.read(prev_pc, DWORD);
    //     let (pattern, _) = self.ibuf.get(&prev_pc.into(), inst).unwrap();
    //     Ok(InstInfo {
    //         is_branch: pattern._name == "jal" || pattern._name == "jalr"
    //     })
    // }

    fn isa_disassemble_inst(&mut self, addr: &VAddr) -> String {
        let inst = self.state.memory.read(addr, DWORD);
        match inst {
            Some(inst) => format!(
                "inst {:#x} at addr {:#x}\nDisassembled as {}",
                inst,
                addr.value(),
                self.disassembler
                    .disassemble(inst as u32, self.state.pc.value())
            ),
            None => format!("Access Fault at addr {:#x}", addr.value()),
        }
    }

    fn read_vaddr(&mut self, addr: &VAddr, len: MemOperationSize) -> Option<u64> {
        self.state.memory.read(addr, len)
    }

    fn isa_get_backtrace(&self) -> String {
        self.state.get_backtrace_string()
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
            return Err(format!(
                "number of regs mismatch: local 33, difftest {}.",
                difftest_regs.len()
            ));
        }

        if self.state.pc.value() != difftest_regs[32] {
            return Err(format!(
                "pc mismatch: local {:#x}, difftest {:#x}.",
                self.state.pc.value(),
                difftest_regs[32]
            ));
        }

        for i in 1..32 {
            if difftest_regs[i] != self.state.regs[i as u64] {
                let reg_str: &str = RegName::iter().nth(i).unwrap().into();
                return Err(format!(
                    "Reg {} is different: local {:#x}, difftest {:#x}.\nfull: {}{}",
                    reg_str,
                    self.state.regs[i as u64],
                    difftest_regs[i],
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
    use crate::isa::riscv64::vaddr::VAddr;
    use crate::monitor::sdb::eval::eval;
    use crate::utils::tests::fake_emulator;

    #[test]
    fn sdb_eval_reg_test() {
        let mut emulator = fake_emulator();
        emulator.cpu.state.regs[a0] = 114;
        emulator.cpu.state.pc = VAddr::new(514);
        let exp = "$a0 * 1000 + $pc".to_string();
        assert_eq!(eval(exp.as_str(), &mut emulator).unwrap(), 114514);
    }
}
