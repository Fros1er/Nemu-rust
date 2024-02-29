use crate::memory::Memory;
use crate::memory::vaddr::{MemOperationSize, VAddr};

pub fn ifetch(pc: &mut VAddr, memory: &Memory, len: MemOperationSize) -> u64 {
    let inst = memory.ifetch(pc, len);
    // pc.inc(len);
    inst
}