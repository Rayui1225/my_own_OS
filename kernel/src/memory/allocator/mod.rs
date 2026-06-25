pub mod bitmap;
pub mod stack;

use super::frame::{Frame, MemoryRange};

pub trait FrameAllocator {
    fn init(&mut self, usable_segments: &[MemoryRange]);
    fn alloc_frame(&mut self) -> Option<Frame>;
    fn dealloc_frame(&mut self, frame: Frame);
    fn total_usable_frames(&self) -> usize;
    fn free_frame_count(&self) -> usize;
    fn name(&self) -> &'static str;
}
