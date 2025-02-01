#![allow(unused_imports)]

use lazy_static::lazy_static;
use log::info;
use crate::isa::riscv64::csr::MCauseCode;
use crate::isa::riscv64::inst::InstType::{B, I, J, R, S, U, Zicsr};
use crate::isa::riscv64::reg::{Reg, RegName};
use crate::isa::riscv64::RISCV64CpuState;
use crate::memory::vaddr::MemOperationSize::{Byte, DWORD, QWORD, WORD};
use crate::memory::vaddr::VAddr;

enum InstType {
    R,
    I,
    S,
    B,
    U,
    J,
    Zicsr,
}

pub struct Pattern {
    mask: u64,
    key: u64,
    inst_type: InstType,
    pub _name: &'static str,
    op: fn(&Decode, &mut RISCV64CpuState),
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
    pub fn decode(&self, inst: &u64) -> Decode {
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
                imm = (bits!(inst, 31, 31) << 12) | (bits!(inst, 7, 7) << 11) | (bits!(inst, 30, 25) << 5) | (bits!(inst, 11, 8) << 1);
                imm = sign_extend64(imm, 13);
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
            Zicsr => {
                rd = bits!(inst, 11, 7);
                rs1 = bits!(inst, 19, 15);
                imm = bits!(inst, 31, 20);
            }
        }
        if rd == 0 {
            rd = RegName::fake_zero as u64;
        }
        Decode { rd, rs1, rs2, imm }
    }

    pub fn exec(&self, decode: &Decode, state: &mut RISCV64CpuState) {
        (self.op)(decode, state);
    }
}

#[derive(Default, Clone)]
pub struct Decode {
    rd: u64,
    rs1: u64,
    rs2: u64,
    imm: u64,
}

impl Decode {
    fn src1(&self, state: &RISCV64CpuState) -> u64 {
        state.regs[self.rs1]
    }
    fn src1_i64(&self, state: &RISCV64CpuState) -> i64 {
        state.regs[self.rs1] as i64
    }

    fn src1_u32(&self, state: &RISCV64CpuState) -> u32 {
        state.regs[self.rs1] as u32
    }
    fn src1_i32(&self, state: &RISCV64CpuState) -> i32 {
        state.regs[self.rs1] as i32
    }

    fn src1_u32_signext64(&self, state: &RISCV64CpuState) -> i64 {
        (state.regs[self.rs1] as i32) as i64
    }
    fn src1_trunc32(&self, state: &RISCV64CpuState) -> u64 {
        (state.regs[self.rs1] as u32) as u64
    }

    fn src2(&self, state: &RISCV64CpuState) -> u64 {
        state.regs[self.rs2]
    }
    fn src2_i64(&self, state: &RISCV64CpuState) -> i64 {
        state.regs[self.rs2] as i64
    }
    fn src2_u32(&self, state: &RISCV64CpuState) -> u32 {
        state.regs[self.rs2] as u32
    }
    fn src2_i32(&self, state: &RISCV64CpuState) -> i32 {
        state.regs[self.rs2] as i32
    }
}

fn make_pattern(
    pat: &'static str,
    inst_type: InstType,
    name: &'static str,
    op: fn(&Decode, &mut RISCV64CpuState),
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

#[inline]
fn sign_ext_32to64(src: u64) -> u64 {
    ((src as i32) as i64) as u64
}

macro_rules! gen_load_u {
    ($size:expr) => {
        |inst, state| {
            state.regs[inst.rd] = state
                .memory
                .borrow()
                .read(&VAddr::new(inst.imm.wrapping_add(inst.src1(state))), $size)
        }
    };
}

macro_rules! gen_load {
    ($size:expr) => {
        |inst, state| {
            state.regs[inst.rd] = sign_extend64(
                state.memory.borrow()
                .read(&VAddr::new(inst.imm.wrapping_add(inst.src1(state))), $size), 8 * $size as usize);
        }
    };
}

macro_rules! gen_store {
    ($size:expr) => {
        |inst, state| {
            state.memory.borrow_mut().write(
                    &VAddr::new(inst.imm.wrapping_add(inst.src1(state))),
                    inst.src2(state),
                    $size,
                );
        }
    };
}

macro_rules! gen_bit_op {
    ($op: tt) => {
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state) $op inst.src2(state)
        }
    };
}

