use crate::memory::paddr::{PAddr, PAddrDiff};

pub const CONFIG_MSIZE: usize = 0x8000000;
pub const CONFIG_MBASE: PAddr = PAddr::new(0x80000000);
pub const CONFIG_PC_RESET_OFFSET: PAddrDiff = PAddrDiff::new(0);
