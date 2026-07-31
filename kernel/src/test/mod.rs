use alloc::{boxed::Box, string::String, vec::Vec};

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
    ];
    run_tests(&tests)
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
