use crate::memory::Memory;
use crate::memory::paddr::PAddr;

#[derive(Copy, Clone)]
pub struct VAddr(u64);

#[derive(Copy, Clone)]
pub enum MemOperationSize {
    Byte = 1,
    WORD = 2,
    DWORD = 4,
    QWORD = 8,
}

impl MemOperationSize {
    pub fn read_sized(&self, dst: *const u8) -> u64 {
        match self {
            MemOperationSize::Byte => unsafe { dst.read() as u64 }
            MemOperationSize::WORD => unsafe { (dst as *const u16).read() as u64 }
            MemOperationSize::DWORD => unsafe { (dst as *const u32).read() as u64 }
            MemOperationSize::QWORD => unsafe { (dst as *const u64).read() }
        }
    }
    pub fn write_sized(&self, data: u64, dst: *mut u8) {
        match self {
            MemOperationSize::Byte => unsafe { dst.write(data as u8) }
            MemOperationSize::WORD => unsafe { (dst as *mut u16).write(data as u16) }
            MemOperationSize::DWORD => unsafe { (dst as *mut u32).write(data as u32) }
            MemOperationSize::QWORD => unsafe { (dst as *mut u64).write(data) }
        }
    }
}

impl VAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }
    pub fn value(&self) -> u64 {
        self.0
    }
    pub fn inc(&mut self, len: MemOperationSize) {
        self.0 += len as u64;
    }
}

impl From<PAddr> for VAddr {
    fn from(value: PAddr) -> Self {
        Self(value.value())
    }
}

impl Memory {
    pub fn ifetch(&self, vaddr: &VAddr, len: MemOperationSize) -> u64 {
        self.read_mem_unchecked_p(&vaddr.into(), len)
    }
    pub fn read(&self, vaddr: &VAddr, len: MemOperationSize) -> u64 {
        self.read_p(&vaddr.into(), len)
    }

    pub fn write(&mut self, vaddr: &VAddr, data: u64, len: MemOperationSize) {
        self.write_p(&vaddr.into(), data, len)
    }
}
