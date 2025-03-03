use crate::memory::paddr::PAddr;

pub const CONFIG_MEM_SIZE: usize = 0x20000000;
pub const CONFIG_MEM_BASE: PAddr = PAddr::new(0x80000000);

pub const CONFIG_IMAGE_BASE: usize = 0x200000;
