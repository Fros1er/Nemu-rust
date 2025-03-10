use crate::isa::riscv64::csr::mstatus::MStatus;
use crate::isa::riscv64::csr::CSRName::{
    mcause, medeleg, mepc, mideleg, mie, mstatus, mtval, mtvec, scause, sepc, sie, stval, stvec,
};
use crate::isa::riscv64::csr::MCauseCode::{MExtInt, MTimerInt, SExtInt, STimerInt};
use crate::isa::riscv64::csr::{CSRName, CSRs, MCauseCode};
use crate::isa::riscv64::ibuf::SetAssociativeIBuf;
use crate::isa::riscv64::inst::{Pattern, PATTERNS};
use crate::isa::riscv64::logo::RISCV_LOGO;
use crate::isa::riscv64::reg::RegName::{a0, a1, a2, a7, t0};
use crate::isa::riscv64::reg::{format_regs, RegName, Registers};
use crate::isa::riscv64::vaddr::{MemOperationSize, MMU};
use crate::isa::Isa;
use crate::memory::paddr::PAddr;
use crate::memory::Memory;
use crate::monitor::sdb::difftest_qemu::DifftestInfo;
use crate::monitor::Args;
use crate::utils::cfg_if_feat;
use crate::utils::configs::CONFIG_MEM_BASE;
use crate::utils::disasm::LLVMDisassembler;
use cfg_if::cfg_if;
use log::{debug, error, info, warn};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::{fs, thread};
use strum::IntoEnumIterator;
use strum_macros::FromRepr;
use vaddr::MemOperationSize::DWORD;
use vaddr::VAddr;

pub mod csr;
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
    stopped: Arc<AtomicBool>,
    sorted_patterns: Vec<&'static Pattern>,
}

#[derive(PartialEq, Copy, Clone, FromRepr, Debug)]
pub enum RISCV64Privilege {
    M = 0b11,
    S = 0b1,
    U = 0b0,
}

pub struct RISCV64CpuState {
    regs: Registers,
    csrs: CSRs,
    pc: VAddr,
    dyn_pc: Option<VAddr>,
    memory: MMU,
    privilege: Rc<UnsafeCell<RISCV64Privilege>>,
    backtrace: Vec<u64>,
    stopping: bool,
    wfi: bool,
    cycles: Rc<UnsafeCell<u64>>,
    pub inst_counter: HashMap<*const Pattern, u64>,
    interrupt_bits: Arc<AtomicU64>,
    prev_interrupt_bits: u64,
    interrupt_cond_dirty: bool,
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
    #[allow(unused_mut)]
    fn new(memory: Memory, reset_vector: &PAddr, interrupt_bits: Arc<AtomicU64>) -> Self {
        let privilege = Rc::new(UnsafeCell::new(RISCV64Privilege::M));
        let mmu = MMU::new(memory, privilege.clone());
        let cycles = Rc::new(UnsafeCell::new(0));

        let mut inst_counter = HashMap::new();

        cfg_if_feat!("log_inst", {
            for pat in PATTERNS.iter() {
                inst_counter.insert(pat as *const Pattern, 0);
            }
        });

        Self {
            regs: Registers::new(),
            csrs: CSRs::new(cycles.clone(), interrupt_bits.clone()),
            pc: mmu.paddr_to_vaddr(reset_vector),
            dyn_pc: None,
            memory: mmu,
            privilege,
            backtrace: Vec::new(),
            stopping: false,
            wfi: false,
            cycles,
            inst_counter,
            interrupt_bits,
            prev_interrupt_bits: 0,
            interrupt_cond_dirty: true,
        }
    }
    #[inline(never)]
    pub(crate) fn handle_interrupt(&mut self) {
        let interrupt_bits = self.interrupt_bits.load(SeqCst);
        if interrupt_bits == 0 {
            return;
        }

        if self.prev_interrupt_bits == interrupt_bits && self.interrupt_cond_dirty == false {
            return;
        }

        self.interrupt_cond_dirty = false;
        self.prev_interrupt_bits = interrupt_bits;

        // interrupt handle
        let prev_priv = self.current_priv();
        let mut goto_s_mode = false;
        let mut cause = MCauseCode::None;

        let mideleg_set = self.csrs[mideleg] & interrupt_bits != 0;
        let mstatus_val = MStatus::from_bits(self.csrs[mstatus]);
        if prev_priv != RISCV64Privilege::M {
            if mideleg_set
                && (prev_priv == RISCV64Privilege::U
                    || (mstatus_val.SIE() || self.wfi)) // if S, check SIE. if U, ignore SIE
                && (interrupt_bits & self.csrs[sie] != 0)
            {
                goto_s_mode = true;
                if interrupt_bits & (1 << 5) != 0 {
                    cause = STimerInt;
                } else if interrupt_bits & (1 << 9) != 0 {
                    cause = SExtInt;
                } else {
                    panic!(
                        "Unknown S mode interrupt: {}, sie: {}",
                        interrupt_bits, self.csrs[sie]
                    )
                }
            }
        } else {
            if !(mstatus_val.MIE() || self.wfi) {
                return;
            }
        }

        if !goto_s_mode {
            if (interrupt_bits & self.csrs[mie] == 0) || mideleg_set {
                return;
            }

            if interrupt_bits & (1 << 7) != 0 {
                cause = MTimerInt;
            } else if interrupt_bits & (1 << 11) != 0 {
                cause = MExtInt;
            } else {
                panic!(
                    "Unknown M mode interrupt: {}, mie: {}",
                    interrupt_bits, self.csrs[mie]
                )
            }
        }

        if cause == MCauseCode::None {
            panic!(
                "Unknown interrupt: {}, mie: {}",
                interrupt_bits, self.csrs[mie]
            )
        }

        let next_priv = if goto_s_mode {
            RISCV64Privilege::S
        } else {
            RISCV64Privilege::M
        };
        debug!(
            "interrupt {:?} at pc {:#x}, from {:?} to {:?}, sie {}",
            cause,
            self.pc.value(),
            prev_priv,
            next_priv,
            mstatus_val.SIE()
        );
        self.set_interrupt_cond_dirty();
        self.wfi = false;
        self.trap_update_csrs(cause, prev_priv, next_priv, None);
    }

