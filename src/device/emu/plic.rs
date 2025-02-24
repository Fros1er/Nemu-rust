use crate::isa::riscv64::csr::InterruptMask;
use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;
use log::{debug, trace};
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

pub const PLIC_MMIO_START: PAddr = PAddr::new(0xc000000);
const PLIC_NDEV: usize = 16;
const NHART: usize = 1;

pub struct PLIC {
    cpu_interrupt_bits: Arc<AtomicU64>,
    priority: [u8; PLIC_NDEV],
    device_bits: u64,
    enable_bits: [u64; NHART * 2],
    pending_bits: UnsafeCell<[u64; NHART * 2]>,
}

impl PLIC {
    pub fn new(cpu_interrupt_bits: Arc<AtomicU64>) -> Self {
        Self {
            cpu_interrupt_bits,
            priority: [0; PLIC_NDEV],
            device_bits: 0,
            enable_bits: [0; NHART * 2],
            pending_bits: UnsafeCell::new([0; NHART * 2]),
        }
    }

    pub fn trigger_interrupt(&mut self, id: u64) {
        self.device_bits |= 1 << id;
        let trigger_m = self.enable_bits[0] & (1 << id) != 0;
        let trigger_s = self.enable_bits[1] & (1 << id) != 0;
        debug!("Trigger PLIC interrupt, M: {}, S: {}", trigger_m, trigger_s);
        if trigger_m {
            self.cpu_interrupt_bits
                .fetch_or(InterruptMask::MExtInt as u64, SeqCst);
            self.pending_bits.get_mut()[0] |= 1 << id;
        }
        if trigger_s {
            self.cpu_interrupt_bits
                .fetch_or(InterruptMask::SExtInt as u64, SeqCst);
            self.pending_bits.get_mut()[1] |= 1 << id;
        }
    }
    pub fn clear_interrupt(&mut self, id: u64) {
        self.device_bits &= !(1 << id);
    }

    fn claim_complete(&self, ctx: usize) -> u64 {
        trace!("plic ctx {} claim/complete", ctx);
        assert!(ctx <= 1);
        let mut en = 0;
        let mut max = 0;

        unsafe {
            if (*self.pending_bits.get())[ctx] == 0 {
                return 0;
            }

            for i in (0..PLIC_NDEV).rev() {
                if (*self.pending_bits.get())[ctx] & (1 << i) != 0 && self.priority[i] >= max {
                    max = self.priority[i];
                    en = i;
                }
            }
            if self.device_bits & (1 << en) == 0 {
                trace!("device bit is clear");
                (*self.pending_bits.get())[ctx] &= !(1 << en);
                if (*self.pending_bits.get())[ctx] == 0 {
                    match ctx {
                        0 => self
                            .cpu_interrupt_bits
                            .fetch_and(!(InterruptMask::MExtInt as u64), SeqCst),
                        1 => self
                            .cpu_interrupt_bits
                            .fetch_and(!(InterruptMask::SExtInt as u64), SeqCst),
                        _ => {
                            panic!("")
                        }
                    };
                };
            } else {
                trace!("device bit is not clear");
            }
        }
        en as u64
    }
}

impl IOMap for PLIC {
    fn len(&self) -> usize {
        0x4000000
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of PLIC")
        }

        if offset < 0x1000 {
            assert_eq!(offset % 4, 0);
            assert!(offset / 4 < 16);
            // todo!("Read PLIC priority, offset {:#x} ", offset);
            return self.priority[offset / 4] as u64;
        } else if offset == 0x1000 {
            todo!("Read PLIC pending bit");
        } else if offset >= 0x2000 && offset < 0x1f1ffc {
            // enable bit
            assert_eq!(offset % 0x80, 0);
            let ctx = (offset - 0x2000) / 0x80;
            assert!(ctx < NHART * 2);
            return len.read_val(self.enable_bits[ctx]);
        } else if offset >= 0x200000 && offset < 0x3ff008 {
            let ctx = (offset - 0x200000) >> 12;
            if offset & 0xfff == 0x4 {
                return self.claim_complete(ctx);
            } else if offset & 0xfff == 0 {
                // threshold, ignored
                return 0;
            }
        }
        todo!("Read PLIC offset {:#x}", offset);
    }

    fn write(&mut self, offset: usize, data: u64, len: MemOperationSize) {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of PLIC")
        }
        if offset < 0x1000 {
            assert_eq!(offset % 4, 0);
            assert!(offset / 4 < 16);
            // todo!("Write PLIC priority, offset {:#x} ", offset);
            self.priority[offset / 4] = data as u8;
            return;
        } else if offset == 0x1000 {
            todo!("Write PLIC pending bit");
        } else if offset >= 0x2000 && offset < 0x1f1ffc {
            // enable bit
            assert_eq!(offset % 0x80, 0);
            let ctx = (offset - 0x2000) / 0x80;
            assert!(ctx < NHART * 2);
            self.enable_bits[ctx] = len.read_val(data);
            return;
        } else if offset >= 0x200000 && offset < 0x3ff008 {
            let ctx = (offset - 0x200000) >> 12;
            if offset & 0xfff == 0x4 {
                self.claim_complete(ctx);
                return;
            } else if offset & 0xfff == 0 {
                // threshold, ignored
                return;
            }
        }
        todo!("Write PLIC offset {:#x} data {:#x}", offset, data);
    }
}
