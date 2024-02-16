use crate::isa::riscv64::inst::InstType::{R, I, S, B, U, J};
use crate::isa::riscv64::reg::{Reg, RegName};
use crate::isa::riscv64::RISCV64CpuState;
use crate::memory::vaddr::MemOperationSize::Byte;
use crate::memory::vaddr::VAddr;

enum InstType {
    R,
    I,
    S,
    B,
    U,
    J,
}

pub struct Pattern {
    mask: u64,
    key: u64,
    inst_type: InstType,
    name: &'static str,
    op: fn(Decode, &mut RISCV64CpuState),
}

macro_rules! bits {
    ($var:expr, $hi:literal, $lo:literal) => {
        ((($var) >> ($lo)) & ((1u64 << (($hi) - ($lo) + 1)) - 1))
    };
}

#[inline]
pub fn sign_extend64(data: u64, size: usize) -> u64 {
    assert!(size > 0 && size <= 32);
    (((data << (64 - size)) as i64) >> (64 - size)) as u64
}

impl Pattern {
    pub fn match_inst(&self, inst: &u64) -> bool {
        (inst & self.mask) == self.key
    }
    fn decode(&self, inst: &u64) -> Decode {
        let mut rd = 0;
        let mut rs1 = 0;
        let mut rs2 = 0;
        let mut imm = 0;
        match self.inst_type {
            R => {
                rd = bits!(inst, 11, 7);
                rs1 = bits!(inst, 19, 15);
                rs2 = bits!(inst, 24, 20);
            }
            I => {
                rd = bits!(inst, 11, 7);
                rs1 = bits!(inst, 19, 15);
                imm = sign_extend64(bits!(inst, 31, 20), 12);
            }
            S => {
                rs1 = bits!(inst, 19, 15);
                rs2 = bits!(inst, 24, 20);
                imm = (sign_extend64(bits!(inst, 31, 25), 7) << 5) | bits!(inst, 11, 7);
            }
            B => {
                rs1 = bits!(inst, 19, 15);
                rs2 = bits!(inst, 24, 20);
            }
            U => {
                rd = bits!(inst, 11, 7);
                imm = sign_extend64(bits!(inst, 31, 12), 20) << 12;
            }
            J => {
                rd = bits!(inst, 11, 7);
            }
        }
        Decode {
            rd,
            rs1,
            rs2,
            imm,
        }
    }

    pub fn exec(&self, inst: &u64, state: &mut RISCV64CpuState) {
        let decode = self.decode(inst);
        (self.op)(decode, state);
    }
}

struct Decode {
    rd: u64,
    rs1: u64,
    rs2: u64,
    imm: u64,
}

impl Decode {
    fn src1(&self, state: &RISCV64CpuState) -> u64 {
        state.regs[self.rs1 as u8]
    }

    fn src2(&self, state: &RISCV64CpuState) -> u64 {
        state.regs[self.rs2 as u8]
    }
}

fn make_pattern(pat: &str, inst_type: InstType, name: &'static str, op: fn(Decode, &mut RISCV64CpuState)) -> Pattern {
    let pat = "??????? ????? ????? ??? ????? 00101 11";
    let mut mask: u64 = 0;
    let mut key: u64 = 0;
    let mut cnt = 0;
    for c in pat.chars() {
        match c {
            '1' => {
                mask = (mask << 1) | 1;
                key = (key << 1) | 1;
            }
            '0' => {
                mask = (mask << 1) | 1;
                key <<= 1;
            }
            '?' => {}
            ' ' => continue,
            _ => panic!("Bad pattern {}", pat)
        }
        cnt += 1;
        if cnt > 32 {
            panic!("Bad pattern {} with len {}", pat, cnt);
        }
    }
    if cnt < 32 {
        panic!("Bad pattern {} with len {}", pat, cnt);
    }

    Pattern {
        mask,
        key,
        inst_type,
        name,
        op,
    }
}

pub fn init_patterns() -> Vec<Pattern> {
    vec![
        make_pattern("??????? ????? ????? ??? ????? 00101 11", U, "auipc", |inst, state| {
            state.regs[inst.rd as u8] = (state.pc.value() as u64 + inst.imm) as Reg;
        }),
        make_pattern("??????? ????? ????? 100 ????? 00000 11", I, "lbu", |inst, state| {
            state.regs[inst.rd as u8] = VAddr::new((inst.imm + inst.src1(state)) as usize).read(Byte);
        }),
        make_pattern("??????? ????? ????? 000 ????? 01000 11", S, "sb", |inst, state| {
            VAddr::new((inst.imm + inst.src1(state)) as usize).write(inst.rs2 as u64, Byte);
        }),
    ]
    //
    // make_pattern("0000000 00001 00000 000 00000 11100 11", "ebreak", |inst, state| {
    //     NEMUTRAP(s->pc, R(10))
    // }); // R(10) is $a0
    // make_pattern("??????? ????? ????? ??? ????? ????? ??", "inv", |inst, state| {
    //     INV(s->pc)
    // });
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let x = 0b111_10100_1111111;
//         println!("{:b}", bits!(x, 6, 0));
//     }
// }