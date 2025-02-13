use crate::isa::riscv64::csr::satp::{SATPMode, Satp};
use crate::memory::paddr::PAddr;
use crate::memory::Memory;

#[derive(Copy, Clone)]
pub struct VAddr(u64);

#[derive(Copy, Clone, PartialEq)]
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

struct SV39 {
    lvl1_base: u64,
}

pub struct TranslationCtrl {
    pub is_bare: bool,
    sv39: SV39,
}

pub(crate) struct MMU {
    mem: Memory,
    translation_ctrl: TranslationCtrl,
}

impl MMU {
    pub fn new(mem: Memory) -> Self {
        Self {
            mem,
            translation_ctrl: TranslationCtrl::new(),
        }
    }

    pub fn paddr_to_vaddr(&self, paddr: &PAddr) -> VAddr {
        assert!(self.translation_ctrl.is_bare);
        VAddr::new(paddr.value())
    }

    pub fn read(&self, vaddr: &VAddr, len: MemOperationSize) -> Option<u64> {
        assert!(self.translation_ctrl.is_bare);
        self.mem.read(&vaddr.into(), len)
    }

    pub fn write(&mut self, vaddr: &VAddr, data: u64, len: MemOperationSize) -> Result<(), ()> {
        assert!(self.translation_ctrl.is_bare);
        self.mem.write(&vaddr.into(), data, len)
    }

    pub fn is_aligned(&self, vaddr: &VAddr, len: MemOperationSize) -> bool {
        vaddr.value() % (len as u64) == 0
    }

    pub fn update_translation_ctrl(&mut self, satp: &Satp) {
        let mode = SATPMode::from_repr(satp.mode());
        match mode {
            None => panic!("Unsupported SATP mode: {}", satp.mode()),
            Some(mode) => {
                self.translation_ctrl.is_bare = if mode == SATPMode::Bare { true } else { false };
            }
        }
        self.translation_ctrl.sv39.lvl1_base = satp.ppn() << 12;
    }
}

impl TranslationCtrl {
    pub fn new() -> Self {
        Self {
            is_bare: true,
            sv39: SV39::new(),
        }
    }
}

impl SV39 {
    pub fn new() -> Self {
        Self { lvl1_base: 0 }
    }
}
