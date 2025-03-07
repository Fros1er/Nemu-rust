#![allow(non_snake_case)]

use crate::isa::riscv64::csr::mstatus::MStatus;
use crate::isa::riscv64::csr::satp::{SATPMode, Satp};
use crate::isa::riscv64::csr::MCauseCode;
use crate::isa::riscv64::csr::MCauseCode::{
    InstAccessFault, InstPageFault, LoadAccessFault, LoadPageFault, StoreAMOAccessFault,
    StoreAMOPageFault,
};
use crate::isa::riscv64::vaddr::TranslationErr::{AccessFault, PageFault};
use crate::isa::riscv64::RISCV64Privilege;
use crate::memory::paddr::PAddr;
use crate::memory::Memory;
use bitfield_struct::bitfield;
use log::{debug, trace, warn};
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::rc::Rc;

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
    pub fn read_val(&self, src: u64) -> u64 {
        match self {
            MemOperationSize::Byte => src as u8 as u64,
            MemOperationSize::WORD => src as u16 as u64,
            MemOperationSize::DWORD => src as u32 as u64,
            MemOperationSize::QWORD => src,
        }
    }
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

    pub fn bitor_sized(&self, data: u64, dst: *mut u8) {
        match self {
            MemOperationSize::Byte => unsafe { *dst |= data as u8 },
            MemOperationSize::WORD => unsafe { *(dst as *mut u16) |= data as u16 },
            MemOperationSize::DWORD => unsafe { *(dst as *mut u32) |= data as u32 },
            MemOperationSize::QWORD => unsafe { *(dst as *mut u64) |= data },
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
    privilege: Rc<RefCell<RISCV64Privilege>>,
    SUM: bool,
    MXR: bool,
    translate_in_m: bool,
}

pub(crate) struct MMU {
    mem: Memory,
    translation_ctrl: TranslationCtrl,
}

#[bitfield(u64)]
pub struct SV39PTE {
    valid: bool,
    R: bool,
    W: bool,
    X: bool,
    U: bool,
    G: bool,
    A: bool,
    D: bool,
    #[bits(2)]
    RSW: u64,
    // #[bits(9)]
    // PPN0: u64,
    // #[bits(9)]
    // PPN1: u64,
    // #[bits(26)]
    // PPN2: u64,
    #[bits(44)]
    PPN: u64,
    #[bits(10)]
    _8: usize,
}

impl SV39PTE {
    pub fn is_next_lvl_ptr(&self) -> bool {
        !(self.R() || self.X())
    }

    pub fn is_invalid(&self) -> bool {
        !self.valid() || (self.W() && !self.R())
    }

    pub fn check_access_type(&self, typ: MemoryAccessType, MXR: bool) -> bool {
        match typ {
            MemoryAccessType::R => self.R() || (self.X() && MXR),
            MemoryAccessType::W => self.W(),
            MemoryAccessType::X => self.X(),
        }
    }
}

pub enum TranslationErr {
    AccessFault,
    PageFault,
}

#[derive(PartialEq)]
pub enum MemoryAccessType {
    R,
    W,
    X,
}

impl MMU {
    pub fn new(mem: Memory, privilege: Rc<RefCell<RISCV64Privilege>>) -> Self {
        Self {
            mem,
            translation_ctrl: TranslationCtrl::new(privilege),
        }
    }

    pub fn paddr_to_vaddr(&self, paddr: &PAddr) -> VAddr {
        assert!(self.translation_ctrl.is_bare);
        VAddr::new(paddr.value())
    }

    fn pt_walk_debug(&self, vaddr: u64) {
        if log::max_level() < log::LevelFilter::Debug {
            return;
        }
        debug!("pt_walk_debug begin, vaddr {:#x}", vaddr);
        let vpn = [
            (vaddr >> 12) & 0b111111111,
            (vaddr >> 21) & 0b111111111,
            (vaddr >> 30) & 0b111111111,
        ];
        let mut a = self.translation_ctrl.sv39.lvl1_base;
        let mut i = 2;
        for _ in 0..3 {
            let lvl = i;
            debug!(
                "try get pte {} at {:#x} ({:#x}+{:#x})",
                i,
                a + vpn[lvl] * 8,
                a,
                vpn[lvl] * 8
            );
            let pte = SV39PTE::from(
                self.mem
                    .read_mem(&PAddr::new(a + vpn[lvl] * 8), MemOperationSize::DWORD)
                    .unwrap(),
            );
            debug!("pte {} {:#x}", i, pte.0);
            if pte.0 == 0x0 {
                return;
            }
            if pte.is_invalid() {
                return;
            }
            if !pte.is_next_lvl_ptr() {
                return;
            }
            a = pte.PPN() << 12;
            i -= 1;
        }
    }

    pub fn translate(
        &mut self,
        vaddr: &VAddr,
        typ: MemoryAccessType,
    ) -> Result<PAddr, TranslationErr> {
        if self.translation_ctrl.is_bare
            || (*self.translation_ctrl.privilege.borrow() == RISCV64Privilege::M
                && !(typ != MemoryAccessType::X && self.translation_ctrl.translate_in_m))
        {
            return Ok(PAddr::new(vaddr.value()));
        }
        trace!("translate vaddr {:#x}", vaddr.value());

        let vaddr = vaddr.value();
        let ofs = vaddr & 0b111111111111;
        let vpn = [
            (vaddr >> 12) & 0b111111111,
            (vaddr >> 21) & 0b111111111,
            (vaddr >> 30) & 0b111111111,
        ];
        let mut pte_addrs = [0u64; 3];

        let mut a = self.translation_ctrl.sv39.lvl1_base;
        let mut res_pte: Option<SV39PTE> = None;
        let mut i = 2;
        for _ in 0..3 {
            let lvl = i;
            // info!("try get pte {} at {:#x}", i, a + vpn[lvl] * 8);
            let pte = SV39PTE::from(
                self.mem
                    .read_mem(&PAddr::new(a + vpn[lvl] * 8), MemOperationSize::DWORD)
                    .ok_or(AccessFault)?,
            );
            // info!("pte {} {:#x}", i, pte.0);
            if pte.0 == 0x0 {
                self.pt_walk_debug(vaddr);
                warn!("PTE IS ZERO, vaddr = {:#x}", vaddr)
                // return Err(PageFault);
            }
            if pte.is_invalid() {
                return Err(PageFault);
            }
            pte_addrs[lvl] = a + vpn[lvl] * 8;
            if !pte.is_next_lvl_ptr() {
                res_pte = Some(pte);
                break;
            }
            a = pte.PPN() << 12;
            i -= 1;
        }
        if res_pte.is_none() {
            warn!("res_pte is none");
            return Err(PageFault);
        }
        let res_pte = res_pte.unwrap();
        // info!("res pte {} {:#x}", i, res_pte.0);
        if i > 0 && (res_pte.PPN() & ((1 << (9 * i)) - 1)) != 0 {
            warn!("misaligned super page, ppn {:#x}", res_pte.PPN());
            return Err(PageFault); // misaligned super page
        }

        let privilege = *self.translation_ctrl.privilege.borrow();
        if privilege == RISCV64Privilege::U && !res_pte.U() {
            return Err(PageFault);
        }
        if res_pte.U() && privilege != RISCV64Privilege::U && !self.translation_ctrl.SUM {
            return Err(PageFault);
        }
        if !res_pte.check_access_type(typ, self.translation_ctrl.MXR) {
            warn!("check_access_type failed");
            return Err(PageFault);
        }

        // update pte.d
        for j in i..3 {
            self.mem
                .pmem_bitor(
                    &PAddr::new(pte_addrs[j]),
                    0b10000000,
                    MemOperationSize::DWORD,
                )
                .unwrap();
        }

        // TODO: pte.a
        let paddr = if i > 0 {
            let ofs_len = 9 * i + 12;
            (res_pte.PPN() << 12) | vaddr & ((1 << ofs_len) - 1)
        } else {
            (res_pte.PPN() << 12) | ofs
        };
        // info!("PTE PPN is {:#x}, i is {}", res_pte.PPN(), i);
        // info!("translate {:#x} to {:#x}", vaddr, paddr);
        Ok(PAddr::new(paddr))
    }

    pub fn ifetch(
        &mut self,
        vaddr: &VAddr,
        len: MemOperationSize,
    ) -> Result<(u64, PAddr), MCauseCode> {
        match self.translate(vaddr, MemoryAccessType::X) {
            Ok(paddr) => match self.mem.read(&paddr, len) {
                Some(v) => Ok((v, paddr)),
                None => Err(InstAccessFault),
            },
            Err(e) => match e {
                AccessFault => Err(InstAccessFault),
                PageFault => Err(InstPageFault),
            },
        }
    }
    pub fn read(&mut self, vaddr: &VAddr, len: MemOperationSize) -> Result<u64, MCauseCode> {
        match self.translate(vaddr, MemoryAccessType::R) {
            Ok(paddr) => match self.mem.read(&paddr, len) {
                Some(v) => Ok(v),
                None => Err(LoadAccessFault),
            },
            Err(e) => match e {
                AccessFault => Err(LoadAccessFault),
                PageFault => Err(LoadPageFault),
            },
        }
    }

    pub fn write(
        &mut self,
        vaddr: &VAddr,
        data: u64,
        len: MemOperationSize,
    ) -> Result<(), MCauseCode> {
        match self.translate(vaddr, MemoryAccessType::W) {
            Ok(paddr) => match self.mem.write(&paddr, data, len) {
                Ok(_) => Ok(()),
                Err(_) => Err(StoreAMOAccessFault),
            },
            Err(e) => match e {
                AccessFault => Err(StoreAMOAccessFault),
                PageFault => Err(StoreAMOPageFault),
            },
        }
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

    pub fn update_priv(&mut self, mstatus: &MStatus) {
        self.translation_ctrl.SUM = mstatus.SUM();
        self.translation_ctrl.MXR = mstatus.MXR();
        self.translation_ctrl.translate_in_m = mstatus.MPRV() && !mstatus.MPP_is_m_mode();
    }
}

impl TranslationCtrl {
    pub fn new(privilege: Rc<RefCell<RISCV64Privilege>>) -> Self {
        Self {
            is_bare: true,
            sv39: SV39::new(),
            privilege,
            SUM: false,
            MXR: false,
            translate_in_m: false,
        }
    }
}

impl SV39 {
    pub fn new() -> Self {
        Self { lvl1_base: 0 }
    }
}
