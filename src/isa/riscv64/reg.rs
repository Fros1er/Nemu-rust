use std::ops::{Index, IndexMut};

pub type Reg = u64;

pub struct Registers([Reg; 32]);

impl Registers {
    pub(crate) fn new() -> Registers {
        Registers([0; 32])
    }
}

#[allow(non_camel_case_types)]
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

// #[repr(C)]
// pub struct Registers {
//     zero: Reg,
//     ra: Reg,
//     sp: Reg,
//     gp: Reg,
//     tp: Reg,
//     t0: Reg,
//     t1: Reg,
//     t2: Reg,
//     s0: Reg,
//     s1: Reg,
//     a0: Reg,
//     a1: Reg,
//     a2: Reg,
//     a3: Reg,
//     a4: Reg,
//     a5: Reg,
//     a6: Reg,
//     a7: Reg,
//     s2: Reg,
//     s3: Reg,
//     s4: Reg,
//     s5: Reg,
//     s6: Reg,
//     s7: Reg,
//     s8: Reg,
//     s9: Reg,
//     s10: Reg,
//     s11: Reg,
//     t3: Reg,
//     t4: Reg,
//     t5: Reg,
//     t6: Reg,
// }
//
// impl Registers {
//     pub(crate) fn new() -> Registers {
//         Registers {
//             zero: 0,
//             ra: 0,
//             sp: 0,
//             gp: 0,
//             tp: 0,
//             t0: 0,
//             t1: 0,
//             t2: 0,
//             s0: 0,
//             s1: 0,
//             a0: 0,
//             a1: 0,
//             a2: 0,
//             a3: 0,
//             a4: 0,
//             a5: 0,
//             a6: 0,
//             a7: 0,
//             s2: 0,
//             s3: 0,
//             s4: 0,
//             s5: 0,
//             s6: 0,
//             s7: 0,
//             s8: 0,
//             s9: 0,
//             s10: 0,
//             s11: 0,
//             t3: 0,
//             t4: 0,
//             t5: 0,
//             t6: 0,
//         }
//     }
// }
//
// impl Index<usize> for Registers {
//     type Output = Reg;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         unsafe {
//             let ptr = addr_of!(self) as *const Reg;
//             &*ptr.offset(index as isize)
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::isa::riscv64::reg::Registers;
//
//     #[test]
//     fn it_works() {
//         let mut regs = Registers::new();
//         regs.t1 = 10;
//         println!("{}", regs[6]);
//     }
// }