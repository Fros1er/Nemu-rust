#![allow(non_snake_case)]

mod mie;
pub mod mstatus;
pub mod satp;

use crate::device::glob_timer;
use crate::isa::riscv64::csr::mstatus::MStatus;
use crate::isa::riscv64::csr::CSRAccessLevel::RW;
use crate::isa::riscv64::csr::CSRAccessLevel::{NotSupported, ROnly};
use crate::isa::riscv64::reg::Reg;
use crate::isa::riscv64::{RISCV64CpuState, RISCV64Privilege};
use log::{trace, warn};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::rc::Rc;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, FromRepr, IntoStaticStr};

#[derive(PartialEq)]
enum CSRAccessLevel {
    TODO,
    NotSupported,
    ROnly,
    RW,
}

struct CSRInfo {
    pub write_mask: u64,
    pub access_level: CSRAccessLevel, // illegal inst when write for readonly CSRs
}

impl CSRInfo {
    pub fn new(write_mask: u64, access_level: CSRAccessLevel) -> Self {
        Self {
            write_mask,
            access_level,
        }
    }
}

type WriteHook = fn(&Reg, &mut RISCV64CpuState);

pub struct CSRs {
    csrs: HashMap<u64, (Reg, CSRInfo)>,
    write_hooks: HashMap<u64, WriteHook>,
    time: Reg, // hack!
    cycles: Rc<UnsafeCell<u64>>,
} // name => (csr, write_mask)

trait CSR: Into<u64> {
    fn create() -> Self;
    fn info() -> CSRInfo {
        CSRInfo::new(0xdeadbeef, CSRAccessLevel::TODO)
    }
    fn name() -> CSRName;
}

pub struct CSROpResult {
    pub old: u64,
    new: u64,
    hook: Option<WriteHook>,
}

impl CSROpResult {
    pub fn new(old: u64, new: u64, hook: Option<&WriteHook>) -> Self {
        Self {
            old,
            new,
            hook: hook.copied(),
        }
    }
    pub fn call_hook(&self, state: &mut RISCV64CpuState) {
        if let Some(hook) = self.hook {
            hook(&self.new, state)
        }
    }
}

