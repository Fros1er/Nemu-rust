use crate::memory::paddr::PAddr;

pub const CONFIG_MEM_SIZE: u64 = 0x20000000;
pub const CONFIG_MEM_BASE: PAddr = PAddr::new(0x80000000);

pub const CONFIG_FIRMWARE_SIZE: u64 = 0x90000;
pub const CONFIG_FIRMWARE_BASE: PAddr = PAddr::new(0x10000);
