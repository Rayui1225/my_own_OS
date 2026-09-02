use alloc::{boxed::Box, string::String, vec::Vec};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[cfg(feature = "test-kernel")]
use crate::driver::qemu;
use crate::{print, println};

pub struct TestCase {
    pub name: &'static str,
    pub run: fn(),
}

pub fn run() -> ! {
    let tests = [
        trivial_assertion(),
        console_print(),
        heap_box(),
        heap_vec(),
        heap_string(),
        cooperative_and_preemptive_task_switch(),
    ];
    run_tests(&tests)
}

static TASK_STEP: AtomicUsize = AtomicUsize::new(0);
static PREEMPT_RELEASED: AtomicBool = AtomicBool::new(false);

fn cooperative_and_preemptive_task_switch() -> TestCase {
    TestCase {
        name: "cooperative_and_preemptive_task_switch",
        run: || {
            TASK_STEP.store(0, Ordering::SeqCst);
            PREEMPT_RELEASED.store(false, Ordering::Relaxed);
            let task_one = crate::task::spawn(test_task_one).expect("failed to create test task 1");
            let task_two = crate::task::spawn(test_task_two).expect("failed to create test task 2");
            let preempt_waiter =
                crate::task::spawn(test_preempt_waiter).expect("failed to create preempt waiter");
            let preempt_releaser = crate::task::spawn(test_preempt_releaser)
                .expect("failed to create preempt releaser");
            crate::arch::riscv64::timer::init();
            crate::task::run();
            crate::arch::riscv64::timer::stop();
            assert_eq!(TASK_STEP.load(Ordering::SeqCst), 4);
            assert!(PREEMPT_RELEASED.load(Ordering::Acquire));
            assert_eq!(
                crate::task::state(task_one),
                Some(crate::task::TaskState::Exited)
            );
            assert_eq!(
                crate::task::state(preempt_waiter),
                Some(crate::task::TaskState::Exited)
            );
            assert_eq!(
                crate::task::state(preempt_releaser),
                Some(crate::task::TaskState::Exited)
            );
            assert_eq!(
                crate::task::state(task_two),
                Some(crate::task::TaskState::Exited)
            );
        },
    }
}

fn test_task_one() {
    assert_eq!(TASK_STEP.fetch_add(1, Ordering::SeqCst), 0);
    crate::task::yield_now();
    assert_eq!(TASK_STEP.fetch_add(1, Ordering::SeqCst), 2);
}

fn test_task_two() {
    assert_eq!(TASK_STEP.fetch_add(1, Ordering::SeqCst), 1);
    crate::task::yield_now();
    assert_eq!(TASK_STEP.fetch_add(1, Ordering::SeqCst), 3);
}

fn test_preempt_waiter() {
    while !PREEMPT_RELEASED.load(Ordering::Acquire) {
        core::hint::spin_loop();
    }
}

fn test_preempt_releaser() {
    PREEMPT_RELEASED.store(true, Ordering::Release);
}

fn heap_box() -> TestCase {
    TestCase {
        name: "heap_box",
        run: || {
            let value = Box::new(42usize);
            assert_eq!(*value, 42);
        },
    }
}

fn heap_vec() -> TestCase {
    TestCase {
        name: "heap_vec",
        run: || {
            let mut values = Vec::new();
            values.push(1);
            values.push(2);
            assert_eq!(values, [1, 2]);
        },
    }
}

fn heap_string() -> TestCase {
    TestCase {
        name: "heap_string",
        run: || {
            let message = String::from("kernel heap is ready");
            assert_eq!(message.as_str(), "kernel heap is ready");
        },
    }
}

fn run_tests(tests: &[TestCase]) -> ! {
    println!("running {} tests", tests.len());

    for test in tests {
        print!("{} ... ", test.name);
        (test.run)();
        println!("ok");
    }

    #[cfg(feature = "test-kernel")]
    {
        qemu::exit_success()
    }

    #[cfg(not(feature = "test-kernel"))]
    crate::arch::riscv64::boot::wait_forever()
}

fn trivial_assertion() -> TestCase {
    TestCase {
        name: "trivial_assertion",
        run: || {
            assert_eq!(1, 1);
        },
    }
}

fn console_print() -> TestCase {
    TestCase {
        name: "console_print",
        run: || {
            println!("test output from console_print");
        },
    }
}
