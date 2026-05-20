use core::ptr::write_volatile;

use crate::arch::riscv64::boot::wait_forever;

// QEMU virt exposes the SiFive test finisher device at this MMIO address.
const SIFIVE_TEST_BASE: usize = 0x0010_0000;
const EXIT_SUCCESS: u32 = 0x5555;
const EXIT_FAILURE: u32 = 0x3333;

pub fn exit_success() -> ! {
    exit(EXIT_SUCCESS)
}

pub fn exit_failure() -> ! {
    exit(EXIT_FAILURE)
}

fn exit(code: u32) -> ! {
    unsafe {
        write_volatile(SIFIVE_TEST_BASE as *mut u32, code);
    }

    wait_forever()
}
