#![no_std]
#![no_main]

mod arch;
mod console;
mod driver;
mod memory;
mod panic;
#[cfg(feature = "test-kernel")]
mod test;

use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn kernel_main(_hart_id: usize, _dtb_pa: usize) -> ! {
    clear_bss();
    console::init();

    #[cfg(feature = "test-kernel")]
    {
        test::run()
    }

    #[cfg(not(feature = "test-kernel"))]
    {
        arch::riscv64::trap::init();
        println!("[boot] kernel entered");
        println!("[boot] arch = riscv64");
        println!("[debug] uart ready");
        println!("[debug] trap ready");
        memory::init();
        println!("[memory] total usable frames = {}", memory::total_usable_frames());
        if let Some(frame) = memory::alloc_frame() {
            println!("[memory] alloc frame = {:#x}", frame.start_address());
            memory::dealloc_frame(frame);
            println!("[memory] free frame = {:#x}", frame.start_address());
        } else {
            println!("[memory] alloc frame = none");
        }
        arch::riscv64::boot::wait_forever()
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::handle(info)
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
