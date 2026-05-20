use core::panic::PanicInfo;

use crate::println;

#[cfg(feature = "test-kernel")]
use crate::driver::qemu;
#[cfg(not(feature = "test-kernel"))]
use crate::arch::riscv64::boot::wait_forever;

pub fn handle(info: &PanicInfo) -> ! {
    println!("[panic] {}", info);

    #[cfg(feature = "test-kernel")]
    {
        qemu::exit_failure()
    }

    #[cfg(not(feature = "test-kernel"))]
    wait_forever()
}