impl CSRs {
    pub fn new(cycles: Rc<UnsafeCell<u64>>) -> Self {
        #[allow(unused_mut)]
        let mut map: HashMap<u64, (u64, CSRInfo)> = HashMap::new();
        let mut write_hooks: HashMap<u64, WriteHook> = HashMap::new();
        macro_rules! insert_csr {
            // illegal inst when write
            ($name: expr, $val:expr, $mask:expr, $ronly:expr) => {
                map.insert($name as u64, ($val, CSRInfo::new($mask, $ronly)));
            };
        }
        macro_rules! insert_defined_csr {
            ($CSR:ty) => {
                map.insert(
                    <$CSR>::name() as u64,
                    (<$CSR>::create().into(), <$CSR>::info()),
                );
            };
        }
        macro_rules! insert_csr_hook {
            ($name:expr, $func:expr) => {
                write_hooks.insert($name as u64, $func);
            };
        }
        macro_rules! insert_rw_csr {
            ($name: expr, $val:expr) => {
                insert_csr!($name, $val, !0x0, RW);
            };
        }
        macro_rules! insert_ronly_csr {
            ($name: expr, $val:expr) => {
                insert_csr!($name, $val, 0x0, ROnly);
            };
        }

        macro_rules! map_s_csr {
            // s_csr is a subset of m_csr
            ($s_name: expr, $m_name: expr, $view_mask: expr, $ronly:expr) => {
                insert_csr!(
                    $s_name,
                    (map[&($m_name as u64)].0 & $view_mask) as u64,
                    $view_mask & map[&($m_name as u64)].1.write_mask,
                    $ronly
                );
                insert_csr_hook!($m_name, |csr, state| {
                    state.csrs.set_fast($s_name, *csr & $view_mask);
                });
                insert_csr_hook!($s_name, |csr, state| {
                    state
                        .csrs
                        .set_fast($m_name, csr | (state.csrs[$m_name] & !$view_mask));
                });
            };
        }

        insert_defined_csr!(satp::Satp);
        insert_csr_hook!(CSRName::satp, satp::Satp::write_hook);

        insert_defined_csr!(MStatus);
        const SSTATUS_VIEW_MASK: u64 =
            0b1000000000000000000000000000001100000001100011011110011101100010;
        map_s_csr!(CSRName::sstatus, CSRName::mstatus, SSTATUS_VIEW_MASK, RW);
        insert_csr_hook!(CSRName::mstatus, |csr, state| {
            state
                .csrs
                .set_fast(CSRName::sstatus, *csr & SSTATUS_VIEW_MASK);
            let mstatus: MStatus = (*csr).into();
            state.memory.update_priv(&mstatus);
        });

        insert_defined_csr!(mie::MIE);
        map_s_csr!(CSRName::sie, CSRName::mie, 0b10001000100010, RW);

        insert_defined_csr!(mie::MIP);
        map_s_csr!(CSRName::sip, CSRName::mip, 0b10001000100010, RW);

        insert_rw_csr!(CSRName::mtvec, 0);
        insert_rw_csr!(CSRName::stvec, 0);
        insert_rw_csr!(CSRName::mscratch, 0);
        insert_rw_csr!(CSRName::sscratch, 0);
        insert_rw_csr!(CSRName::mepc, 0);
        insert_rw_csr!(CSRName::sepc, 0);
        insert_rw_csr!(CSRName::mcause, 0);
        insert_rw_csr!(CSRName::scause, 0);
        insert_rw_csr!(CSRName::mtval, 0);
        insert_rw_csr!(CSRName::stval, 0);
        insert_rw_csr!(CSRName::mideleg, 0);
        insert_csr!(CSRName::medeleg, 0, !0b10000100000000000, RW);

        insert_ronly_csr!(CSRName::mvendorid, 0);
        insert_ronly_csr!(CSRName::marchid, 0);
        insert_ronly_csr!(CSRName::mimpid, 0);
        insert_ronly_csr!(CSRName::mhartid, 0);

        insert_csr!(
            CSRName::misa,
            // 0b10u64 << 62 | 0b101000001000100101001, // rv64imafd with U and S
            0b10u64 << 62 | 0b101000001000100000001, // rv64ima with U and S
            0x0,
            RW
        );
        insert_csr!(CSRName::pmpcfg0, 0, 0, RW);
        insert_csr!(CSRName::pmpaddr0, 0, 0, RW);

        insert_csr!(CSRName::mcounteren, 0, 0, RW); // TODO
        insert_csr!(CSRName::scounteren, 0, 0, RW);

        for i in 0..29 {
            insert_csr!(0xb03 + i, 0, 0, ROnly); // mhpmcounterx, not implemented(ronly).
        }
        for name in CSRNameNotImpl::iter() {
            insert_csr!(name, 0, 0, NotSupported);
        }

        // menvcfg has only FIOM bit implemented, as we don't have extensions.
        // As device memory access are always seq ordered, FIOM bit has no actual use.
        insert_csr!(CSRName::menvcfg, 0, 1, RW);

        // time read is explicitly handled
        insert_ronly_csr!(CSRName::time, 0);
        insert_ronly_csr!(CSRName::mcycle, 0);
        insert_ronly_csr!(CSRName::cycle, 0);

        Self {
            csrs: map,
            time: 0,
            cycles,
            write_hooks,
        }
    }

    fn check_idx(&self, idx: u64) {
        if !self.csrs.contains_key(&idx) {
            panic!("CSR not found: {:#x}", idx);
        }
    }

