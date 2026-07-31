#![no_std]
#![no_main]

extern crate alloc;

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
    arch::riscv64::trap::init();
    memory::init();
    memory::paging::init().expect("kernel virtual memory initialization failed");
    memory::heap::init().expect("kernel heap initialization failed");

    #[cfg(feature = "test-kernel")]
    {
        test::run()
    }

    #[cfg(not(feature = "test-kernel"))]
    {
        println!("[boot] kernel entered");
        println!("[boot] arch = riscv64");
        println!("[debug] uart ready");
        println!("[debug] trap ready");
        println!("[memory] allocator = {}", memory::allocator_name());
        println!(
            "[memory] total usable frames = {}",
            memory::total_usable_frames()
        );
        println!("[memory] free frames = {}", memory::free_frame_count());
        println!("[heap] allocated bytes = {}", memory::heap::allocated_bytes());
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
