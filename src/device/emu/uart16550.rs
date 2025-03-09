use crate::device::emu::plic::PLIC;
use crate::isa::riscv64::vaddr::MemOperationSize;
use crate::memory::paddr::PAddr;
use crate::memory::IOMap;
use bitfield_struct::bitfield;
use log::{info, trace};
use ringbuf::storage::Heap;
use ringbuf::traits::*;
use ringbuf::{CachingCons, CachingProd, HeapRb, SharedRb};
use std::cell::UnsafeCell;
use std::process::Command;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const UART16550_MMIO_START: PAddr = PAddr::new(0x10000000);

struct LCR {
    word_len: u8,
    dlab_en: bool,
}

#[bitfield(u8)]
pub struct IER {
    // data_ready_int_en: Arc<AtomicBool>,
    data_ready_int_en: bool,
    #[bits(7)]
    _1: u8,
}

#[bitfield(u8)]
pub struct ISR {
    interrupt_status: bool, // 0 if interrupt is pending
    #[bits(3)]
    code: u8,
    #[bits(4)]
    _1: u8,
}

pub struct UART16550 {
    in_rb: UnsafeCell<CachingCons<Arc<SharedRb<Heap<u8>>>>>,
    out_rb: UnsafeCell<CachingProd<Arc<SharedRb<Heap<u8>>>>>,
    out_notify: Arc<tokio::sync::Notify>,
    lcr: LCR,
    ier: Arc<AtomicU8>,
    isr: Arc<AtomicU8>,
    mem: [u8; 16],
    plic: Arc<Mutex<PLIC>>,
}

impl UART16550 {
    pub fn new(
        plic: Arc<Mutex<PLIC>>,
        stopped: Arc<AtomicBool>,
        term_close_timeout: Option<u64>,
    ) -> Self {
        let (mut in_prod, in_cons) = HeapRb::<u8>::new(256).split();
        let (out_prod, mut out_cons) = HeapRb::<u8>::new(256).split();

        let notify = Arc::new(tokio::sync::Notify::new());
        let notify_clone = notify.clone();

        let stopped_clone = stopped.clone();

        let ier = Arc::new(AtomicU8::new(0));
        let ier_clone = ier.clone();

        let isr = Arc::new(AtomicU8::new(0));
        let isr_clone = isr.clone();

        let plic_clone = plic.clone();
        let _tokio_thread = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build()
                .unwrap();
            runtime.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:14514")
                    .await
                    .unwrap();
                info!("UART Server started at 127.0.0.1:14514");
                let (socket, _) = listener.accept().await.unwrap();
                let (mut reader, mut writer) = tokio::io::split(socket);
                info!("Client connected");

                // Reader
                let reader = tokio::task::spawn(async move {
                    let mut buf = [0u8; 256];
                    while !stopped_clone.load(Relaxed) {
                        let n = reader.read(&mut buf).await.unwrap();
                        let mut now = 0;
                        while now < n {
                            now += in_prod.push_slice(&buf[now..n]);
                        }
                        if (ier_clone.load(SeqCst) & 1 == 1) && n > 0 {
                            isr_clone.store(0b0100, SeqCst);
                            plic_clone.lock().unwrap().trigger_interrupt(10);
                        }
                    }
                });

                // Writer
                let writer = tokio::spawn(async move {
                    writer
                        .write_all("Nemu Rust UART\n".as_bytes())
                        .await
                        .unwrap();
                    let mut buf = [0u8; 256];
                    while !stopped.load(Relaxed) {
                        notify.notified().await;
                        let n = out_cons.pop_slice(&mut buf);
                        writer.write_all(&buf[..n]).await.unwrap();
                        writer.flush().await.unwrap();
                    }
                });

                let _ = tokio::join!(reader, writer);
            });
        });

        if term_close_timeout.is_some() && term_close_timeout.unwrap() == 0 {
            info!("UART Server won't start due to term_timeout is 0");
        } else {
            let timeout_str = match term_close_timeout {
                Some(timeout) => format!("-w {}", timeout),
                None => "".to_string(),
            };
            let _ = Command::new("gnome-terminal")
                .args(&[
                    "--",
                    "bash",
                    "-c",
                    // format!("stty -echo -icanon && nc {} 127.0.0.1 14514", timeout_str).as_str(),
                    format!(
                        "stty -echo -icanon && stdbuf -i0 -o0 -e0 nc {} 127.0.0.1 14514",
                        timeout_str
                    )
                    .as_str(),
                ])
                .spawn()
                .unwrap()
                .wait();
        }

        Self {
            in_rb: UnsafeCell::new(in_cons),
            out_rb: UnsafeCell::new(out_prod),
            out_notify: notify_clone,
            lcr: LCR {
                word_len: 0,
                dlab_en: false,
            },
            ier,
            isr,
            mem: [0; 16],
            plic,
        }
    }

    fn clear_interrupt(&self) {
        if self.ier.load(SeqCst) & 2 == 0 {
            self.isr.store(0b1, SeqCst);
            self.plic.lock().unwrap().clear_interrupt(10);
        } else {
            self.isr.store(0b10, SeqCst);
            self.plic.lock().unwrap().trigger_interrupt(10);
        }
    }
}