    #[inline]
    fn get_csr_mut(
        &mut self,
        idx: u64,
        check_ronly: bool,
    ) -> Result<(&mut Reg, &u64, Option<&WriteHook>), ()> {
        self.check_idx(idx);
        let (csr, info) = self.csrs.get_mut(&(idx)).ok_or(())?;
        match info.access_level {
            CSRAccessLevel::TODO => {
                panic!("CSR not implemented: {:#x}", idx)
            }
            NotSupported => {
                return Err(());
            }
            ROnly => {
                if check_ronly {
                    return Err(());
                }
            }
            RW => {}
        }

        let hook = self.write_hooks.get(&idx);

        const TIME_CSR_IDX: u64 = CSRName::time as u64;
        const MCYCLE_CSR_IDX: u64 = CSRName::mcycle as u64;
        const CYCLE_CSR_IDX: u64 = CSRName::cycle as u64;
        match idx {
            TIME_CSR_IDX => {
                self.time = glob_timer.lock().unwrap().since_boot_us();
                Ok((&mut self.time, &info.write_mask, hook))
            }
            MCYCLE_CSR_IDX | CYCLE_CSR_IDX => unsafe {
                Ok((&mut *self.cycles.get(), &info.write_mask, hook))
            },

            _ => Ok((csr, &info.write_mask, hook)),
        }
    }

    fn set_fast(&mut self, idx: CSRName, val: u64) {
        trace!(
            "set_fast csr {:?}: from {:#x} to {:#x}",
            idx,
            self[idx],
            val
        );
        self.csrs.get_mut(&(idx as u64)).unwrap().0 = val;
    }

    pub fn set_zero_fast(&mut self, idx: CSRName) {
        self.csrs.get_mut(&(idx as u64)).unwrap().0 = 0;
    }

    pub fn set_n(&mut self, idx: CSRName, val: u64) -> Result<CSROpResult, ()> {
        let (csr, mask, hook) = self.get_csr_mut(idx as u64, true)?;
        trace!(
            "set_n csr {:?}: from {:#x} to {:#x}, mask {:#x}, val {:#x}",
            idx,
            *csr,
            (val & *mask) | (*csr & !mask),
            mask,
            val
        );
        let res = *csr;
        *csr = (val & *mask) | (*csr & !mask);
        Ok(CSROpResult::new(res, *csr, hook))
    }

    pub fn set(&mut self, idx: u64, val: u64, _rs1_is_x0: bool) -> Result<CSROpResult, ()> {
        let (csr, mask, hook) = self.get_csr_mut(idx, true)?;
        trace!(
            "set csr {:?}: from {:#x} to {:#x}, mask {:#x}, val {:#x}",
            CSRs::get_csr_name(idx),
            *csr,
            (val & *mask) | (*csr & !mask),
            mask,
            val
        );
        let res = *csr;
        *csr = (val & *mask) | (*csr & !mask);
        Ok(CSROpResult::new(res, *csr, hook))
    }

    pub fn or(&mut self, idx: u64, val: u64, rs1_is_x0: bool) -> Result<CSROpResult, ()> {
        let (csr, mask, hook) = self.get_csr_mut(idx, !rs1_is_x0)?;
        trace!(
            "or csr {:?}: from {:#x} to {:#x}, mask {:#x}, val {:#x}",
            CSRs::get_csr_name(idx),
            *csr,
            *csr | (val & *mask),
            mask,
            val
        );
        let res = *csr;
        *csr |= val & *mask;
        Ok(CSROpResult::new(res, *csr, hook))
    }

    pub fn clear_bits(&mut self, idx: u64, val: u64, rs1_is_x0: bool) -> Result<CSROpResult, ()> {
        let (csr, mask, hook) = self.get_csr_mut(idx, !rs1_is_x0)?;
        trace!(
            "and csr {:?}: from {:#x} to {:#x}, mask {:#x}, val {:#x}",
            CSRs::get_csr_name(idx),
            *csr,
            *csr & !(val & mask),
            mask,
            val
        );
        let res = *csr;
        *csr &= !(val & mask);
        Ok(CSROpResult::new(res, *csr, hook))
    }

