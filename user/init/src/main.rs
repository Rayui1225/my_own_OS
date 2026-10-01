#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo, ptr::read_volatile};

const SYS_WRITE: usize = 1;
const SYS_EXIT: usize = 2;
const SYS_YIELD: usize = 3;
const SYS_GETPID: usize = 4;

static MESSAGE: &[u8] = b"[user] hello from loaded Rust program\n";
static mut DATA_PROBE: usize = 7;
static mut BSS_PROBE: usize = 0;

#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() -> ! {
    if unsafe { read_volatile(core::ptr::addr_of!(DATA_PROBE)) } != 7 {
        exit(20);
    }
    if unsafe { read_volatile(core::ptr::addr_of!(BSS_PROBE)) } != 0 {
        exit(21);
    }
    if syscall3(SYS_WRITE, 1, MESSAGE.as_ptr() as usize, MESSAGE.len()) != MESSAGE.len() as isize {
        exit(22);
    }
    if syscall3(SYS_WRITE, 1, 0x8020_0000, 1) != -14 {
        exit(23);
    }
    if syscall0(SYS_GETPID) != 1 {
        exit(24);
    }
    if syscall0(SYS_YIELD) != 0 {
        exit(25);
    }
    exit(0)
}

fn syscall0(number: usize) -> isize {
    syscall3(number, 0, 0, 0)
}

fn syscall3(number: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let mut result = arg0;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") result,
            in("a1") arg1,
            in("a2") arg2,
            in("a7") number,
            options(nostack),
        );
    }
    result as isize
}

fn exit(code: usize) -> ! {
    let _ = syscall3(SYS_EXIT, code, 0, 0);
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(99)
}
