use crate::memory::paddr::PAddr;
use crate::memory::Memory;

#[derive(Copy, Clone)]
pub struct VAddr(u64);

#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum MemOperationSize {
    Byte = 1,
    WORD = 2,
    DWORD = 4,
    QWORD = 8,
}

impl MemOperationSize {
    pub fn read_sized(&self, dst: *const u8) -> u64 {
        match self {
            MemOperationSize::Byte => unsafe { dst.read() as u64 },
            MemOperationSize::WORD => unsafe { (dst as *const u16).read() as u64 },
            MemOperationSize::DWORD => unsafe { (dst as *const u32).read() as u64 },
            MemOperationSize::QWORD => unsafe { (dst as *const u64).read() },
        }
    }
    pub fn write_sized(&self, data: u64, dst: *mut u8) {
        match self {
            MemOperationSize::Byte => unsafe { dst.write(data as u8) },
            MemOperationSize::WORD => unsafe { (dst as *mut u16).write(data as u16) },
            MemOperationSize::DWORD => unsafe { (dst as *mut u32).write(data as u32) },
            MemOperationSize::QWORD => unsafe { (dst as *mut u64).write(data) },
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

pub struct MMU {
    mem: Memory,
}

impl MMU {
    pub fn new(mem: Memory) -> Self {
        Self { mem }
    }

    pub fn read_if_tmp(&self, vaddr: &VAddr, len: MemOperationSize) -> Option<u64> {
        self.mem.read(&vaddr.into(), len)
    }

    pub fn read(&self, vaddr: &VAddr, len: MemOperationSize) -> Option<u64> {
        // info!("READ {:#x}", vaddr.value());
        self.mem.read(&vaddr.into(), len)
    }

    pub fn write(&mut self, vaddr: &VAddr, data: u64, len: MemOperationSize) -> Result<(), ()> {
        self.mem.write(&vaddr.into(), data, len)
    }

    pub fn is_aligned(&self, vaddr: &VAddr, len: MemOperationSize) -> bool {
        vaddr.value() % (len as u64) == 0
    }
}
