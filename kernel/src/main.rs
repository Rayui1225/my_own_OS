#![no_std]
#![no_main]

extern crate alloc;

mod arch;
mod console;
mod driver;
mod loader;
mod memory;
mod panic;
mod process;
mod syscall;
mod task;
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
    task::init();

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
        println!(
            "[heap] allocated bytes = {}",
            memory::heap::allocated_bytes()
        );
        verify_ramdisk();
        let free_frames_before_process = memory::free_frame_count();
        println!("[process] loading init");
        let init_task = process::spawn_init().expect("failed to create init process");
        arch::riscv64::timer::init();
        task::run();
        arch::riscv64::timer::stop();
        assert_eq!(task::state(init_task), Some(task::TaskState::Exited));
        assert_eq!(process::reap_init(), Ok(0));
        assert_eq!(memory::free_frame_count(), free_frames_before_process);
        arch::riscv64::boot::wait_forever()
    }
}

#[cfg(not(feature = "test-kernel"))]
fn verify_ramdisk() {
    use driver::block::{Block, BlockDevice, BlockId, RamDisk, BLOCK_SIZE};

    let mut storage = [0u8; BLOCK_SIZE * 4];
    let mut disk = RamDisk::new(&mut storage).expect("failed to create ramdisk");
    let input: Block = [0x5a; BLOCK_SIZE];
    let mut output: Block = [0; BLOCK_SIZE];

    println!("[block] device = ramdisk");
    println!("[block] block size = {}", BLOCK_SIZE);
    println!("[block] block count = {}", disk.block_count());
    println!("[block] write block 1");
    disk.write_block(BlockId::new(1), &input)
        .expect("ramdisk write failed");
    println!("[block] read block 1");
    disk.read_block(BlockId::new(1), &mut output)
        .expect("ramdisk read failed");
    assert_eq!(output, input);
    println!("[block] verify ok");
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
