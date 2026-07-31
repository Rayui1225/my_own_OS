//! A small kernel heap backed by PMM frames and the kernel page table.

use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null_mut,
};

use crate::println;

use super::{
    alloc_frame,
    frame::PAGE_SIZE,
    paging::{self, MapError, PteFlags, VirtAddr},
};

pub const KERNEL_HEAP_START: VirtAddr = 0xffff_ffc0_1000_0000;
pub const KERNEL_HEAP_SIZE: usize = 1024 * 1024;

#[global_allocator]
static KERNEL_ALLOCATOR: BumpAllocator = BumpAllocator::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeapInitError {
    AlreadyInitialized,
    OutOfFrames,
    Mapping(MapError),
}

pub fn init() -> Result<(), HeapInitError> {
    if KERNEL_ALLOCATOR.is_initialized() {
        return Err(HeapInitError::AlreadyInitialized);
    }

    for virt in (KERNEL_HEAP_START..KERNEL_HEAP_START + KERNEL_HEAP_SIZE).step_by(PAGE_SIZE) {
        let frame = alloc_frame().ok_or(HeapInitError::OutOfFrames)?;
        paging::map_kernel_page(virt, frame.start_address(), PteFlags::READ | PteFlags::WRITE)
            .map_err(HeapInitError::Mapping)?;
    }

    KERNEL_ALLOCATOR.initialize(KERNEL_HEAP_START, KERNEL_HEAP_SIZE);
    println!(
        "[heap] initialized at {:#x}, size = {} KiB",
        KERNEL_HEAP_START,
        KERNEL_HEAP_SIZE / 1024
    );
    Ok(())
}

#[cfg_attr(feature = "test-kernel", allow(dead_code))]
pub fn allocated_bytes() -> usize {
    KERNEL_ALLOCATOR.allocated_bytes()
}

struct BumpAllocator {
    state: UnsafeCell<BumpState>,
}

struct BumpState {
    start: VirtAddr,
    end: VirtAddr,
    next: VirtAddr,
}

impl BumpAllocator {
    const fn new() -> Self {
        Self {
            state: UnsafeCell::new(BumpState {
                start: 0,
                end: 0,
                next: 0,
            }),
        }
    }

    fn is_initialized(&self) -> bool {
        unsafe { (*self.state.get()).start != 0 }
    }

    fn initialize(&self, start: VirtAddr, size: usize) {
        let state = unsafe { &mut *self.state.get() };
        state.start = start;
        state.end = start + size;
        state.next = start;
    }

    fn allocated_bytes(&self) -> usize {
        let state = unsafe { &*self.state.get() };
        state.next.saturating_sub(state.start)
    }
}

// Milestone 7 runs on one hart and keeps timer interrupts disabled. Later scheduler work must
// replace this with interrupt-safe synchronization before allocations can be concurrent.
unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let state = &mut *self.state.get();
        if state.start == 0 {
            return null_mut();
        }

        let Some(allocation_start) = align_up(state.next, layout.align()) else {
            return null_mut();
        };
        let Some(allocation_end) = allocation_start.checked_add(layout.size()) else {
            return null_mut();
        };
        if allocation_end > state.end {
            return null_mut();
        }

        state.next = allocation_end;
        allocation_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // A bump allocator reclaims all memory together only when its whole heap is reset.
    }
}

fn align_up(addr: usize, align: usize) -> Option<usize> {
    debug_assert!(align.is_power_of_two());
    addr.checked_add(align - 1).map(|value| value & !(align - 1))
}
