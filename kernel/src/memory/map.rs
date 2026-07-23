#![cfg_attr(feature = "test-kernel", allow(dead_code))]

use super::frame::{align_up_to_page, MemoryRange};

pub const RAM_START: usize = 0x8020_0000;
pub const RAM_END: usize = 0x8800_0000;
pub const UART_BASE: usize = 0x1000_0000;

const USABLE_MEMORY_RANGES: [MemoryRange; 1] = [MemoryRange::new(RAM_START, RAM_END)];
pub const MAX_MANAGED_FRAMES: usize = (RAM_END - RAM_START) / super::frame::PAGE_SIZE;

pub fn usable_memory_ranges() -> &'static [MemoryRange] {
    &USABLE_MEMORY_RANGES
}

pub fn kernel_reserved_range() -> MemoryRange {
    extern "C" {
        static skernel: u8;
        static ekernel: u8;
    }

    let start = unsafe { core::ptr::addr_of!(skernel) as usize };
    let end = unsafe { core::ptr::addr_of!(ekernel) as usize };

    MemoryRange::new(start, align_up_to_page(end))
}

pub fn kernel_sections() -> KernelSections {
    extern "C" {
        static stext: u8;
        static etext: u8;
        static srodata: u8;
        static erodata: u8;
        static sdata: u8;
        static edata: u8;
        static sboot_stack: u8;
        static eboot_stack: u8;
        static sbss: u8;
        static ebss: u8;
    }

    KernelSections {
        text: linker_range(unsafe { core::ptr::addr_of!(stext) }, unsafe {
            core::ptr::addr_of!(etext)
        }),
        rodata: linker_range(unsafe { core::ptr::addr_of!(srodata) }, unsafe {
            core::ptr::addr_of!(erodata)
        }),
        data: linker_range(unsafe { core::ptr::addr_of!(sdata) }, unsafe {
            core::ptr::addr_of!(edata)
        }),
        boot_stack: linker_range(unsafe { core::ptr::addr_of!(sboot_stack) }, unsafe {
            core::ptr::addr_of!(eboot_stack)
        }),
        bss: linker_range(unsafe { core::ptr::addr_of!(sbss) }, unsafe {
            core::ptr::addr_of!(ebss)
        }),
    }
}

#[derive(Clone, Copy)]
pub struct KernelSections {
    pub text: MemoryRange,
    pub rodata: MemoryRange,
    pub data: MemoryRange,
    pub boot_stack: MemoryRange,
    pub bss: MemoryRange,
}

fn linker_range(start: *const u8, end: *const u8) -> MemoryRange {
    MemoryRange::new(start as usize, align_up_to_page(end as usize))
}