    fn trap_update_csrs(
        &mut self,
        cause: MCauseCode,
        prev_priv: RISCV64Privilege,
        next_priv: RISCV64Privilege,
        mtval_val: Option<u64>,
    ) {
        macro_rules! set_csr {
            ($csr:expr, $val:expr) => {
                let res = self.csrs.set_n($csr, $val);
                if let Ok(res) = res {
                    res.call_hook(self)
                }
            };
        }
        // update mstatus
        let mut mstatus_reg: MStatus = self.csrs[mstatus].into();
        mstatus_reg.update_when_trap(prev_priv, next_priv);
        set_csr!(mstatus, mstatus_reg.into());

        if next_priv == RISCV64Privilege::M {
            set_csr!(mepc, self.pc.value());
            set_csr!(mcause, cause as u64);
            self.dyn_pc = Some(VAddr::new(self.csrs[mtvec].into()));
            if let Some(val) = mtval_val {
                set_csr!(mtval, val);
            }
        } else {
            set_csr!(sepc, self.pc.value());
            set_csr!(scause, cause as u64);
            self.dyn_pc = Some(VAddr::new(self.csrs[stvec].into()));
            if let Some(val) = mtval_val {
                set_csr!(stval, val);
            }
        }

        // info!(
        //     "trap_update_csrs by trap/intr at pc {:#x}, cause {:?}({:#x}) from {:?} to {:?}",
        //     self.pc.value(),
        //     cause,
        //     cause as u64,
        //     prev_priv,
        //     next_priv
        // );
        self.set_priv(next_priv);
    }

