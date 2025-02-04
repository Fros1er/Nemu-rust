#![allow(non_snake_case)]

mod medeleg;
mod mie;
mod mstatus;
mod satp;

use crate::isa::riscv64::reg::Reg;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[derive(Debug)]
pub struct CSRs(HashMap<u64, (Reg, u64)>); // name => (csr, write_mask)

trait CSR: Into<u64> {
    fn create() -> Self;
    fn write_mask() -> u64 {
        0x0
    }
    fn name() -> CSRName;
}

impl CSRs {
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut map = HashMap::new();
        macro_rules! insert_csr {
            ($CSR:ty) => {
                map.insert(
                    <$CSR>::name() as u64,
                    (<$CSR>::create().into(), <$CSR>::write_mask()),
                );
            };
        }
        macro_rules! insert_zero_csr {
            ($name: expr, $val:expr) => {
                map.insert($name as u64, ($val, !0x0));
            };
        }
        insert_csr!(satp::Satp);
        insert_csr!(mstatus::MStatus);
        insert_csr!(medeleg::MeDeleg);
        insert_csr!(medeleg::MiDeleg);
        insert_csr!(mie::MIE);
        insert_csr!(mie::MIP);
        insert_zero_csr!(CSRName::mtvec, 0);
        insert_zero_csr!(CSRName::mscratch, 0);
        insert_zero_csr!(CSRName::mepc, 0);
        insert_zero_csr!(CSRName::mcause, 0);
        map.insert(CSRName::mhartid as u64, (0, 0x1)); // TODO: 0x0
        Self(map)
    }

    fn check_idx(&self, idx: u64) {
        if !self.0.contains_key(&idx) {
            panic!("CSR not found: {:#x}", idx);
        }
        if self.0.get(&idx).unwrap().1 == 0 {
            panic!("CSR not implemented (write_mask == 0): {:#x}", idx);
        }
    }

    pub fn set_n(&mut self, idx: CSRName, val: u64) -> u64 {
        let (csr, mask) = self.0.get_mut(&(idx as u64)).unwrap();
        *csr = val & *mask;
        *csr
    }

    pub fn set(&mut self, idx: u64, val: u64) -> u64 {
        self.check_idx(idx);
        let (csr, mask) = self.0.get_mut(&idx).unwrap();
        let res = *csr;
        *csr = val & *mask;
        res
    }

    pub fn or(&mut self, idx: u64, val: u64) -> u64 {
        self.check_idx(idx);
        let (csr, mask) = self.0.get_mut(&idx).unwrap();
        let res = *csr;
        *csr |= val & *mask;
        res
    }

    pub fn and(&mut self, idx: u64, val: u64) -> u64 {
        self.check_idx(idx);
        let (csr, mask) = self.0.get_mut(&idx).unwrap();
        let res = *csr;
        *csr &= val | !*mask;
        res
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
    StoreAMOMisaligned = 6, // Store/AMO address misaligned
    StoreAMOAccessFault = 7, // Support misaligned access for store
    ECallM = 11,
    DeadLoop = 128, // custom
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, EnumIter, EnumString, IntoStaticStr)]
pub enum CSRName {
    satp = 0x180,
    mstatus = 0x300,
    medeleg = 0x302,
    mideleg = 0x303,
    mie = 0x304,
    mtvec = 0x305,

    mscratch = 0x340,
    mepc = 0x341,
    mcause = 0x342,
    mip = 0x344,

    // pmpcfg0 = 0x3A0,
    // pmpcfg2 = 0x3A2,
    // pmpaddr0 = 0x3B0,
    // pmpaddr1 = 0x3B1,
    // pmpaddr2 = 0x3B2,
    // pmpaddr3 = 0x3B3,

    mhartid = 0xF14,
    // mnscratch = 0x740,
    // mnstatus = 0x744,
}

impl Index<u64> for CSRs {
    type Output = Reg;

    fn index(&self, index: u64) -> &Self::Output {
        if !self.0.contains_key(&index) {
            panic!("CSR not found: {:#x}", index);
        }
        &self.0[&index].0
    }
}

impl Index<CSRName> for CSRs {
    type Output = Reg;

    fn index(&self, index: CSRName) -> &Self::Output {
        &self.0[&(index as u64)].0
    }
}
