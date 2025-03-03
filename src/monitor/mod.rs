pub mod sdb;

use crate::memory::Memory;
use crate::utils::configs::{CONFIG_IMAGE_BASE, CONFIG_MEM_SIZE};
use clap::{Parser, ValueEnum};
use log::{info, LevelFilter};
use simplelog::{SimpleLogger, WriteLogger};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[derive(Default)]
pub struct Args {
    /// IMAGE
    #[arg(long)]
    pub(crate) image: Option<String>,

    /// Firmware
    #[arg(long)]
    pub(crate) firmware: String,

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
    let cfg = simplelog::ConfigBuilder::default()
        .add_filter_ignore_str("rustyline")
        .build();
    match &args.log {
        Some(log_file) => {
            let path = Path::new(log_file);
            match File::create(path) {
                Ok(file) => WriteLogger::init(args.log_level.into(), cfg, file),
                Err(why) => panic!("couldn't create {}: {}", path.display(), why),
            }
        }
        None => SimpleLogger::init(args.log_level.into(), cfg),
    }
    .expect("Failed to create logger.");
}

pub(crate) fn load_firmware(firm_file: &String, has_img: bool, memory: &mut Memory) -> usize {
    let path = Path::new(firm_file);
    let mut f = File::open(path).unwrap();
    let size = f.metadata().unwrap().len();
    if (has_img && size > CONFIG_IMAGE_BASE as u64) || size > CONFIG_MEM_SIZE as u64 {
        panic!("Firm too large ({} or {:#x} bytes).", size, size);
    }
    f.read(&mut memory.pmem).unwrap()
}

pub(crate) fn load_img(img_file: &String, memory: &mut Memory) -> usize {
    info!("loading image {}", img_file);
    let path = Path::new(img_file);
    let mut f = File::open(path).unwrap();
    let size = f.metadata().unwrap().len();
    if size > (CONFIG_MEM_SIZE - CONFIG_IMAGE_BASE) as u64 {
        panic!("Image too large ({} or {:#x} bytes).", size, size);
    }
    f.read(&mut memory.pmem[CONFIG_IMAGE_BASE..]).unwrap()
}