    fn trap(&mut self, cause: MCauseCode, mtval_val: Option<u64>) {
        if cause != MCauseCode::ECallM && cause != MCauseCode::ECallS {
            let cause_name: &'static str = (&cause).into();
            debug!("trap at {:#x}, caused by {}", self.pc.value(), cause_name);
        }

        if (cause == MCauseCode::ECallM
            || cause == MCauseCode::ECallS
            || cause == MCauseCode::ECallU)
            && self.regs[a7] == 93
        {
            info!("riscv-test passfail triggered");
            if self.regs[a0] == 0 {
                info!("test passed!");
            } else {
                error!("test case {} failed", self.regs[a0] >> 1);
            }
            self.stopping = true;
            return;
        }

        let is_deleg = self.current_priv() != RISCV64Privilege::M
            && (self.csrs[medeleg] & (1u64 << cause as u64)) != 0;
        let prev_priv = self.current_priv();
        let next_priv = if is_deleg {
            RISCV64Privilege::S
        } else {
            RISCV64Privilege::M
        };

        // info!(
        //     "medeleg {:#x}, is_deleg {}, from {:?} to {:?}",
        //     self.csrs[medeleg], is_deleg, prev_priv, next_priv
        // );

        if self.current_priv() == RISCV64Privilege::U {
            debug!(
                "User trap! medeleg = {:#x}, cause = {:?}, next_priv = {:?}, to stvec {:#x}",
                self.csrs[medeleg], cause, next_priv, self.csrs[stvec]
            )
        } else if self.current_priv() == RISCV64Privilege::S {
            debug!(
                "Kernel trap! mstatus = {:?}",
                MStatus::from_bits(self.csrs[mstatus])
            );
        }

        self.trap_update_csrs(cause, prev_priv, next_priv, mtval_val);

        if self.csrs[mtvec] == 0 {
            error!("mtvec unset. Stopping.");
            self.stopping = true;
        }
    }

    fn ret(&mut self, ret_inst: RISCV64Privilege) {
        if ret_inst != self.current_priv() {
            self.trap(MCauseCode::IllegalInst, None);
            return;
        }

        // update mstatus
        let mut mstatus_reg: MStatus = self.csrs[mstatus].into();
        let next_priv = mstatus_reg.update_when_ret(ret_inst);
        if let Ok(res) = self.csrs.set_n(mstatus, mstatus_reg.into()) {
            res.call_hook(self)
        }

        if ret_inst == RISCV64Privilege::S {
            assert_ne!(next_priv, RISCV64Privilege::M);
        }

        let xepc = if ret_inst == RISCV64Privilege::M {
            mepc
        } else {
            sepc
        };
        self.dyn_pc = Some(VAddr::new(self.csrs[xepc].into()));

        // info!(
        //     "Ret at {:#x} epc: v({:#x}) p({:#x}), from {:?} to {:?}",
        //     self.pc.value(),
        //     self.csrs[xepc],
        //     self.memory
        //         .translate(&VAddr::new(self.csrs[xepc]), MemoryAccessType::X)
        //         .unwrap_or(PAddr::new(0)),
        //     self.privilege,
        //     next_priv
        // );

        self.set_priv(next_priv);
    }

    fn get_backtrace_string(&self) -> String {
        let mut res = String::new();
        for (i, addr) in self.backtrace.iter().rev().enumerate() {
            write!(&mut res, "#{}: {:#x}\n", i, addr - 4).unwrap();
        }
        res
    }

    fn current_priv(&self) -> RISCV64Privilege {
        unsafe { *self.privilege.get() }
    }

    fn set_priv(&mut self, privilege: RISCV64Privilege) {
        unsafe {
            if *self.privilege.get() != privilege {
                *self.privilege.get() = privilege;
                self.set_interrupt_cond_dirty();
            }
        }
    }

    pub fn set_interrupt_cond_dirty(&mut self) {
        self.interrupt_cond_dirty = true;
    }
}

