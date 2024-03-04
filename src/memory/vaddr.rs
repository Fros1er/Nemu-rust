use crate::memory::paddr::PAddr;
use crate::memory::Memory;

#[derive(Copy, Clone)]
pub struct VAddr(u64);

#[derive(Copy, Clone)]
pub enum MemOperationSize {
    Byte = 1,
    WORD = 2,
    DWORD = 4,
    QWORD = 8,
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
        self.read(vaddr, len)
    }
    pub fn read(&self, vaddr: &VAddr, len: MemOperationSize) -> u64 {
        let paddr: PAddr = vaddr.into();
        match len {
            MemOperationSize::Byte => self.read_p::<u8>(&paddr) as u64,
            MemOperationSize::WORD => self.read_p::<u16>(&paddr) as u64,
            MemOperationSize::DWORD => self.read_p::<u32>(&paddr) as u64,
            MemOperationSize::QWORD => self.read_p::<u64>(&paddr),
        }
    }

    pub fn write(&mut self, vaddr: &VAddr, data: u64, len: MemOperationSize) {
        let paddr: PAddr = vaddr.into();
        match len {
            MemOperationSize::Byte => self.write_p(&paddr, data as u8),
            MemOperationSize::WORD => self.write_p(&paddr, data as u16),
            MemOperationSize::DWORD => self.write_p(&paddr, data as u32),
            MemOperationSize::QWORD => self.write_p(&paddr, data),
        };
    }
}
