use crate::memory::paddr::PAddr;

pub const CONFIG_MSIZE: u64 = 0x20000000;
pub const CONFIG_MBASE: PAddr = PAddr::new(0x80000000);
pub const CONFIG_PC_RESET_OFFSET: u64 = 0;

