use crate::memory::vaddr::{MemOperationSize, VAddr};

fn ifetch(pc: &mut VAddr, len: MemOperationSize) {
    let inst = pc.ifetch(len);
    pc.inc(len);
}