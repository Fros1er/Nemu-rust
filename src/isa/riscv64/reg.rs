use std::ops::{Index, IndexMut};
use strum_macros::EnumIter; // 0.17.1

pub type Reg = u64;

pub struct Registers([Reg; 32]);

impl Registers {
    pub(crate) fn new() -> Self {
        Self([0; 32])
    }
}


pub struct CSR([Reg; 2]);

impl CSR {
    pub fn new() -> Self {
        Self([0; 2])
    }
}

pub enum MCauseCode {
    Breakpoint = 3
}


#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, EnumIter)]
pub enum RegName {
    zero,
    ra,
    sp,
    gp,
    tp,
    t0,
    t1,
    t2,
    s0,
    s1,
    a0,
    a1,
    a2,
    a3,
    a4,
    a5,
    a6,
    a7,
    s2,
    s3,
    s4,
    s5,
    s6,
    s7,
    s8,
    s9,
    s10,
    s11,
    t3,
    t4,
    t5,
    t6,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, EnumIter)]
pub enum CSRName {
    mepc,
    mcause,
}

impl Index<u8> for Registers {
    type Output = Reg;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl Index<RegName> for Registers {
    type Output = Reg;

    fn index(&self, index: RegName) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Index<u8> for CSR {
    type Output = Reg;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl Index<CSRName> for CSR {
    type Output = Reg;

    fn index(&self, index: CSRName) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for CSR {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl IndexMut<CSRName> for CSR {
    fn index_mut(&mut self, index: CSRName) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}