use super::FrameAllocator;
use crate::memory::{
    frame::{Frame, MemoryRange, PhysAddr, PAGE_SIZE},
    map,
};

const WORD_BITS: usize = usize::BITS as usize;
const BITMAP_WORDS: usize = map::MAX_MANAGED_FRAMES.div_ceil(WORD_BITS);

pub struct BitmapFrameAllocator {
    base: PhysAddr,
    total_frames: usize,
    free_frames: usize,
    bitmap: [usize; BITMAP_WORDS],
}

impl BitmapFrameAllocator {
    pub const fn new() -> Self {
        Self {
            base: 0,
            total_frames: 0,
            free_frames: 0,
            bitmap: [usize::MAX; BITMAP_WORDS],
        }
    }

    fn reset(&mut self) {
        self.base = 0;
        self.total_frames = 0;
        self.free_frames = 0;
        self.bitmap.fill(usize::MAX);
    }

    fn configure_from_first_segment(&mut self, usable_segments: &[MemoryRange]) {
        let Some(segment) = first_non_empty_segment(usable_segments) else {
            return;
        };

        let aligned = segment.align_inward();
        self.base = aligned.start;
        self.total_frames = aligned.size() / PAGE_SIZE;
        self.free_frames = self.total_frames;

        let words_to_clear = self.total_frames.div_ceil(WORD_BITS);
        for word in self.bitmap.iter_mut().take(words_to_clear) {
            *word = 0;
        }

        self.mark_unused_tail_as_used();
    }

    fn mark_unused_tail_as_used(&mut self) {
        let used_bits = self.total_frames % WORD_BITS;
        if used_bits == 0 || self.total_frames == 0 {
            return;
        }

        let last_word = self.total_frames / WORD_BITS;
        let valid_mask = (1usize << used_bits) - 1;
        self.bitmap[last_word] = !valid_mask;
    }

    fn is_used(&self, frame_index: usize) -> bool {
        let word = frame_index / WORD_BITS;
        let bit = frame_index % WORD_BITS;
        (self.bitmap[word] & (1usize << bit)) != 0
    }

    fn set_used(&mut self, frame_index: usize) {
        let word = frame_index / WORD_BITS;
        let bit = frame_index % WORD_BITS;
        self.bitmap[word] |= 1usize << bit;
    }

    fn set_free(&mut self, frame_index: usize) {
        let word = frame_index / WORD_BITS;
        let bit = frame_index % WORD_BITS;
        self.bitmap[word] &= !(1usize << bit);
    }

    fn frame_index(&self, frame: Frame) -> Option<usize> {
        let addr = frame.start_address();
        if addr < self.base || (addr - self.base) % PAGE_SIZE != 0 {
            return None;
        }

        let index = (addr - self.base) / PAGE_SIZE;
        (index < self.total_frames).then_some(index)
    }
}

impl FrameAllocator for BitmapFrameAllocator {
    fn init(&mut self, usable_segments: &[MemoryRange]) {
        self.reset();
        self.configure_from_first_segment(usable_segments);
    }

    fn alloc_frame(&mut self) -> Option<Frame> {
        if self.free_frames == 0 {
            return None;
        }

        for index in 0..self.total_frames {
            if !self.is_used(index) {
                self.set_used(index);
                self.free_frames -= 1;
                return Some(Frame::from_start_address(self.base + index * PAGE_SIZE));
            }
        }

        None
    }

    fn dealloc_frame(&mut self, frame: Frame) {
        let index = self
            .frame_index(frame)
            .expect("frame does not belong to bitmap allocator");
        assert!(self.is_used(index), "double free detected");

        self.set_free(index);
        self.free_frames += 1;
    }

    fn total_usable_frames(&self) -> usize {
        self.total_frames
    }

    fn free_frame_count(&self) -> usize {
        self.free_frames
    }

    fn name(&self) -> &'static str {
        "bitmap"
    }
}

fn first_non_empty_segment(segments: &[MemoryRange]) -> Option<MemoryRange> {
    segments
        .iter()
        .copied()
        .map(MemoryRange::align_inward)
        .find(|segment| !segment.is_empty())
}
