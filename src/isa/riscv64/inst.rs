use crate::isa::riscv64::inst::InstType::{B, I, J, R, S, U};
use crate::isa::riscv64::reg::Reg;
use crate::isa::riscv64::RISCV64CpuState;
use crate::memory::vaddr::MemOperationSize::{Byte, QWORD};
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
    _name: &'static str,
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
                imm = (bits!(inst, 31, 31) << 20) | (bits!(inst, 19, 12) << 12) | (bits!(inst, 20, 20) << 11) | (bits!(inst, 30, 21) << 1);
                imm = sign_extend64(imm, 21);
            }
        }
        Decode { rd, rs1, rs2, imm }
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

fn make_pattern(
    pat: &str,
    inst_type: InstType,
    name: &'static str,
    op: fn(Decode, &mut RISCV64CpuState),
) -> Pattern {
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
            '?' => {
                mask <<= 1;
                key <<= 1;
            }
            ' ' => continue,
            _ => panic!("Bad pattern {}", pat),
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
        _name: name,
        op,
    }
}

pub fn init_patterns() -> Vec<Pattern> {
    vec![
        make_pattern(
            "??????? ????? ????? ??? ????? 00101 11",
            U,
            "auipc",
            |inst, state| {
                state.regs[inst.rd as u8] = (state.pc.value() as u64 + inst.imm) as Reg;
            },
        ),
        make_pattern(
            "??????? ????? ????? 100 ????? 00000 11",
            I,
            "lbu",
            |inst, state| {
                state.regs[inst.rd as u8] = state
                    .memory
                    .borrow()
                    .read(&VAddr::new((inst.imm + inst.src1(state)) as usize), Byte);
            },
        ),
        make_pattern(
            "??????? ????? ????? 000 ????? 01000 11",
            S,
            "sb",
            |inst, state| {
                state.memory.borrow_mut().write(
                    &VAddr::new((inst.imm + inst.src1(state)) as usize),
                    inst.rs2,
                    Byte,
                );
            },
        ),
        make_pattern(
            "??????? ????? ????? 011 ????? 01000 11",
            S,
            "sd",
            |inst, state| {
                state.memory.borrow_mut().write(
                    &VAddr::new((inst.imm + inst.src1(state)) as usize),
                    inst.rs2,
                    QWORD,
                );
            },
        ),
        make_pattern(
            "0000000 00001 00000 000 00000 11100 11",
            I,
            "ebreak",
            |_inst, state| state.trap(),
        ),
        make_pattern(
            "??????? ????? ????? 000 ????? 0010011",
            I,
            "addi",
            |inst, state| {
                println!("{:#x} + {:#x}", inst.src1(state) as i64, inst.imm as i64);
                state.regs[inst.rd as u8] = (inst.src1(state) as i64 + inst.imm as i64) as u64;
            },
        ),
        make_pattern(
            "??????? ????? ????? ??? ????? 1101111",
            J,
            "jal",
            |inst, state| {
                state.regs[inst.rd as u8] = (state.pc.value() + 4) as u64;
                state.dyn_pc = Some(VAddr::new(state.pc.value().wrapping_add(inst.imm as usize)));
            },
        ),
        make_pattern(
            "??????? ????? ????? 000 ????? 1100111",
            I,
            "jalr",
            |inst, state| {
                state.regs[inst.rd as u8] = (state.pc.value() + 4) as u64;
                state.dyn_pc = Some(VAddr::new(inst.src1(state).wrapping_add(inst.imm) as usize));
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use crate::isa::riscv64::inst::InstType::{I, S, J};
    use crate::isa::riscv64::inst::make_pattern;
    use crate::memory::vaddr::MemOperationSize::Byte;
    use crate::memory::vaddr::VAddr;

    #[test]
    fn decode_j_test() {
        let pat = make_pattern(
            "??????? ????? ????? ??? ????? 1101111",
            J,
            "jal",
            |inst, state| {},
        );
        let res = pat.decode(&0xc000efu64);
        println!("{:#x}", res.imm);
        let res = pat.decode(&0b011111111111_00001_1101111u64);
        println!("{:#x}", res.imm);
    }

    // #[test]
    // fn it_works() {
    //     let pat = make_pattern("??????? ????? ????? 100 ????? 00000 11", I, "lbu", |inst, state| {
    //         state.regs[inst.rd as u8] = state.memory.borrow().read(&VAddr::new((inst.imm + inst.src1(state)) as usize), Byte);
    //     });
    //     println!("mask:{:x} key:{:x}", pat.mask, pat.key);
    //     assert!(pat.match_inst(&0x0102c503u64));
    // }
}
