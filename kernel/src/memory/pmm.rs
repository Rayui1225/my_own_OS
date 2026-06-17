#![cfg_attr(feature = "test-kernel", allow(dead_code))]

use super::{
    frame::{Frame, MemoryRange, PhysAddr, PAGE_SIZE},
    map,
};

const MAX_FRAMES: usize = (0x8800_0000usize - 0x8020_0000usize) / PAGE_SIZE;

pub trait FrameAllocator {
    fn alloc_frame(&mut self) -> Option<Frame>;
    fn dealloc_frame(&mut self, frame: Frame);
    fn total_usable_frames(&self) -> usize;
}

pub struct StackFrameAllocator {
    frames: [PhysAddr; MAX_FRAMES],
    len: usize,
    total_usable_frames: usize,
}

impl StackFrameAllocator {
    pub const fn new() -> Self {
        Self {
            frames: [0; MAX_FRAMES],
            len: 0,
            total_usable_frames: 0,
        }
    }

    fn reset(&mut self) {
        self.len = 0;
        self.total_usable_frames = 0;
    }

    fn push(&mut self, frame: Frame) {
        assert!(self.len < self.frames.len(), "frame stack overflow");
        self.frames[self.len] = frame.start_address();
        self.len += 1;
        self.total_usable_frames += 1;
    }

    fn add_range(&mut self, range: MemoryRange) {
        let aligned = range.align_inward();
        let mut current = aligned.start;

        while current < aligned.end {
            self.push(Frame::from_start_address(current));
            current += PAGE_SIZE;
        }
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn alloc_frame(&mut self) -> Option<Frame> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        Some(Frame::from_start_address(self.frames[self.len]))
    }

    fn dealloc_frame(&mut self, frame: Frame) {
        assert!(self.len < self.frames.len(), "frame stack overflow");
        self.frames[self.len] = frame.start_address();
        self.len += 1;
    }

    fn total_usable_frames(&self) -> usize {
        self.total_usable_frames
    }
}

static mut PMM: StackFrameAllocator = StackFrameAllocator::new();

pub fn init() {
    let reserved = map::kernel_reserved_range();

    unsafe {
        PMM.reset();

        for &range in map::usable_memory_ranges() {
            for segment in subtract_reserved(range, reserved) {
                if !segment.is_empty() {
                    PMM.add_range(segment);
                }
            }
        }
    }
}

pub fn alloc_frame() -> Option<Frame> {
    unsafe { PMM.alloc_frame() }
}

pub fn dealloc_frame(frame: Frame) {
    unsafe { PMM.dealloc_frame(frame) }
}

pub fn total_usable_frames() -> usize {
    unsafe { PMM.total_usable_frames() }
}

fn subtract_reserved(range: MemoryRange, reserved: MemoryRange) -> [MemoryRange; 2] {
    if reserved.end <= range.start || range.end <= reserved.start {
        return [range, MemoryRange::new(0, 0)];
    }

    let left = MemoryRange::new(range.start, core::cmp::min(range.end, reserved.start));
    let right = MemoryRange::new(core::cmp::max(range.start, reserved.end), range.end);

    [left, right]
}
