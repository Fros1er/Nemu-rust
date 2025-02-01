use cfg_if::cfg_if;
use crate::isa::riscv64::inst::{Decode, Pattern, PATTERNS};
use crate::memory::paddr::PAddr;
use crate::utils::cfg_if_feat;

const IBUF_ENTRY_MASK: usize = 0xffff;

pub type BufContent = (&'static Pattern, Decode);

#[derive(Clone)]
pub struct IBufEntry {
    pc: PAddr,
    inst: u64,
    content: BufContent,
}

impl IBufEntry {
    fn create_empty() -> Self {
        Self {
            pc: PAddr::new(0),
            inst: 0,
            content: (&PATTERNS[0], Default::default()),
        }
    }
}

#[derive(Clone)]
struct IBufRow {
    entries: IBufEntry
}

impl IBufRow {
    pub(crate) fn new() -> Self {
        Self {
            entries: IBufEntry::create_empty()
        }
    }

    pub(crate) fn get(&mut self, pc: &PAddr, inst: u64) -> Option<&BufContent> {
        if self.entries.pc == *pc && self.entries.inst == inst {
            return Some(&self.entries.content);
        }
        None
    }

    pub(crate) fn set(&mut self, pc: &PAddr, inst: u64, pat: &'static Pattern, decode: Decode) -> &BufContent {
        self.entries.pc = pc.clone();
        self.entries.inst = inst;
        self.entries.content.0 = pat;
        self.entries.content.1 = decode;
        &self.entries.content
    }
}

pub struct SetAssociativeIBuf {
    entries: Box<[IBufRow]>,
    hit: u64,
    missed: u64,
}

impl SetAssociativeIBuf {
    pub(crate) fn new() -> Self {
        Self {
            entries: vec![IBufRow::new(); IBUF_ENTRY_MASK].into_boxed_slice(),
            hit: 0,
            missed: 0,
        }
    }

    fn get_entry_idx(&self, pc: &PAddr) -> usize {
        pc.value() as usize & IBUF_ENTRY_MASK
    }

    pub(crate) fn get(&mut self, pc: &PAddr, inst: u64) -> Option<&BufContent> {
        let idx = self.get_entry_idx(pc);
        let res = unsafe { self.entries.get_unchecked_mut(idx) }.get(pc, inst);
        cfg_if_feat!("log_inst", {
            match res {
                None => self.missed += 1,
                Some(_) => self.hit += 1
            }
        });
        res
    }

    pub(crate) fn set(&mut self, pc: &PAddr, inst: u64, pat: &'static Pattern, decode: Decode) -> &BufContent {
        let idx = self.get_entry_idx(pc);
        unsafe { self.entries.get_unchecked_mut(idx) }.set(pc, inst, pat, decode)
    }

    pub(crate) fn print_info(&self) {
        let total = (self.hit + self.missed) as f64;
        println!("Hit: {}({}), Missed: {}({})", self.hit, self.hit as f64 / total, self.missed, self.missed as f64 / total)
    }
}