impl IOMap for UART16550 {
    fn len(&self) -> usize {
        8
    }
    fn read(&self, offset: usize, len: MemOperationSize) -> u64 {
        assert!(len == MemOperationSize::Byte);
        let rb = unsafe { &mut *self.in_rb.get() };
        // info!("UART read. ofs: {}", offset);
        match offset {
            0 => {
                let res = rb.try_pop().unwrap_or(0) as u64;
                trace!(
                    "UART read. INT: {}, HAS_VALUE: {}",
                    self.ier.load(SeqCst) & 1,
                    !rb.is_empty()
                );
                if self.ier.load(SeqCst) & 1 == 1 && rb.is_empty() {
                    self.clear_interrupt();
                }
                res
            }
            1 => self.ier.load(SeqCst) as u64,
            2 => self.isr.load(SeqCst) as u64,
            3 => (self.lcr.word_len | ((self.lcr.dlab_en as u8) << 7)) as u64,
            5 => {
                let mut res = 0x20;
                if !rb.is_empty() {
                    res |= 0x1;
                }
                res
            } // lsr
            6 => 0b11110000,
            _ => todo!("uart16550 ofs {}", offset),
        }
    }

    fn write(&mut self, offset: usize, data: u64, len: MemOperationSize) {
        assert!(len == MemOperationSize::Byte);
        // info!("UART write. ofs: {}, data: {}", offset, data);
        let mem_offset = if self.lcr.dlab_en {
            match offset {
                0b000 | 0b001 | 0b101 => offset | 0b1000,
                _ => offset,
            }
        } else {
            offset
        };
        len.write_sized(data, self.mem.get_mut(mem_offset).unwrap() as *mut u8);

        let data = data as u8;
        match offset {
            0 => {
                if !self.lcr.dlab_en {
                    loop {
                        let ok = unsafe { (*self.out_rb.get()).try_push(data).is_ok() };
                        if ok {
                            break;
                        }
                    }
                    self.out_notify.notify_one();
                }
            }
            // IER
            1 => {
                // assert_eq!(data & (!0b11), 0); // only support data-ready interrupt
                info!("uart ier set: {}", data);
                self.ier.store(data, SeqCst);
            }
            // FCR
            2 => {
                // assert_eq!(data & (!0x1), 0); // fifo not supported
            }
            // LCR
            3 => {
                self.lcr.word_len = data & 0x11;
                self.lcr.dlab_en = data & 0x80 != 0;
            }
            // MCR
            4 => {
                // assert_eq!(data & 0b1000, 0);
            }
            // SPR
            7 => {}
            _ => panic!("write to UART16550+{:#x} should not happen", offset),
        }
    }
}
