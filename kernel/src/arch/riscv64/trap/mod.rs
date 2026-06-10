mod frame;

use core::arch::{asm, global_asm};

use crate::println;

use super::{csr, timer};
pub use frame::{TrapCause, TrapFrame};

global_asm!(include_str!("entry.S"));

extern "C" {
    static __trap_entry: u8;
}

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn init() {
    let trap_entry = unsafe { core::ptr::addr_of!(__trap_entry) as usize };
    csr::write_stvec(trap_entry);
}

#[allow(dead_code)]
pub fn trigger_breakpoint() {
    unsafe {
        asm!(".4byte 0x00100073", options(nomem, nostack));
    }
}

#[allow(dead_code)]
pub fn trigger_illegal_instruction() {
    unsafe {
        asm!(".4byte 0xffffffff", options(nomem, nostack));
    }
}

#[no_mangle]
extern "C" fn trap_entry_rust(frame: &mut TrapFrame) {
    match frame.cause() {
        TrapCause::Breakpoint => {
            println!("[trap] cause = {}", frame.cause().description());
            println!("[trap] sepc = {:#x}", frame.sepc);
            frame.advance_sepc_by_4(); // Skip the breakpoint instruction, if we don't do this, we'll hit the same breakpoint again and again
        }
        TrapCause::SupervisorTimerInterrupt => {
            timer::handle_interrupt();
        }
        TrapCause::IllegalInstruction
        | TrapCause::LoadPageFault
        | TrapCause::StorePageFault
        | TrapCause::UserEcall
        | TrapCause::Unknown { .. } => {
            log_trap(frame);
            panic!("unhandled trap");
        }
    }
}

fn log_trap(frame: &TrapFrame) {
    println!("[trap] cause = {}", frame.cause().description());
    println!("[trap] sepc = {:#x}", frame.sepc);

    match frame.cause() {
        TrapCause::LoadPageFault | TrapCause::StorePageFault => {
            println!("[trap] fault address = {:#x}", frame.stval);
        }
        TrapCause::Unknown { interrupt, code } => {
            println!("[trap] raw scause interrupt={} code={:#x}", interrupt, code);
            println!("[trap] stval = {:#x}", frame.stval);
        }
        _ => {}
    }
}
