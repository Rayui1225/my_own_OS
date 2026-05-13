#![no_std]
#![no_main]

mod arch;
mod console;
mod driver;

use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn kernel_main(_hart_id: usize, _dtb_pa: usize) -> ! {
    clear_bss();
    console::init();
    println!("[boot] kernel entered");
    println!("[boot] arch = riscv64");
    println!("[debug] uart ready");
    arch::riscv64::boot::wait_forever()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[panic] {}", info);
    arch::riscv64::boot::wait_forever()
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
