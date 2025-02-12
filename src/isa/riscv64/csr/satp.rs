use crate::isa::riscv64::csr::CSRAccessLevel::RW;
use crate::isa::riscv64::csr::{CSRInfo, CSRName, CSR};
use crate::isa::riscv64::reg::Reg;
use crate::isa::riscv64::RISCV64CpuState;
use bitfield_struct::bitfield;
use strum_macros::FromRepr;

#[derive(FromRepr, PartialEq)]
pub enum SATPMode {
    Bare = 0,
    Sv39 = 8,
}

#[bitfield(u64)]
pub struct Satp {
    #[bits(44)]
    pub ppn: u64,
    #[bits(16)]
    pub asid: usize,
    #[bits(4)]
    pub mode: usize,
}

impl Satp {
    pub fn write_hook(csr: &Reg, state: &mut RISCV64CpuState) {
        let satp: Satp = (*csr).into();
        state.memory.update_translation_ctrl(&satp);
    }
}

impl CSR for Satp {
    fn create() -> Self {
        Self(0)
    }

    fn info() -> CSRInfo {
        CSRInfo::new(!(0xffff << 44), RW) // asid not implemented
    }

    fn name() -> CSRName {
        CSRName::satp
    }
}
