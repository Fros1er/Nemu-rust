pub mod configs;
pub mod disasm;

macro_rules! cfg_if_feat {
    ($feature:literal, $({ $($tokens:tt)* })?) => {
        cfg_if! {
            if #[cfg(feature = $feature)] {
                $($($tokens)*)?
            }
        }
    };
}

pub(crate) use cfg_if_feat;

// #[cfg(test)]
// pub mod tests {
//     use crate::device::Devices;
//     use crate::isa::riscv64::RISCV64;
//     use crate::isa::Isa;
//     use crate::memory::Memory;
//     use crate::monitor::Args;
//     use crate::Emulator;
//     use std::sync::atomic::AtomicBool;
//     use std::sync::Arc;
//
//     pub fn fake_emulator() -> Emulator<RISCV64> {
//         let mut memory = Memory::new(); // init mem
//         let stopped = Arc::new(AtomicBool::new(false));
//         let device = Devices::new(stopped.clone(), &mut memory, false); // init device
//         let cpu = RISCV64::new(
//             stopped,
//             memory,
//             &Args {
//                 ..Default::default()
//             },
//         );
//         Emulator::<RISCV64> {
//             cpu,
//             device,
//             difftest_ctx: None,
//             batch: false,
//             exitcode: 0,
//         }
//     }
// }
