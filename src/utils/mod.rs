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

#[cfg(test)]
pub mod tests {
    use crate::device::Devices;
    use crate::isa::riscv64::RISCV64;
    use crate::isa::Isa;
    use crate::memory::Memory;
    use crate::Emulator;

    pub fn fake_emulator() -> Emulator<RISCV64> {
        let mut memory = Memory::new(); // init mem
        let device = Devices::new(&mut memory, false); // init device
        let cpu = RISCV64::new(memory);
        Emulator::<RISCV64> {
            cpu,
            device,
            difftest_ctx: None,
            batch: false,
            exitcode: 0,
        }
    }
}
