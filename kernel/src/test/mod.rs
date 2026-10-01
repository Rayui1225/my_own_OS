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
        simplebin_rejects_bad_magic(),
        program_loader_maps_segments(),
        cooperative_and_preemptive_task_switch(),
        user_mode_syscalls(),
    ];
    run_tests(&tests)
}

fn simplebin_rejects_bad_magic() -> TestCase {
    TestCase {
        name: "simplebin_rejects_bad_magic",
        run: || {
            let image = simplebin_image(&[0x13, 0, 0, 0], &[], 0);
            let mut invalid = image;
            invalid[0] = b'X';
            assert!(matches!(
                crate::loader::load(&invalid),
                Err(crate::loader::LoadError::Parse(
                    crate::loader::ParseError::InvalidMagic
                ))
            ));
        },
    }
}

fn program_loader_maps_segments() -> TestCase {
    TestCase {
        name: "program_loader_maps_segments",
        run: || {
            let mut text = Vec::from([0x13; crate::memory::PAGE_SIZE + 1]);
            text[crate::memory::PAGE_SIZE] = 0x73;
            let data = [0xaa, 0xbb, 0xcc];
            let image = simplebin_image(&text, &data, crate::memory::PAGE_SIZE);

            let free_before = crate::memory::free_frame_count();
            let program = crate::loader::load(&image).expect("failed to load SimpleBin test image");
            let address_space = program.address_space();
            let text_base = crate::loader::USER_TEXT_BASE;
            let data_base = (text_base + text.len() + crate::memory::PAGE_SIZE - 1)
                & !(crate::memory::PAGE_SIZE - 1);
            assert_eq!(
                address_space.read_user_byte(text_base + crate::memory::PAGE_SIZE),
                Some(0x73)
            );
            assert_eq!(address_space.read_user_byte(data_base + 1), Some(0xbb));
            assert_eq!(
                address_space.read_user_byte(data_base + data.len()),
                Some(0)
            );
            assert!(address_space.is_user_executable(text_base));
            assert!(!address_space.is_user_writable(text_base));
            assert!(address_space.is_user_writable(data_base));
            assert!(!address_space.is_user_executable(data_base));
            assert_eq!(address_space.read_user_byte(0x8020_0000), None);
            program.destroy();
            assert_eq!(crate::memory::free_frame_count(), free_before);
        },
    }
}

fn simplebin_image(text: &[u8], data: &[u8], bss_size: usize) -> Vec<u8> {
    let mut image = Vec::with_capacity(32 + text.len() + data.len());
    image.extend_from_slice(b"SBIN");
    image.extend_from_slice(&1u16.to_le_bytes());
    image.extend_from_slice(&32u16.to_le_bytes());
    image.extend_from_slice(&(crate::loader::USER_TEXT_BASE as u64).to_le_bytes());
    image.extend_from_slice(&(text.len() as u32).to_le_bytes());
    image.extend_from_slice(&(data.len() as u32).to_le_bytes());
    image.extend_from_slice(&(bss_size as u32).to_le_bytes());
    image.extend_from_slice(&0u32.to_le_bytes());
    image.extend_from_slice(text);
    image.extend_from_slice(data);
    image
}

fn user_mode_syscalls() -> TestCase {
    TestCase {
        name: "user_mode_syscalls",
        run: || {
            crate::task::reset_for_test();
            crate::task::init();

            let free_before = crate::memory::free_frame_count();
            println!("[process] loading init");
            let init_task = crate::process::spawn_init().expect("failed to create init process");
            crate::arch::riscv64::timer::init();
            crate::task::run();
            crate::arch::riscv64::timer::stop();

            assert_eq!(
                crate::task::state(init_task),
                Some(crate::task::TaskState::Exited)
            );
            assert_eq!(crate::process::reap_init(), Ok(0));
            assert_eq!(crate::memory::free_frame_count(), free_before);
        },
    }
}

static TASK_STEP: AtomicUsize = AtomicUsize::new(0);
static PREEMPT_RELEASED: AtomicBool = AtomicBool::new(false);
static SYSCALL_PID: AtomicUsize = AtomicUsize::new(0);

fn cooperative_and_preemptive_task_switch() -> TestCase {
    TestCase {
        name: "cooperative_and_preemptive_task_switch",
        run: || {
            TASK_STEP.store(0, Ordering::SeqCst);
            PREEMPT_RELEASED.store(false, Ordering::Relaxed);
            SYSCALL_PID.store(0, Ordering::Relaxed);
            let task_one = crate::task::spawn(test_task_one).expect("failed to create test task 1");
            let task_two = crate::task::spawn(test_task_two).expect("failed to create test task 2");
            let preempt_waiter =
                crate::task::spawn(test_preempt_waiter).expect("failed to create preempt waiter");
            let preempt_releaser = crate::task::spawn(test_preempt_releaser)
                .expect("failed to create preempt releaser");
            let syscall_task =
                crate::task::spawn(test_syscalls).expect("failed to create syscall task");
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
            assert_eq!(SYSCALL_PID.load(Ordering::Relaxed), syscall_task.value());
            assert_eq!(crate::task::exit_code(syscall_task), Some(7));
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

fn test_syscalls() {
    const USER_BUFFER: usize = 0x1000;
    const MESSAGE: &[u8] = b"[user] hello from syscall test\n";

    let memory = crate::syscall::SliceUserMemory::new(USER_BUFFER, MESSAGE);
    let mut frame = crate::arch::riscv64::trap::TrapFrame {
        sepc: 0x2000,
        ..Default::default()
    };

    frame.a7 = crate::syscall::SYS_WRITE;
    frame.a0 = 1;
    frame.a1 = USER_BUFFER;
    frame.a2 = MESSAGE.len();
    crate::syscall::handle_with_memory(&mut frame, &memory);
    assert_eq!(frame.a0, MESSAGE.len());
    assert_eq!(frame.sepc, 0x2004);

    frame.a7 = crate::syscall::SYS_WRITE;
    frame.a0 = 99;
    crate::syscall::handle_with_memory(&mut frame, &memory);
    assert_eq!(
        frame.a0 as isize,
        -(crate::syscall::Errno::BadFileDescriptor as isize)
    );

    frame.a7 = crate::syscall::SYS_WRITE;
    frame.a0 = 1;
    frame.a1 = USER_BUFFER + MESSAGE.len();
    frame.a2 = 1;
    crate::syscall::handle_with_memory(&mut frame, &memory);
    assert_eq!(frame.a0 as isize, -(crate::syscall::Errno::Fault as isize));

    frame.a7 = usize::MAX;
    crate::syscall::handle_with_memory(&mut frame, &memory);
    assert_eq!(frame.a0 as isize, -(crate::syscall::Errno::NoSys as isize));

    frame.a7 = crate::syscall::SYS_GETPID;
    crate::syscall::handle_with_memory(&mut frame, &memory);
    SYSCALL_PID.store(frame.a0, Ordering::Relaxed);

    frame.a7 = crate::syscall::SYS_YIELD;
    crate::syscall::handle_with_memory(&mut frame, &memory);
    assert_eq!(frame.a0, 0);

    frame.a7 = crate::syscall::SYS_EXIT;
    frame.a0 = 7;
    crate::syscall::handle_with_memory(&mut frame, &memory);
    unreachable!("sys_exit returned");
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
