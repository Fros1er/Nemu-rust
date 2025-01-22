#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};
use bitfield_struct::bitfield;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};
use crate::isa::riscv64::reg::Reg;

#[derive(Debug)]
pub struct CSR(HashMap<u64, Reg>);

impl CSR {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        for name in CSRName::iter() {
            map.insert(name as u64, 0u64);
        }
        map.insert(CSRName::mstatus as u64, 0xa00001800);
        Self(map)
    }
}

impl Display for CSR {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for reg in CSRName::iter() {
            let name: &'static str = reg.into();
            s.push_str(format!("{}: {:#x}\n", name, self[reg]).as_str())
        }
        write!(f, "{}", s)
    }
}

pub enum MCauseCode {
    Breakpoint = 3,
    ECallM = 11,
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

    pmpcfg0 = 0x3A0,
    pmpaddr0 = 0x3B0,

    mhartid = 0xF14,
    mnscratch = 0x740,
    mnstatus = 0x744,
}

impl Index<u64> for CSR {
    type Output = Reg;

    fn index(&self, index: u64) -> &Self::Output {
        if !self.0.contains_key(&index) {
            panic!("CSR not found: {:#x}", index);
        }
        &self.0[&index]
    }
}


impl Index<CSRName> for CSR {
    type Output = Reg;

    fn index(&self, index: CSRName) -> &Self::Output {
        &self.0[&(index as u64)]
    }
}

impl IndexMut<u64> for CSR {
    fn index_mut(&mut self, index: u64) -> &mut Self::Output {
        if !self.0.contains_key(&index) {
            panic!("CSR not found: {:#x}", index);
        }
        self.0.get_mut(&(index)).unwrap()
    }
}

impl IndexMut<CSRName> for CSR {
    fn index_mut(&mut self, index: CSRName) -> &mut Self::Output {
        self.0.get_mut(&(index as u64)).unwrap()
    }
}


#[bitfield(u64)]
struct MStatus {
    _1: bool,
    SIE: bool,
    _2: bool,
    MIE: bool,
    _3: bool,
    SPIE: bool,
    UBE: bool,
    MPIE: bool,
    SPP: bool,
    #[bits(2)]
    VS: usize,
    #[bits(2)]
    MPP: usize,
    #[bits(2)]
    FS: usize,
    #[bits(2)]
    XS: usize,
    MPRV: bool,
    SUM: bool,
    MXR: bool,
    TVM: bool,
    TW: bool,
    TSR: bool,
    #[bits(9)]
    _4: usize,
    #[bits(2)]
    UXL: usize,
    #[bits(2)]
    SXL: usize,
    SBE: bool,
    MBE: bool,
    #[bits(25)]
    _5: usize,
    SD: bool,
}

#[cfg(test)]
mod tests {
    use std::mem::transmute;
    use crate::isa::riscv64::csr::MStatus;

    #[test]
    fn it_works() {
        let i = 0xa00001800u64;
        let tt: MStatus = unsafe { transmute(i) };
        println!("{:?}", tt);
        let t = MStatus::new().with_SIE(true);
        println!("{:#x}", t.0);
    }
}