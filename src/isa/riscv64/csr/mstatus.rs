use crate::isa::riscv64::csr::{CSRName, CSR};
use bitfield_struct::bitfield;

#[bitfield(u64)]
pub struct MStatus {
    _1: bool,
    SIE: bool, // IRQ: S mode interrupt enable
    _2: bool,
    MIE: bool, // IRQ: M mode interrupt enable
    _3: bool,
    SPIE: bool, // IRQ: When a trap is taken from privilege mode y into privilege mode x, xPIE is set to the value of xIE; xIE is set to 0; and xPP is set to y.
    UBE: bool, // ENDIAN: 0
    MPIE: bool, // IRQ
    SPP: bool, // IRQ
    #[bits(2)]
    VS: usize, // 0
    #[bits(2)]
    MPP: usize, // IRQ
    #[bits(2)]
    FS: usize, // 0
    #[bits(2)]
    XS: usize, // 0
    MPRV: bool, // MMU: Enable MMU even in M Mode
    SUM: bool, // MMU: S-mode memory accesses to pages that are accessible by U-mode is permitted
    MXR: bool, // MMU: loads from pages marked executable will succeed.
    TVM: bool, // VIRT: attempts to read or write the satp CSR or execute an SFENCE.VMA or SINVAL.VMA instruction while executing in S-mode will raise an illegal-instruction exception.
    TW: bool, // for WFI
    TSR: bool, // VIRT: illegal-instruction when sret in S mode
    #[bits(9)]
    _4: usize,
    #[bits(2)]
    UXL: usize, // 0b10
    #[bits(2)]
    SXL: usize, // 0b10
    SBE: bool, // ENDIAN: 0
    MBE: bool, // ENDIAN: 0
    #[bits(25)]
    _5: usize,
    SD: bool, // 0
}

impl CSR for MStatus {
    fn create() -> Self {
        // TODO
        Self(0xa00001800) // MPP=3, UXL=SXL=2
    }

    fn write_mask() -> u64 {
        0b11111100001100110101010
    }

    fn name() -> CSRName {
        CSRName::mstatus
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::riscv64::csr::CSR;
    use crate::isa::riscv64::csr::mstatus::MStatus;

    #[test]
    fn it_works() {
        let tt: MStatus = MStatus::create();
        println!("{:?}", tt);
        let t = MStatus::new().with_SIE(true);
        println!("{:#x}", t.0);
    }
}
