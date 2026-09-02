mod registry;
mod scheduler;
mod thread;

pub(crate) use scheduler::on_timer_tick;
pub use scheduler::{init, run, spawn, state};
#[cfg_attr(not(feature = "test-kernel"), allow(unused_imports))]
pub use scheduler::yield_now;
#[cfg_attr(feature = "test-kernel", allow(unused_imports))]
pub use thread::TaskId;
pub use thread::TaskState;

use thread::TaskEntry;

#[no_mangle]
extern "C" fn task_entry_rust(entry_address: usize) -> ! {
    // A first-run task has no suspended InterruptGuard to restore SIE.
    crate::arch::riscv64::csr::enable_supervisor_interrupts();
    let entry: TaskEntry = unsafe { core::mem::transmute(entry_address) };
    entry();
    scheduler::exit_current()
}
