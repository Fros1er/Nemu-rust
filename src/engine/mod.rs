use crate::isa::{CPUState, Isa};
use crate::memory::vaddr::VAddr;
use crate::monitor::sdb::sdb_loop;

pub fn engine_start<U: CPUState, T: Isa<U>>(mut isa: T) {
    sdb_loop(&mut isa);
}

pub enum EmuState {
    RUNNING, STOP, END, ABORT, QUIT
}

pub fn set_emu_state(state: EmuState, pc: VAddr) {

}