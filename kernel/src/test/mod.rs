#[cfg(feature = "test-kernel")]
use crate::driver::qemu;
use crate::{print, println};

pub struct TestCase {
    pub name: &'static str,
    pub run: fn(),
}

pub fn run() -> ! {
    let tests = [trivial_assertion(), console_print()];
    run_tests(&tests)
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
