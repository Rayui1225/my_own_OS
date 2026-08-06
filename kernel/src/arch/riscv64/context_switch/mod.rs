//! RISC-V cooperative context switching.

use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

/// Registers that a normal function call must preserve under the RISC-V ABI.
#[repr(C)]
#[derive(Default)]
pub struct Context {
    pub ra: usize,
    pub sp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

extern "C" {
    fn __switch_context(current: *mut Context, next: *const Context);
    fn __task_entry() -> !;
}

pub fn task_entry_address() -> usize {
    __task_entry as usize
}

/// Saves `current`, restores `next`, and returns on the restored stack.
///
/// # Safety
///
/// Both pointers must remain valid for the whole switch. `next` must contain a
/// valid stack pointer and return address prepared by the scheduler.
pub unsafe fn switch_context(current: *mut Context, next: *const Context) {
    __switch_context(current, next);
}
