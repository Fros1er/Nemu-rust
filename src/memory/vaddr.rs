use crate::memory::paddr::PAddr;

pub struct VAddr(usize);

#[derive(Copy, Clone)]
pub enum MemOperationSize {
    Byte = 1,
    WORD = 2,
    DWORD = 4,
    QWORD = 8,
}

impl VAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }
    pub fn ifetch(&self, len: MemOperationSize) -> u64 {
        self.read(len)
    }
    pub fn read(&self, len: MemOperationSize) -> u64 {
        let paddr: PAddr = self.into();
        match len {
            MemOperationSize::Byte => paddr.read::<u8>() as u64,
            MemOperationSize::WORD => paddr.read::<u16>() as u64,
            MemOperationSize::DWORD => paddr.read::<u32>() as u64,
            MemOperationSize::QWORD => paddr.read::<u64>(),
        }
    }
    pub fn write(&self, data: u64, len: MemOperationSize) {
        let paddr: PAddr = self.into();
        match len {
            MemOperationSize::Byte => paddr.write(data as u8),
            MemOperationSize::WORD => paddr.write(data as u16),
            MemOperationSize::DWORD => paddr.write(data as u32),
            MemOperationSize::QWORD => paddr.write(data),
        };
    }
    pub fn value(&self) -> usize {
        self.0
    }
    pub fn inc(&mut self, len: MemOperationSize) {
        self.0 += len as usize;
    }
}

impl From<PAddr> for VAddr {
    fn from(value: PAddr) -> Self {
        Self(value.value())
    }
}