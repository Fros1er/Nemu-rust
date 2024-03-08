pub mod configs;
pub mod disasm;

#[cfg(test)]
pub mod tests {
    use crate::device::Devices;
    use crate::isa::riscv64::RISCV64;
    use crate::isa::Isa;
    use crate::memory::Memory;
    use crate::Emulator;
    use std::cell::RefCell;
    use std::ops::DerefMut;
    use std::rc::Rc;

    pub fn fake_emulator() -> Emulator<RISCV64> {
        let memory = Rc::new(RefCell::new(Memory::new())); // init mem
        let device = Devices::new(memory.borrow_mut().deref_mut()); // init device
        let cpu = RISCV64::new(memory.clone());
        Emulator::<RISCV64> {
            cpu,
            memory,
            device: device,
            difftest_ctx: None,
            batch: false
        }
    }
}
