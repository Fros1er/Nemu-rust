use crate::isa::riscv64::csr::{CSRName, CSR};

pub struct Satp(u64);

impl Into<u64> for Satp {
    fn into(self) -> u64 {
        self.0
    }
}

impl CSR for Satp {
    fn create() -> Self {
        Self(0)
    }

    fn name() -> CSRName {
        CSRName::satp
    }
}
