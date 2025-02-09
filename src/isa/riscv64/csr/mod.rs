#![allow(non_snake_case)]

mod medeleg;
mod mie;
mod mstatus;
mod satp;

use crate::device::glob_timer;
use crate::isa::riscv64::csr::CSRAccessLevel::RW;
use crate::isa::riscv64::csr::CSRAccessLevel::{NotSupported, ROnly};
use crate::isa::riscv64::reg::Reg;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

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

pub struct CSRs {
    csrs: HashMap<u64, (Reg, CSRInfo)>,
    time: Reg, // hack!
} // name => (csr, write_mask)

trait CSR: Into<u64> {
    fn create() -> Self;
    fn info() -> CSRInfo {
        CSRInfo::new(0xdeadbeef, CSRAccessLevel::TODO)
    }
    fn name() -> CSRName;
}

impl CSRs {
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut map = HashMap::new();
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

        insert_defined_csr!(satp::Satp);
        insert_defined_csr!(mstatus::MStatus);
        insert_defined_csr!(medeleg::MeDeleg);
        insert_defined_csr!(medeleg::MiDeleg);
        insert_defined_csr!(mie::MIE);
        insert_defined_csr!(mie::MIP);
        insert_rw_csr!(CSRName::mtvec, 0);
        insert_rw_csr!(CSRName::mscratch, 0);
        insert_rw_csr!(CSRName::mepc, 0);
        insert_rw_csr!(CSRName::mcause, 0);
        insert_rw_csr!(CSRName::mtval, 0);

        insert_ronly_csr!(CSRName::mhartid, 0);
        insert_csr!(
            CSRName::misa,
            0b10u64 << 62 | 0b101000001000100101001,
            0x0,
            RW
        ); // rv64imafd with U and S
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

        Self { csrs: map, time: 0 }
    }

    fn check_idx(&self, idx: u64) {
        if !self.csrs.contains_key(&idx) {
            panic!("CSR not found: {:#x}", idx);
        }
    }

    #[inline]
    fn get_csr_mut(&mut self, idx: u64, check_ronly: bool) -> Option<(&mut Reg, &u64)> {
        self.check_idx(idx);
        let (csr, info) = self.csrs.get_mut(&(idx)).unwrap();
        match info.access_level {
            CSRAccessLevel::TODO => {
                panic!("CSR not implemented: {:#x}", idx)
            }
            NotSupported => {
                return None;
            }
            ROnly => {
                if check_ronly {
                    return None;
                }
            }
            RW => {}
        }

        if idx == CSRName::time as u64 {
            self.time = glob_timer.since_boot_us();
            return Some((&mut self.time, &info.write_mask));
        }

        Some((csr, &info.write_mask))
    }

    pub fn set_n(&mut self, idx: CSRName, val: u64) -> Option<u64> {
        let (csr, mask) = self.get_csr_mut(idx as u64, true)?;
        *csr = val & *mask;
        Some(*csr)
    }

    pub fn set(&mut self, idx: u64, val: u64, _rs1_is_x0: bool) -> Option<u64> {
        let (csr, mask) = self.get_csr_mut(idx, true)?;
        let res = *csr;
        *csr = val & *mask;
        Some(res)
    }

    pub fn or(&mut self, idx: u64, val: u64, rs1_is_x0: bool) -> Option<u64> {
        let (csr, mask) = self.get_csr_mut(idx, !rs1_is_x0)?;
        let res = *csr;
        *csr |= val & *mask;
        Some(res)
    }

    pub fn and(&mut self, idx: u64, val: u64, rs1_is_x0: bool) -> Option<u64> {
        let (csr, mask) = self.get_csr_mut(idx, !rs1_is_x0)?;
        let res = *csr;
        *csr &= val | !*mask;
        Some(res)
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

#[derive(PartialEq, IntoStaticStr)]
pub enum MCauseCode {
    InstAccessFault = 1,
    IllegalInst = 2,
    Breakpoint = 3,
    LoadAccessFault = 5,
    StoreAMOMisaligned = 6,  // Store/AMO address misaligned
    StoreAMOAccessFault = 7, // Support misaligned access for store
    ECallM = 11,
    DeadLoop = 128, // custom
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, EnumIter, EnumString, IntoStaticStr)]
pub enum CSRName {
    scounteren = 0x106,

    satp = 0x180,
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
    time = 0xc01,

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
}

// impl Index<u64> for CSRs {
//     type Output = Reg;
//
//     fn index(&self, index: u64) -> &Self::Output {
//         if !self.0.contains_key(&index) {
//             panic!("CSR not found: {:#x}", index);
//         }
//         &self.0[&index].0
//     }
// }
//
impl Index<CSRName> for CSRs {
    type Output = Reg;

    fn index(&self, index: CSRName) -> &Self::Output {
        &self.csrs[&(index as u64)].0
    }
}
