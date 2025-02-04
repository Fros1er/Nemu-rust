use crate::isa::riscv64::inst::{Decode, Pattern, PATTERNS};
use crate::memory::paddr::PAddr;
use crate::utils::cfg_if_feat;
use cfg_if::cfg_if;
use std::cell::RefCell;

const IBUF_ENTRY_MASK: usize = 0xffff;

pub type BufContent = (&'static Pattern, Decode);

#[derive(Clone)]
pub struct IBufEntry {
    pc: PAddr,
    inst: u64, // TODO: use fence.i instead
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
    entries: IBufEntry,
}

impl IBufRow {
    pub(crate) fn new() -> Self {
        Self {
            entries: IBufEntry::create_empty(),
        }
    }

    pub(crate) fn get(&self, pc: &PAddr, inst: u64) -> Option<&BufContent> {
        if self.entries.pc == *pc && self.entries.inst == inst {
            return Some(&self.entries.content);
        }
        None
    }

    pub(crate) fn set(
        &mut self,
        pc: &PAddr,
        inst: u64,
        pat: &'static Pattern,
        decode: Decode,
    ) -> &BufContent {
        self.entries.pc = pc.clone();
        self.entries.inst = inst;
        self.entries.content.0 = pat;
        self.entries.content.1 = decode;
        &self.entries.content
    }
}

struct IBufStatistics {
    hit: u64,
    missed: u64,
}

pub struct SetAssociativeIBuf {
    entries: Box<[IBufRow]>,
    stat: RefCell<IBufStatistics>,
}

impl SetAssociativeIBuf {
    pub(crate) fn new() -> Self {
        Self {
            entries: vec![IBufRow::new(); IBUF_ENTRY_MASK].into_boxed_slice(),
            stat: RefCell::new(IBufStatistics { hit: 0, missed: 0 }),
        }
    }

    fn get_entry_idx(&self, pc: &PAddr) -> usize {
        pc.value() as usize & IBUF_ENTRY_MASK
    }

    cfg_if_feat!("log_inst", {
        fn update(&self, hit: bool) {
            let stat = &mut *self.stat.borrow_mut();
            match hit {
                true => stat.hit += 1,
                false => stat.missed += 1,
            }
        }
    });

    pub(crate) fn get(&self, pc: &PAddr, inst: u64) -> Option<&BufContent> {
        let idx = self.get_entry_idx(pc);
        let res = unsafe { self.entries.get_unchecked(idx) }.get(pc, inst);
        cfg_if_feat!("log_inst", { self.update(res.is_some()) });
        res
    }

    pub(crate) fn set(
        &mut self,
        pc: &PAddr,
        inst: u64,
        pat: &'static Pattern,
        decode: Decode,
    ) -> &BufContent {
        let idx = self.get_entry_idx(pc);
        unsafe { self.entries.get_unchecked_mut(idx) }.set(pc, inst, pat, decode)
    }

    pub(crate) fn print_info(&self) {
        let stat = self.stat.borrow();
        let total = (stat.hit + stat.missed) as f64;
        println!(
            "Hit: {}({}), Missed: {}({})",
            stat.hit,
            stat.hit as f64 / total,
            stat.missed,
            stat.missed as f64 / total
        )
    }
}
