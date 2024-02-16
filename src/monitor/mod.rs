use std::fs::File;
use std::io::Read;
use std::path::Path;
use clap::Parser;
use log::{info, LevelFilter};
use simplelog::{Config, SimpleLogger, WriteLogger};
use crate::device::init_device;
use crate::isa::Isa;
use crate::isa::riscv64::RISCV64;
use crate::memory::paddr::{init_mem, PMEM};
use crate::utils::configs::{CONFIG_MBASE, CONFIG_PC_RESET_OFFSET};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IMAGE
    image: Option<String>,

    /// run with batch mode
    #[arg(short, long)]
    batch: bool,

    /// output log to FILE
    #[arg(short, long, value_name = "FILE")]
    log: Option<String>,

    // /// run DiffTest with reference REF_SO
    // #[arg(short, long, value_name = "REF_SO")]
    // diff: String,
    //
    // /// run DiffTest with port PORT
    // #[arg(short, long, value_name = "PORT")]
    // port: i32,
}

pub fn init_log(log_file: Option<&String>) {
    match log_file {
        Some(log_file) => {
            let path = Path::new(log_file);
            match File::create(path) {
                Ok(file) => WriteLogger::init(LevelFilter::Info, Config::default(), file),
                Err(why) => panic!("couldn't create {}: {}", path.display(), why)
            }
        }
        None => SimpleLogger::init(LevelFilter::Info, Config::default())
    }.expect("Failed to create logger.");
}

fn load_img(img_file: Option<&String>) -> usize {
    match img_file {
        None => {
            info!("No image is given. Use the default build-in image.");
            4096
        }
        Some(img_file) => {
            unsafe {
                let start = (CONFIG_MBASE + CONFIG_PC_RESET_OFFSET).to_host_arr_index();
                File::open(Path::new(img_file)).unwrap().read(&mut PMEM[start..]).unwrap()
            }
        }
    }
}

pub fn init_monitor() {
    let args = Args::parse();
    init_log(args.log.as_ref());
    init_mem();
    init_device();
    let mut rv = RISCV64::new();
    rv.init_isa();
    let img_size = load_img(args.image.as_ref());
}