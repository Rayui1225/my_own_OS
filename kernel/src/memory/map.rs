#![cfg_attr(feature = "test-kernel", allow(dead_code))]

use super::frame::{align_up_to_page, MemoryRange};

const USABLE_MEMORY_RANGES: [MemoryRange; 1] = [MemoryRange::new(0x8020_0000, 0x8800_0000)];

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
