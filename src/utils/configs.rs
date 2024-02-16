use std::ops::Add;
use crate::memory::paddr::{PAddr, PAddrDiff};

pub const CONFIG_MSIZE: PAddrDiff = PAddrDiff::new(0x8000000);
pub const CONFIG_MBASE: PAddr = PAddr::new(0x80000000);
pub const CONFIG_PC_RESET_OFFSET: PAddrDiff = PAddrDiff::new(0);

// use config::{Config, File};
// use lazy_static::lazy_static;
// use serde::Deserialize;
//
// #[derive(Debug, Deserialize)]
// #[allow(unused)]
// pub struct Configs {
//     pub(crate) mem_random: bool
//     pub(crate) mem
// }
//
// lazy_static! {
//     pub static ref CONFIG: Configs = {
//         Configs::new()
//     };
// }
//
// impl Configs {
//     pub fn new() -> Self {
//         let s = Config::builder()
//             .add_source(File::with_name("config"))
//             .build().unwrap();
//         s.try_deserialize().unwrap()
//     }
// }