    pub fn check_privilege(idx: u64, privilege: RISCV64Privilege) -> bool {
        let u_ok = 0xC00 <= idx && idx <= 0xCFF;
        let s_ok = 0x100 <= idx && idx <= 0x1FF;
        let res = match privilege {
            RISCV64Privilege::M => true,
            RISCV64Privilege::S => s_ok || u_ok,
            RISCV64Privilege::U => u_ok,
        };
        if !res {
            warn!(
                "check csr priv failed: access {:?} at {:?}",
                CSRs::get_csr_name(idx),
                privilege
            );
        }
        res
    }

    fn get_csr_name(idx: u64) -> String {
        match CSRName::from_repr(idx as usize) {
            Some(csr) => format!("{:?}", csr),
            None => format!("{:#x}", idx),
        }
    }
}

impl Display for CSRs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for reg in CSRName::iter() {
            let name: &'static str = reg.into();
            s.push_str(format!("{}: {:#x}\n", name, self[reg]).as_str())
        }
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, IntoStaticStr, Copy, Clone, Debug)]
#[repr(u64)]
pub enum MCauseCode {
    None = 0,
    InstAccessFault = 1,
    IllegalInst = 2,
    Breakpoint = 3,
    LoadAccessFault = 5,
    StoreAMOMisaligned = 6,  // Store/AMO address misaligned
    StoreAMOAccessFault = 7, // Support misaligned access for store
    ECallU = 8,
    ECallS = 9,
    ECallM = 11,
    InstPageFault = 12,
    LoadPageFault = 13,
    StoreAMOPageFault = 15,
    DeadLoop = 128, // custom
    STimerInt = 0x8000000000000005,
    MTimerInt = 0x8000000000000007,
    SExtInt = 0x8000000000000009,
    MExtInt = 0x800000000000000b,
}

pub enum InterruptMask {
    // STimerInt = 1 << 5,
    MTimerInt = 1 << 7,
    SExtInt = 1 << 9,
    MExtInt = 1 << 11,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, EnumIter, EnumString, IntoStaticStr, FromRepr)]
pub enum CSRName {
    sstatus = 0x100,
    sie = 0x104,
    stvec = 0x105,
    scounteren = 0x106,
    sscratch = 0x140,
    sepc = 0x141,
    scause = 0x142,
    stval = 0x143,
    sip = 0x144,
    satp = 0x180,

    mcycle = 0xB00,
    mstatus = 0x300,
    misa = 0x301,
    medeleg = 0x302,
    mideleg = 0x303,
    mie = 0x304,
    mtvec = 0x305,
    mcounteren = 0x306,
    menvcfg = 0x30a,

    mscratch = 0x340,
    mepc = 0x341,
    mcause = 0x342,
    mtval = 0x343,
    mip = 0x344,

    pmpcfg0 = 0x3A0,
    // pmpcfg2 = 0x3A2,
    pmpaddr0 = 0x3B0,
    // pmpaddr1 = 0x3B1,
    // pmpaddr2 = 0x3B2,
    // pmpaddr3 = 0x3B3,
    cycle = 0xc00,
    time = 0xc01,

    mvendorid = 0xF11,
    marchid = 0xF12,
    mimpid = 0xF13,
    mhartid = 0xF14,
    // mnscratch = 0x740,
    // mnstatus = 0x744,
}

#[allow(non_camel_case_types)]
#[derive(EnumIter)]
pub enum CSRNameNotImpl {
    mcountinhibit = 0x320,
    scountovf = 0xda0,
    mtopi = 0xfb0,
    tselect = 0x7a0,
    tcontrol = 0x7a5,
}

impl Index<CSRName> for CSRs {
    type Output = Reg;

    fn index(&self, index: CSRName) -> &Self::Output {
        &self.csrs[&(index as u64)].0
    }
}
