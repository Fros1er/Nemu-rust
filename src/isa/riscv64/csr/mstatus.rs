use crate::isa::riscv64::csr::CSRAccessLevel::RW;
use crate::isa::riscv64::csr::{CSRInfo, CSRName, CSR};
use crate::isa::riscv64::RISCV64Privilege;
use bitfield_struct::bitfield;

#[bitfield(u64)]
pub struct MStatus {
    _1: bool,
    pub SIE: bool, // IRQ: S mode interrupt enable
    _2: bool,
    pub MIE: bool, // IRQ: M mode interrupt enable
    _3: bool,
    SPIE: bool, // IRQ: When a trap is taken from privilege mode y into privilege mode x, xPIE is set to the value of xIE; xIE is set to 0; and xPP is set to y.
    UBE: bool,  // ENDIAN: 0
    MPIE: bool, // IRQ
    SPP: bool,  // IRQ
    #[bits(2)]
    VS: usize, // 0
    #[bits(2)]
    MPP: usize, // IRQ
    #[bits(2)]
    FS: usize, // 0
    #[bits(2)]
    XS: usize, // 0
    pub MPRV: bool, // MMU: Enable MMU even in M Mode
    pub SUM: bool, // MMU: S-mode memory accesses to pages that are accessible by U-mode is permitted
    pub MXR: bool, // MMU: loads from pages marked executable will succeed.
    TVM: bool, // VIRT: attempts to read or write the satp CSR or execute an SFENCE.VMA or SINVAL.VMA instruction while executing in S-mode will raise an illegal-instruction exception.
    TW: bool,  // for WFI
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

impl MStatus {
    // IRQ: When a trap is taken from privilege mode y into privilege mode x,
    // xPIE is set to the value of xIE; xIE is set to 0; and xPP is set to y.
    pub fn update_when_trap(&mut self, from: RISCV64Privilege, to: RISCV64Privilege) {
        if to == RISCV64Privilege::M {
            self.set_MPIE(self.MIE());
            self.set_MIE(false);
            self.set_MPP(from as usize);
        } else {
            assert_ne!(from, RISCV64Privilege::M);
            self.set_SPIE(self.SIE());
            self.set_SIE(false);
            self.set_SPP(from == RISCV64Privilege::S);
        }
    }

    /// returns next priv
    pub fn update_when_ret(&mut self, ret_inst: RISCV64Privilege) -> RISCV64Privilege {
        // When executing an xRET instruction, supposing xPP holds the value y, xIE is set to xPIE; the privilege mode
        // is changed to y; xPIE is set to 1; and xPP is set to the least-privileged supported mode (U if U-mode is
        // implemented, else M). If y≠M, xRET also sets MPRV=0.
        let to;
        if ret_inst == RISCV64Privilege::M {
            self.set_MIE(self.MPIE());
            self.set_MPIE(true);
            to = self.MPP();
            self.set_MPP(RISCV64Privilege::U as usize);
        } else {
            self.set_SIE(self.SPIE());
            self.set_SPIE(true);
            to = self.SPP() as usize;
            self.set_SPP(false); // Priv::U
        }
        let to: RISCV64Privilege = RISCV64Privilege::from_repr(to).unwrap();

        if to != RISCV64Privilege::M {
            self.set_MPRV(false);
        }
        to
    }
}

impl CSR for MStatus {
    fn create() -> Self {
        // TODO
        Self(0xa00001800) // MPP=3, UXL=SXL=2
    }

    fn info() -> CSRInfo {
        CSRInfo::new(0b11111100001100110101010, RW)
    }

    fn name() -> CSRName {
        CSRName::mstatus
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::riscv64::csr::mstatus::MStatus;
    use crate::isa::riscv64::csr::CSR;

    #[test]
    fn it_works() {
        println!("{:#x}", 0b10u64 << 62 | 0b101000001000100101001);
        let tt: MStatus = MStatus::create();
        println!("{:?}", tt);
        let t = MStatus::new().with_SIE(true);
        println!("{:#x}", t.0);
    }
}
