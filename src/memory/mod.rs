use crate::memory::paddr::init_mem;
use crate::utils::configs::CONFIG_MSIZE;

pub mod paddr;
pub mod vaddr;

pub struct Memory {
    pub pmem: Box<[u8]>
}

impl Memory {
    pub fn new() -> Self {
        init_mem();
        Self {
            pmem: vec![0u8; CONFIG_MSIZE].into_boxed_slice()
        }
    }
}
