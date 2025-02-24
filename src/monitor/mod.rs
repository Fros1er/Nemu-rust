pub mod sdb;

use crate::memory::Memory;
use crate::utils::configs::{CONFIG_FIRMWARE_SIZE, CONFIG_MEM_SIZE};
use clap::{Parser, ValueEnum};
use log::{info, LevelFilter};
use simplelog::{Config, SimpleLogger, WriteLogger};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[derive(Default)]
pub struct Args {
    /// IMAGE
    pub(crate) image: String,

    /// Firmware
    #[arg(long)]
    pub(crate) firmware: Option<String>,

    /// run without SDL devices
    #[arg(short, long)]
    pub no_sdl_devices: bool,

    /// run with difftest
    #[arg(short, long)]
    pub difftest: bool,

    /// run with batch mode
    #[arg(short, long)]
    pub batch: bool,

    /// output log to FILE
    #[arg(short, long, value_name = "FILE")]
    pub(crate) log: Option<String>,

    /// don't stop at hardware breakpoint
    #[arg(long)]
    pub ignore_isa_breakpoint: bool,

    #[arg(long)]
    pub log_level: LogLevel,
}

#[derive(ValueEnum, Debug, Default, Copy, Clone)]
pub enum LogLevel {
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

pub fn init_log(args: &Args) {
    match &args.log {
        Some(log_file) => {
            let path = Path::new(log_file);
            match File::create(path) {
                Ok(file) => WriteLogger::init(args.log_level.into(), Config::default(), file),
                Err(why) => panic!("couldn't create {}: {}", path.display(), why),
            }
        }
        None => SimpleLogger::init(args.log_level.into(), Config::default()),
    }
    .expect("Failed to create logger.");
}

pub(crate) fn load_firmware(img_file: &String, memory: &mut Memory) -> usize {
    let path = Path::new(img_file);
    let mut f = File::open(path).unwrap();
    let size = f.metadata().unwrap().len();
    if size > CONFIG_FIRMWARE_SIZE {
        panic!("Firm too large ({} or {:#x} bytes).", size, size);
    }
    // f.read(&mut memory.firmware).unwrap()
    f.read(&mut memory.pmem).unwrap()
}

pub(crate) fn load_img(img_file: &String, memory: &mut Memory) -> usize {
    info!("loading image {}", img_file);
    let path = Path::new(img_file);
    let mut f = File::open(path).unwrap();
    let size = f.metadata().unwrap().len();
    if size > CONFIG_MEM_SIZE - 0x200000 {
        panic!("Image too large ({} or {:#x} bytes).", size, size);
    }
    f.read(&mut memory.pmem[0x200000..]).unwrap()
}

// pub fn init_monitor<U: CPUState, T: Isa<U>>() -> T {
//     let args = Args::parse();
//     init_log(args.log.as_ref());
//     init_mem();
//     init_device();
//     let mut isa = T::new();
//     isa.init_isa();
//     let img_size = load_img(args.image.as_ref());
//     isa
// }
