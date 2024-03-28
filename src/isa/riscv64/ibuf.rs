use cfg_if::cfg_if;
use crate::isa::riscv64::inst::{Decode, Pattern, PATTERNS};
use crate::memory::paddr::PAddr;
use crate::utils::cfg_if_feat;

const IBUF_ENTRY_MASK: usize = 0xffff;

type BufContent = (&'static Pattern, Decode);

#[derive(Clone)]
pub struct IBufEntry {
    pc: PAddr,
    content: BufContent,
}

impl IBufEntry {
    fn create_empty() -> Self {
        Self {
            pc: PAddr::new(0),
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

    pub(crate) fn get(&mut self, pc: &PAddr) -> Option<&BufContent> {
        if self.entries.pc == *pc {
            return Some(&self.entries.content);
        }
        None
    }

    pub(crate) fn set(&mut self, pc: &PAddr, pat: &'static Pattern, decode: Decode) -> &BufContent {
        self.entries.pc = pc.clone();
        self.entries.content.0 = pat;
        self.entries.content.1 = decode;
        &self.entries.content
    }
}

// #[derive(Clone)]
// struct IBufRow {
//     entries: (IBufEntry, IBufEntry),
//     last_used_is_0: bool,
// }
//
//
// impl IBufRow {
//     pub(crate) fn new() -> Self {
//         Self {
//             entries: (IBufEntry::create_empty(), IBufEntry::create_empty()),
//             last_used_is_0: false,
//         }
//     }
//
//     pub(crate) fn get(&mut self, pc: &PAddr) -> Option<&BufContent> {
//         if self.entries.0.pc == *pc {
//             self.last_used_is_0 = true;
//             return Some(&self.entries.0.content);
//         }
//         if self.entries.1.pc == *pc {
//             self.last_used_is_0 = false;
//             return Some(&self.entries.1.content);
//         }
//         None
//     }
//
//     pub(crate) fn set(&mut self, pc: &PAddr, pat: &'static Pattern, decode: Decode) -> &BufContent {
//         let not_used = match self.last_used_is_0 {
//             true => &mut self.entries.1,
//             false => &mut self.entries.0
//         };
//         not_used.pc = pc.clone();
//         not_used.content.0 = pat;
//         not_used.content.1 = decode;
//         self.last_used_is_0 = !self.last_used_is_0;
//         &not_used.content
//     }
// }

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
        // (pc.clone() - self.mem_base).value() as usize & IBUF_ENTRY_MASK
        pc.value() as usize & IBUF_ENTRY_MASK
    }

    pub(crate) fn get(&mut self, pc: &PAddr) -> Option<&BufContent> {
        let idx = self.get_entry_idx(pc);
        let res = unsafe { self.entries.get_unchecked_mut(idx) }.get(pc);
        cfg_if_feat!("log_inst", {
            match res {
                None => self.missed += 1,
                Some(_) => self.hit += 1
            }
        });
        res
    }

    pub(crate) fn set(&mut self, pc: &PAddr, pat: &'static Pattern, decode: Decode) -> &BufContent {
        let idx = self.get_entry_idx(pc);
        unsafe { self.entries.get_unchecked_mut(idx) }.set(pc, pat, decode)
    }

    pub(crate) fn print_info(&self) {
        let total = (self.hit + self.missed) as f64;
        println!("Hit: {}({}), Missed: {}({})", self.hit, self.hit as f64 / total, self.missed, self.missed as f64 / total)
    }
}