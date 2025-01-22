use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

// 0.17.1

pub type Reg = u64;

pub struct Registers(pub(crate) [Reg; 33]);

impl Registers {
    pub(crate) fn new() -> Self {
        Self([0; 33])
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for reg in RegName::iter() {
            let name: &'static str = reg.into();
            s.push_str(format!("{}: {:#x}\n", name, self[reg]).as_str())
        }
        write!(f, "{}", s)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, EnumIter, EnumString, IntoStaticStr)]
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
    fake_zero,
}

pub fn format_regs(regs: &[u64], pc: u64) -> String {
    let mut res = String::new();
    for i in 0..32 {
        let reg_str: &str = RegName::iter().nth(i).unwrap().into();
        res = format!("{}{}: {:#x}\n", res, reg_str, regs[i]);
    }
    res = format!("{}pc: {:#x}\n", res, pc);
    res
}

impl Index<u64> for Registers {
    type Output = Reg;

    fn index(&self, index: u64) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl Index<RegName> for Registers {
    type Output = Reg;

    fn index(&self, index: RegName) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl IndexMut<u64> for Registers {
    fn index_mut(&mut self, index: u64) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}

impl IndexMut<RegName> for Registers {
    fn index_mut(&mut self, index: RegName) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}


