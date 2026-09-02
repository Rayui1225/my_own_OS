#![no_std]
#![no_main]

extern crate alloc;

mod arch;
mod console;
mod driver;
mod memory;
mod panic;
mod task;
#[cfg(feature = "test-kernel")]
mod test;

use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn kernel_main(_hart_id: usize, _dtb_pa: usize) -> ! {
    clear_bss();
    console::init();
    arch::riscv64::trap::init();
    memory::init();
    memory::paging::init().expect("kernel virtual memory initialization failed");
    memory::heap::init().expect("kernel heap initialization failed");
    task::init();

    #[cfg(feature = "test-kernel")]
    {
        test::run()
    }

    #[cfg(not(feature = "test-kernel"))]
    {
        println!("[boot] kernel entered");
        println!("[boot] arch = riscv64");
        println!("[debug] uart ready");
        println!("[debug] trap ready");
        println!("[memory] allocator = {}", memory::allocator_name());
        println!(
            "[memory] total usable frames = {}",
            memory::total_usable_frames()
        );
        println!("[memory] free frames = {}", memory::free_frame_count());
        println!(
            "[heap] allocated bytes = {}",
            memory::heap::allocated_bytes()
        );
        let task_one_id: task::TaskId = task::spawn(task_one).expect("failed to create task 1");
        let task_two_id: task::TaskId = task::spawn(task_two).expect("failed to create task 2");
        let task_three_id: task::TaskId = task::spawn(task_three).expect("failed to create task 3");
        arch::riscv64::timer::init();
        task::run();
        arch::riscv64::timer::stop();
        assert_eq!(task::state(task_one_id), Some(task::TaskState::Exited));
        assert_eq!(task::state(task_two_id), Some(task::TaskState::Exited));
        assert_eq!(task::state(task_three_id), Some(task::TaskState::Exited));
        println!("[task] all tasks exited");
        arch::riscv64::boot::wait_forever()
    }
}

#[cfg(not(feature = "test-kernel"))]
fn task_one() {
    run_cpu_bound_task(1);
}

#[cfg(not(feature = "test-kernel"))]
fn task_two() {
    run_cpu_bound_task(2);
}

#[cfg(not(feature = "test-kernel"))]
fn task_three() {
    run_cpu_bound_task(3);
}

#[cfg(not(feature = "test-kernel"))]
fn run_cpu_bound_task(id: usize) {
    const RUN_TICKS: usize = 2;

    println!("[task {}] started", id);
    let start = arch::riscv64::timer::ticks();
    while arch::riscv64::timer::ticks().wrapping_sub(start) < RUN_TICKS {
        core::hint::spin_loop();
    }
    println!("[task {}] finished", id);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::handle(info)
}

fn clear_bss() {
    extern "C" {
        static mut sbss: u8;
        static mut ebss: u8;
    }

    let start = unsafe { core::ptr::addr_of_mut!(sbss) };
    let end = unsafe { core::ptr::addr_of_mut!(ebss) };

    let mut current = start;
    while current < end {
        unsafe { current.write_volatile(0) };
        current = current.wrapping_add(1);
    }
}
