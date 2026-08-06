mod scheduler;
mod thread;

pub use scheduler::{init, run, spawn, state, yield_now};
#[cfg_attr(feature = "test-kernel", allow(unused_imports))]
pub use thread::TaskId;
pub use thread::TaskState;

use thread::TaskEntry;

#[no_mangle]
extern "C" fn task_entry_rust(entry_address: usize) -> ! {
    let entry: TaskEntry = unsafe { core::mem::transmute(entry_address) };
    entry();
    scheduler::exit_current()
}
