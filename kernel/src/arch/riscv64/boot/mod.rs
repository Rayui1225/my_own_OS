use core::arch::{asm, global_asm};

global_asm!(include_str!("entry.S"));

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        unsafe {
            asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

