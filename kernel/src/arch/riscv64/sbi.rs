use core::arch::asm;

const SBI_EXT_TIMER: usize = 0x5449_4D45; // "TIME" in ASCII
const SBI_FID_SET_TIMER: usize = 0;

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn set_timer(deadline: u64) {
    let low = deadline as usize;

    unsafe {
        asm!(
            "ecall",
            inlateout("a0") low => _,
            inlateout("a1") 0usize => _,
            in("a6") SBI_FID_SET_TIMER,
            in("a7") SBI_EXT_TIMER,
            options(nostack)
        );
    }
}
