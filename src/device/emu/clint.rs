use crate::device::glob_timer;
use crate::isa::riscv64::csr::InterruptMask;
use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;
use log::trace;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;
use std::{sync, thread};

pub const CLINT_MMIO_START: PAddr = PAddr::new(0x2000000);

const MSIP_OFFSET: usize = 0;
const MTIMECMP_OFFSET: usize = 0x4000;
const MTIME_OFFSET: usize = 0xBFF8;

pub struct CLINT {
    // cpu_interrupt_bits: Arc<AtomicU64>,
    mtimecmp_update_notify: Sender<()>,
    mtimecmp: Arc<AtomicU64>,
}

impl CLINT {
    pub fn new(cpu_interrupt_bits: Arc<AtomicU64>, stopped: Arc<AtomicBool>) -> Self {
        let (tx, rx) = sync::mpsc::channel::<()>();

        let mtimecmp = Arc::new(AtomicU64::new(1145141919810));
        let mtimecmp_clone = mtimecmp.clone();
        thread::spawn(move || {
            while !stopped.load(Relaxed) {
                let mtimecmp = mtimecmp_clone.load(SeqCst);
                let now = glob_timer.lock().unwrap().since_boot_us();
                let wait_res = if mtimecmp > now {
                    trace!(
                        "mtimecmp({}) > now({}), next trigger at {}ms",
                        mtimecmp,
                        now,
                        Duration::from_micros(mtimecmp - now).as_millis()
                    );
                    cpu_interrupt_bits.fetch_and(!0b10000000, SeqCst);
                    rx.recv_timeout(Duration::from_micros(mtimecmp - now))
                } else {
                    trace!(
                        "mtimecmp({}) <= now({}), next trigger at {}ms",
                        mtimecmp,
                        now,
                        Duration::from_micros(now - mtimecmp).as_millis()
                    );
                    cpu_interrupt_bits.fetch_or(InterruptMask::MTimerInt as u64, SeqCst);
                    rx.recv_timeout(Duration::from_micros(now - mtimecmp))
                };
                if let Err(e) = wait_res {
                    if e == RecvTimeoutError::Disconnected {
                        break;
                    }
                }
            }
        });

        Self {
            // cpu_interrupt_bits,
            mtimecmp_update_notify: tx,
            mtimecmp,
        }
    }
}

impl IOMap for CLINT {
    fn len(&self) -> usize {
        0x10000
    }

    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of clint")
        }
        match offset {
            MSIP_OFFSET => {
                todo!("read msip");
            }
            MTIMECMP_OFFSET => len.read_val(self.mtimecmp.load(SeqCst)),
            MTIME_OFFSET => len.read_val(glob_timer.lock().unwrap().since_boot_us()),
            _ => 0,
        }
    }

    fn write(&mut self, offset: usize, data: u64, len: MemOperationSize) {
        if offset % (len as usize) != 0 {
            panic!("misaligned access of clint")
        }
        match offset {
            MSIP_OFFSET => {
                if data & 1 != 0 {
                    todo!("write msip val: {}", data)
                }
            }
            MTIMECMP_OFFSET => {
                self.mtimecmp.store(len.read_val(data), SeqCst);
                println!("mtime: {}", glob_timer.lock().unwrap().since_boot_us());
                println!("write mtimecmp val: {}", data);
                self.mtimecmp_update_notify.send(()).unwrap();
            }
            MTIME_OFFSET => {
                todo!("write mtime val: {}", data)
            }
            _ => {}
        }
    }
}