macro_rules! gen_bit_op_imm {
    ($op: tt) => {
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state) $op inst.imm;
        }
    };
}

macro_rules! gen_branch {
    ($op: tt) => {
        |inst, state| {
            if (inst.src1(state) as i64) $op (inst.src2(state) as i64) {
                state.dyn_pc = Some(VAddr::new(state.pc.value().wrapping_add(inst.imm)));
            }
        }
    };
}

macro_rules! gen_branch_u {
    ($op: tt) => {
        |inst, state| {
            if inst.src1(state) $op inst.src2(state) {
                state.dyn_pc = Some(VAddr::new(state.pc.value().wrapping_add(inst.imm)));
            }
        }
    };
}

macro_rules! gen_arithmetic {
    ($op: tt) => {
        |inst, state| {
            state.regs[inst.rd] = (inst.src1_i64(state).$op(inst.src2_i64(state))) as u64;
        }
    };
}

macro_rules! gen_arithmetic_u {
    ($op: tt) => {
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state).$op(inst.src2(state));
        }
    };
}

macro_rules! gen_arithmetic_w {
    ($op: tt) => {
        |inst, state| {
            state.regs[inst.rd] = sign_ext_32to64(inst.src1_i32(state).$op(inst.src2_i32(state)) as u64);
        }
    };
}


macro_rules! gen_arithmetic_uw {
    ($op: tt) => {
        |inst, state| {
            state.regs[inst.rd] = sign_ext_32to64(inst.src1_u32(state).$op(inst.src2_u32(state)) as u64);
        }
    };
}


