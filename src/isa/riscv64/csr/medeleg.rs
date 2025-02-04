use crate::isa::riscv64::csr::{CSRName, CSR};

pub struct MeDeleg(u64);
pub struct MiDeleg(u64);

impl Into<u64> for MeDeleg {
    fn into(self) -> u64 {
        self.0
    }
}

impl CSR for MeDeleg {
    fn create() -> Self {
        Self(0)
    }

    fn name() -> CSRName {
        CSRName::medeleg
    }
}

impl Into<u64> for MiDeleg {
    fn into(self) -> u64 {
        self.0
    }
}

impl CSR for MiDeleg {
    fn create() -> Self {
        Self(0)
    }

    fn name() -> CSRName {
        CSRName::mideleg
    }
}