use crate::isa::riscv64::csr::CSRAccessLevel::RW;
use crate::isa::riscv64::csr::{CSRInfo, CSRName, CSR};
use bitfield_struct::bitfield;

#[bitfield(u64)]
pub struct MIP {
    // TODO: Interrupt handling
    _1: bool,
    SSIP: bool,
    _2: bool,
    MSIP: bool,
    _3: bool,
    STIP: bool,
    _4: bool,
    MTIP: bool,
    _5: bool,
    SEIP: bool,
    _6: bool,
    MEIP: bool,
    _7: bool,
    LCOFIP: bool,
    #[bits(50)]
    _8: usize,
}
#[bitfield(u64)]
pub struct MIE {
    // TODO: Interrupt handling
    _1: bool,
    SSIE: bool,
    _2: bool,
    MSIE: bool,
    _3: bool,
    STIE: bool,
    _4: bool,
    MTIE: bool,
    _5: bool,
    SEIE: bool,
    _6: bool,
    MEIE: bool,
    _7: bool,
    LCOFIE: bool,
    #[bits(50)]
    _8: usize,
}

impl CSR for MIP {
    fn create() -> Self {
        Self::new()
    }

    fn info() -> CSRInfo {
        CSRInfo::new(0b10101010101010, RW)
    }

    fn name() -> CSRName {
        CSRName::mip
    }
}

impl CSR for MIE {
    fn create() -> Self {
        Self::new()
    }

    fn info() -> CSRInfo {
        CSRInfo::new(0b10101010101010, RW)
    }

    fn name() -> CSRName {
        CSRName::mie
    }
}