impl Isa for RISCV64 {
    fn new(
        stopped: Arc<AtomicBool>,
        memory: Memory,
        cpu_interrupt_bits: Arc<AtomicU64>,
        args: &Args,
    ) -> Self {
        // let reset_addr: PAddr = CONFIG_MBASE + CONFIG_PC_RESET_OFFSET;
        let reset_addr: PAddr = PAddr::new(CONFIG_MEM_BASE.value());
        // let reset_addr: PAddr = PAddr::new(CONFIG_FIRMWARE_BASE.value());
        let state = RISCV64CpuState::new(memory, &reset_addr, cpu_interrupt_bits);

        let mut sorted_patterns = Vec::new();
        for line in fs::read_to_string("./inst_perf.txt").unwrap().lines().rev() {
            let colon = line.find(":").unwrap();
            let name = line[..colon].trim();
            let pat = PATTERNS.iter().find(|&p| p._name == name).unwrap();
            sorted_patterns.push(pat);
        }

        Self {
            state,
            disassembler: LLVMDisassembler::new(
                "riscv64-unknown-linux-gnu",
                "rv64imafd_zicsr_zifencei",
            ),
            ibuf: SetAssociativeIBuf::new(),
            stop_at_ebreak: !args.ignore_isa_breakpoint,
            stopped,
            sorted_patterns,
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
        info!("priv: {:?}", self.state.current_priv());
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
        if self.state.stopping || self.stopped.load(Relaxed) {
            return false;
        }

        let inst = self.state.memory.ifetch(&self.state.pc, DWORD);
        match inst {
            Err(err) => {
                self.state.trap(err, Some(self.state.pc.value()));
            }
            Ok((inst, pc_paddr)) => {
                if inst == 0xff1ff06f && pc_paddr.value() == 0x80000050 {
                    // riscv-test fail
                    error!("riscv-test write-to-host triggered");
                    return false;
                }

                let (pattern, decode) = match self.ibuf.get(&pc_paddr, inst) {
                    Some(content) => content,
                    None => {
                        if inst == 0x0000006f {
                            error!("dead loop at pc {:#x}", self.state.pc.value());
                            self.state.regs[a0] = 1;
                            return false;
                        }
                        // decode exec
                        match self.sorted_patterns.iter().find(|&&p| p.match_inst(&inst)) {
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
                            Some(pat) => self.ibuf.set(&pc_paddr, inst, pat, pat.decode(&inst)),
                        }
                    }
                };
                pattern.exec(decode, &mut self.state);
            }
        }

        match &self.state.dyn_pc {
            Some(pc) => {
                if pc.value() == 0 {
                    warn!(
                        "Jump to address 0. Current pc vaddr: {:#x}",
                        self.state.pc.value()
                    );
                }
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

        while !self.stopped.load(Relaxed) {
            self.state.handle_interrupt();
            if let Some(pc) = &self.state.dyn_pc {
                // TODO: fix dup code
                if pc.value() == 0 {
                    warn!(
                        "Jump to address 0. Current pc vaddr: {:#x}",
                        self.state.pc.value()
                    );
                }
                if pc.value() == self.state.pc.value() {
                    panic!("deadloop at {:#x}", pc.value())
                }
                self.state.pc = *pc;
                self.state.dyn_pc = None;
            }
            if !self.state.wfi {
                break;
            }
            unsafe {
                *self.state.cycles.get() += 1;
            }
        }

        true
    }

    fn isa_get_exit_code(&self) -> u8 {
        self.state.regs[a0] as u8
    }

    fn isa_print_icache_info(&self) {
        cfg_if_feat!("log_inst", {
            println!("inst counter");
            let mut vec: Vec<(&'static str, u64)> = vec![];
            let mut total = 0u64;
            for (k, v) in self.state.inst_counter.iter() {
                unsafe {
                    vec.push((k.as_ref().unwrap()._name, *v));
                    total += *v;
                    // println!("{}: {}", k.as_ref().unwrap()._name, v);
                }
            }
            println!("{} insts in total", total);
            vec.sort_by(|a, b| a.1.cmp(&b.1));
            for (k, v) in vec {
                println!("{}: {} ({})", k, v, (v as f64) / (total as f64));
            }
            self.ibuf.print_info();

            self.state.memory.print_tlb_statistics();
        });
    }

    fn isa_disassemble_inst(&mut self, addr: &VAddr) -> String {
        let inst = self.state.memory.read(addr, DWORD);
        match inst {
            Ok(inst) => format!(
                "inst {:#x} at addr {:#x}\nDisassembled as {}",
                inst,
                addr.value(),
                self.disassembler
                    .disassemble(inst as u32, self.state.pc.value())
            ),
            Err(err) => format!("{:?} at addr {:#x}", err, addr.value()),
        }
    }

    fn read_vaddr(&mut self, addr: &VAddr, len: MemOperationSize) -> Result<u64, String> {
        self.state
            .memory
            .read(addr, len)
            .map_err(|e| format!("{:?}", e))
    }

    fn isa_get_backtrace(&self) -> String {
        self.state.get_backtrace_string()
    }

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