lazy_static! {
pub static ref PATTERNS: [Pattern; 67] = [
    // memory
    make_pattern("??????? ????? ????? 000 ????? 0000011", I, "lb", gen_load!(Byte)),
    make_pattern("??????? ????? ????? 100 ????? 0000011", I, "lbu", gen_load_u!(Byte)),
    make_pattern("??????? ????? ????? 001 ????? 0000011", I, "lh", gen_load!(WORD)),
    make_pattern("??????? ????? ????? 101 ????? 0000011", I, "lhu", gen_load_u!(WORD)),
    make_pattern("??????? ????? ????? 010 ????? 0000011", I, "lw", gen_load!(DWORD)),
    make_pattern("??????? ????? ????? 110 ????? 0000011", I, "lwu", gen_load_u!(DWORD)),
    make_pattern("??????? ????? ????? 011 ????? 0000011", I, "ld", gen_load_u!(QWORD)),
    make_pattern("??????? ????? ????? 000 ????? 0100011", S, "sb", gen_store!(Byte)),
    make_pattern("??????? ????? ????? 001 ????? 0100011", S, "sh", gen_store!(WORD)),
    make_pattern("??????? ????? ????? 010 ????? 0100011", S, "sw", gen_store!(DWORD)),
    make_pattern("??????? ????? ????? 011 ????? 0100011", S, "sd", gen_store!(QWORD)),
    make_pattern(
        "??????? ????? ????? ??? ????? 0110111", U, "lui",
        |inst, state| {
            state.regs[inst.rd] = inst.imm;
        },
    ),

    // arithmetic
    make_pattern(
        "??????? ????? ????? 000 ????? 0010011", I, "addi",
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state).wrapping_add(inst.imm);
        },
    ),
    make_pattern(
        "??????? ????? ????? 000 ????? 0011011", I, "addiw",
        |inst, state| {
            state.regs[inst.rd] = sign_ext_32to64(inst.src1_i32(state).wrapping_add(inst.imm as i32) as u64);
        },
    ),
    make_pattern("0000000 ????? ????? 000 ????? 0110011", R, "add", gen_arithmetic!(wrapping_add)),
    make_pattern("0100000 ????? ????? 000 ????? 0110011", R, "sub", gen_arithmetic!(wrapping_sub)),
    make_pattern("0000001 ????? ????? 000 ????? 0110011", R, "mul", gen_arithmetic!(wrapping_mul)),
    make_pattern("0000001 ????? ????? 100 ????? 0110011", R, "div", gen_arithmetic!(wrapping_div)),
    make_pattern("0000001 ????? ????? 101 ????? 0110011", R, "divu", gen_arithmetic_u!(wrapping_div)),
    make_pattern("0000001 ????? ????? 110 ????? 0110011", R, "rem", gen_arithmetic!(wrapping_rem)),
    make_pattern("0000001 ????? ????? 111 ????? 0110011", R, "remu", gen_arithmetic_u!(wrapping_rem)),
    make_pattern("0000000 ????? ????? 000 ????? 0111011", R, "addw", gen_arithmetic_w!(wrapping_add)),
    make_pattern("0100000 ????? ????? 000 ????? 0111011", R, "subw", gen_arithmetic_w!(wrapping_sub)),
    make_pattern("0000001 ????? ????? 000 ????? 0111011", R, "mulw", gen_arithmetic_w!(wrapping_mul)),
    make_pattern("0000001 ????? ????? 100 ????? 0111011", R, "divw", gen_arithmetic_w!(wrapping_div)),
    make_pattern("0000001 ????? ????? 101 ????? 0111011", R, "divuw", gen_arithmetic_uw!(wrapping_div)),
    make_pattern("0000001 ????? ????? 110 ????? 0111011", R, "remw", gen_arithmetic_w!(wrapping_rem)),
    make_pattern("0000001 ????? ????? 111 ????? 0111011", R, "remuw", gen_arithmetic_uw!(wrapping_rem)),

    // bit op
    make_pattern("0000000 ????? ????? 111 ????? 0110011", R, "and", gen_bit_op!(&)),
    make_pattern("0000000 ????? ????? 110 ????? 0110011", R, "or", gen_bit_op!(|)),
    make_pattern("0000000 ????? ????? 100 ????? 0110011", R, "xor", gen_bit_op!(^)),
    make_pattern("??????? ????? ????? 111 ????? 0010011", I, "andi", gen_bit_op_imm!(&)),
    make_pattern("??????? ????? ????? 110 ????? 0010011", I, "ori", gen_bit_op_imm!(|)),
    make_pattern("??????? ????? ????? 100 ????? 0010011", I, "xori", gen_bit_op_imm!(^)),
    // TODO: revisit w insts, some truncate32 to reg still needed!

    make_pattern(
        "0000000 ????? ????? 001 ????? 0110011", R, "sll",
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state) << (inst.src2(state) & 0b111111);
        },
    ),
    make_pattern(
        "0000000 ????? ????? 101 ????? 0110011", R, "srl",
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state) >> (inst.src2(state) & 0b111111);
        },
    ),
    make_pattern(
        "000000 ?????? ????? 001 ????? 0010011", I, "slli",
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state) << (inst.imm & 0b111111);
        },
    ),
    make_pattern(
        "000000 ?????? ????? 101 ????? 0010011", I, "srli",
        |inst, state| {
            state.regs[inst.rd] = inst.src1(state) >> (inst.imm & 0b111111);
        },
    ),
    make_pattern(
        "010000 ?????? ????? 101 ????? 0010011", I, "srai",
        |inst, state| {
            state.regs[inst.rd] = (inst.src1(state) as i64 >> (inst.imm & 0b111111)) as u64;
        },
    ),
    make_pattern(
        "0000000 ????? ????? 001 ????? 0111011", R, "sllw",
        |inst, state| {
            state.regs[inst.rd] = sign_ext_32to64(inst.src1_trunc32(state) << (inst.src2(state) & 0b11111));
        },
    ),
    make_pattern(
        "0000000 ????? ????? 101 ????? 0111011", R, "srlw",
        |inst, state| {
            state.regs[inst.rd] = sign_ext_32to64(inst.src1_trunc32(state) >> (inst.src2(state) & 0b11111));
        },
    ),
    make_pattern(
        "0100000 ????? ????? 101 ????? 0111011", R, "sraw",
        |inst, state| {
            state.regs[inst.rd] = (inst.src1_u32_signext64(state) >> (inst.src2(state) & 0b11111)) as u64;
        },
    ),
    make_pattern(
        "0000000 ????? ????? 001 ????? 0011011", I, "slliw",
        |inst, state| {
            state.regs[inst.rd] = sign_ext_32to64(inst.src1_trunc32(state) << (inst.imm & 0b11111));
        },
    ),
    make_pattern(
        "0000000 ????? ????? 101 ????? 0011011", I, "srliw",
        |inst, state| {
            state.regs[inst.rd] = sign_ext_32to64(inst.src1_trunc32(state) >> (inst.imm & 0b11111));
        },
    ),
    make_pattern(
        "010000 ?????? ????? 101 ????? 0011011", I, "sraiw",
        |inst, state| {
            state.regs[inst.rd] = (inst.src1_u32_signext64(state) >> (inst.imm & 0b11111)) as u64;
        },
    ),
    // branch
    make_pattern("??????? ????? ????? 000 ????? 1100011", B, "beq", gen_branch_u!(==)),
    make_pattern("??????? ????? ????? 001 ????? 1100011", B, "bne", gen_branch_u!(!=)),
    make_pattern("??????? ????? ????? 101 ????? 1100011", B, "bge", gen_branch!(>=)),
    make_pattern("??????? ????? ????? 111 ????? 1100011", B, "bgeu", gen_branch_u!(>=)),
    make_pattern("??????? ????? ????? 100 ????? 1100011", B, "blt", gen_branch!(<)),
    make_pattern("??????? ????? ????? 110 ????? 1100011", B, "bltu", gen_branch_u!(<)),
    make_pattern(
        "??????? ????? ????? ??? ????? 1101111",
        J,
        "jal",
        |inst, state| {
            state.regs[inst.rd] = state.pc.value() + 4;
            state.dyn_pc = Some(VAddr::new(state.pc.value().wrapping_add(inst.imm)));
            if inst.rd == RegName::ra as u64 {
                state.backtrace.push(state.pc.value() + 4);
                // info!("call {:#x}", state.dyn_pc.unwrap().value());
            }
        },
    ),
    make_pattern(
        "??????? ????? ????? 000 ????? 1100111",
        I,
        "jalr",
        |inst, state| {
            state.dyn_pc = Some(VAddr::new(inst.src1(state).wrapping_add(inst.imm)));
            state.regs[inst.rd] = state.pc.value() + 4;
            if inst.rs1 == RegName::ra as u64 && inst.rd == RegName::fake_zero as u64 {
                state.backtrace.pop();
                // info!("return to {:#x}", state.dyn_pc.unwrap().value());
            }
            if inst.rd == RegName::ra as u64 {
                state.backtrace.push(state.pc.value() + 4);
                // info!("call {:#x}", state.dyn_pc.unwrap().value());
            }

            // if let Some(addr) = state.backtrace.last() {
            //     if state.dyn_pc.unwrap().value() == *addr {
            //         state.backtrace.pop();
            //     }
            // }
            //
        },
    ),

    // set
    make_pattern(
        "0000000 ????? ????? 010 ????? 0110011", R, "slt",
        |inst, state| {
            state.regs[inst.rd] = if inst.src1_i64(state) < inst.src2_i64(state) { 1 } else { 0 }
        },
    ),
    make_pattern(
        "0000000 ????? ????? 011 ????? 0110011", R, "sltu",
        |inst, state| {
            state.regs[inst.rd] = if inst.src1(state) < inst.src2(state) { 1 } else { 0 }
        },
    ),
    make_pattern(
        "??????? ????? ????? 010 ????? 0010011", I, "slti",
        |inst, state| {
            state.regs[inst.rd] = if inst.src1_i64(state) < inst.imm as i64 { 1 } else { 0 }
        },
    ),
    make_pattern(
        "??????? ????? ????? 011 ????? 0010011", I, "sltiu",
        |inst, state| {
            state.regs[inst.rd] = if inst.src1(state) < inst.imm { 1 } else { 0 }
        },
    ),
    // Zicsr
    make_pattern(
        "??????? ????? ????? 001 ????? 1110011", Zicsr, "csrrw",
        |inst, state| {
            if inst.rd != RegName::zero as u64 {
                state.regs[inst.rd] = state.csrs[inst.imm];
            }
            state.csrs[inst.imm] = state.regs[inst.rs1];
        },
    ),
    make_pattern(
        "??????? ????? ????? 010 ????? 1110011", Zicsr, "csrrs",
        |inst, state| {
            if inst.rd != RegName::zero as u64 {
                state.regs[inst.rd] = state.csrs[inst.imm];
            }
            state.csrs[inst.imm] |= state.regs[inst.rs1];
        },
    ),
    make_pattern(
        "??????? ????? ????? 011 ????? 1110011", Zicsr, "csrrc",
        |inst, state| {
            if inst.rd != RegName::zero as u64 {
                state.regs[inst.rd] = state.csrs[inst.imm];
            }
            state.csrs[inst.imm] &= !state.regs[inst.rs1];
        },
    ),
    make_pattern(
        "??????? ????? ????? 101 ????? 1110011", Zicsr, "csrrwi",
        |inst, state| {
            if inst.rd != RegName::zero as u64 {
                state.regs[inst.rd] = state.csrs[inst.imm];
            }
            state.csrs[inst.imm] = inst.rs1;
        },
    ),

    // misc
    make_pattern(
        "??????? ????? ????? ??? ????? 0010111", U, "auipc",
        |inst, state| {
            state.regs[inst.rd] = (state.pc.value() + inst.imm) as Reg;
        },
    ),
    make_pattern(
        "0000000 00001 00000 000 00000 1110011", I, "ebreak",
        |_inst, state| state.trap(MCauseCode::Breakpoint),
    ),
    make_pattern(
        "0000000 00000 00000 000 00000 1110011", I, "ecall",
        |_inst, state| state.trap(MCauseCode::ECallM),
    ),
    make_pattern(
        "0001000 00010 00000 000 00000 1110011", I, "sret",
        |_inst, state| state.ret()
    ),
    make_pattern(
        "0011000 00010 00000 000 00000 1110011", I, "mret",
        |_inst, state| state.ret()
    ),
    make_pattern(
        "??????? ????? ????? 000 ????? 0001111", I, "fence",
        |_inst, _state| {}
    ),
];
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::isa::riscv64::inst::{make_pattern, Pattern, PATTERNS};
    use crate::isa::riscv64::inst::InstType::J;

    #[test]
    fn decode_j_test() {
        let pat = make_pattern(
            "??????? ????? ????? ??? ????? 1101111",
            J,
            "jal",
            |_inst, _state| {},
        );
        let res = pat.decode(&0xc000efu64);
        println!("{:#x}", res.imm);
        let res = pat.decode(&0b011111111111_00001_1101111u64);
        println!("{:#x}", res.imm);
    }

    #[test]
    fn it_works() {
        // let pat = make_pattern("??????? ????? ????? 100 ????? 00000 11", I, "lbu", |inst, state| {
        //     state.regs[inst.rd] = state.memory.borrow().read(&VAddr::new((inst.imm + inst.src1(state))), Byte);
        // });
        // println!("mask:{:x} key:{:x}", pat.mask, pat.key);
        // assert!(pat.match_inst(&0x0102c503u64));
        // println!("{:#x}", truncate_32(0xffffffffffff));
        // println!("{}", test!(^));
        // println!("{}", test!(+));
        let pat = &PATTERNS;
        let mut pat_map = HashMap::<&str, &Pattern>::new();
        for p in pat as &[Pattern; 67] {
            if p.match_inst(&0x03079793u64) {
                println!("{}", p._name);
            }
            pat_map.insert(p._name, p);
        }
        // pat_map["srliw"].match_inst(&0x0017d69bu64);
        pat_map["slli"].match_inst(&0x3079793u64);
        // if p.match_inst(&0x0017d69bu64) {
        //     println!("{}", p._name);
        // }
    }
}
