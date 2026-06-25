#![cfg_attr(feature = "test-kernel", allow(dead_code))]

use super::{
    allocator::{bitmap::BitmapFrameAllocator, FrameAllocator},
    frame::MemoryRange,
    map,
};

type SelectedFrameAllocator = BitmapFrameAllocator;

static mut PMM: SelectedFrameAllocator = SelectedFrameAllocator::new();

pub fn init() {
    let usable_segments = usable_segments_excluding_kernel();

    unsafe {
        PMM.init(&usable_segments);
    }
}

pub fn alloc_frame() -> Option<super::Frame> {
    unsafe { PMM.alloc_frame() }
}

pub fn dealloc_frame(frame: super::Frame) {
    unsafe { PMM.dealloc_frame(frame) }
}

pub fn total_usable_frames() -> usize {
    unsafe { PMM.total_usable_frames() }
}

pub fn free_frame_count() -> usize {
    unsafe { PMM.free_frame_count() }
}

pub fn allocator_name() -> &'static str {
    unsafe { PMM.name() }
}

fn usable_segments_excluding_kernel() -> [MemoryRange; 2] {
    let reserved = map::kernel_reserved_range();
    let mut segments = [MemoryRange::new(0, 0); 2];
    let mut len = 0;

    for &range in map::usable_memory_ranges() {
        for segment in subtract_reserved(range, reserved) {
            if !segment.is_empty() {
                assert!(len < segments.len(), "too many usable memory segments");
                segments[len] = segment;
                len += 1;
            }
        }
    }

    segments
}

fn subtract_reserved(range: MemoryRange, reserved: MemoryRange) -> [MemoryRange; 2] {
    if reserved.end <= range.start || range.end <= reserved.start {
        return [range, MemoryRange::new(0, 0)];
    }

    let left = MemoryRange::new(range.start, core::cmp::min(range.end, reserved.start));
    let right = MemoryRange::new(core::cmp::max(range.start, reserved.end), range.end);

    [left, right]
}
