#![allow(dead_code)]

use super::FrameAllocator;
use crate::memory::{
    frame::{Frame, MemoryRange, PhysAddr, PAGE_SIZE},
    map,
};

const MAX_FRAMES: usize = map::MAX_MANAGED_FRAMES;

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

    fn push_discovered_frame(&mut self, frame: Frame) {
        self.push(frame);
        self.total_usable_frames += 1;
    }

    fn push(&mut self, frame: Frame) {
        assert!(self.len < self.frames.len(), "frame stack overflow");
        self.frames[self.len] = frame.start_address();
        self.len += 1;
    }

    fn add_range(&mut self, range: MemoryRange) {
        let aligned = range.align_inward();
        let mut current = aligned.start;

        while current < aligned.end {
            self.push_discovered_frame(Frame::from_start_address(current));
            current += PAGE_SIZE;
        }
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn init(&mut self, usable_segments: &[MemoryRange]) {
        self.reset();

        for &segment in usable_segments {
            if !segment.is_empty() {
                self.add_range(segment);
            }
        }
    }

    fn alloc_frame(&mut self) -> Option<Frame> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        Some(Frame::from_start_address(self.frames[self.len]))
    }

    fn dealloc_frame(&mut self, frame: Frame) {
        self.push(frame);
    }

    fn total_usable_frames(&self) -> usize {
        self.total_usable_frames
    }

    fn free_frame_count(&self) -> usize {
        self.len
    }

    fn name(&self) -> &'static str {
        "stack"
    }
}
