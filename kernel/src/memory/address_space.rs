//! A process-owned Sv39 address space assembled from loader-defined segments.

use alloc::vec::Vec;
use core::ptr::{copy_nonoverlapping, read_volatile, write_bytes};

use super::{
    alloc_frame, dealloc_frame,
    paging::{MapError, PageTable, PteFlags, VirtAddr},
    Frame, PAGE_SIZE,
};
use crate::arch::riscv64::csr;

pub const USER_STACK_TOP: VirtAddr = 0x4000_0000;
const USER_STACK_BASE: VirtAddr = USER_STACK_TOP - PAGE_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressSpaceError {
    InvalidSegment,
    OutOfFrames,
    Mapping(MapError),
}

pub struct AddressSpace {
    page_table: PageTable,
    owned_frames: Vec<Frame>,
}

impl AddressSpace {
    pub fn new() -> Result<Self, AddressSpaceError> {
        let page_table = PageTable::new_user().map_err(AddressSpaceError::Mapping)?;
        Ok(Self {
            page_table,
            owned_frames: Vec::new(),
        })
    }

    pub fn map_segment(
        &mut self,
        base: VirtAddr,
        file_data: &[u8],
        memory_size: usize,
        flags: PteFlags,
    ) -> Result<(), AddressSpaceError> {
        if base & (PAGE_SIZE - 1) != 0
            || memory_size == 0
            || file_data.len() > memory_size
            || base.checked_add(memory_size).is_none()
        {
            return Err(AddressSpaceError::InvalidSegment);
        }

        let page_count = memory_size
            .checked_add(PAGE_SIZE - 1)
            .ok_or(AddressSpaceError::InvalidSegment)?
            / PAGE_SIZE;

        for page_index in 0..page_count {
            let frame = alloc_frame().ok_or(AddressSpaceError::OutOfFrames)?;
            unsafe {
                write_bytes(frame.start_address() as *mut u8, 0, PAGE_SIZE);
            }

            let file_offset = page_index * PAGE_SIZE;
            if file_offset < file_data.len() {
                let copy_length = core::cmp::min(PAGE_SIZE, file_data.len() - file_offset);
                unsafe {
                    copy_nonoverlapping(
                        file_data.as_ptr().add(file_offset),
                        frame.start_address() as *mut u8,
                        copy_length,
                    );
                }
            }

            let virt = base + page_index * PAGE_SIZE;
            if let Err(error) = self.page_table.map_page(virt, frame.start_address(), flags) {
                dealloc_frame(frame);
                return Err(AddressSpaceError::Mapping(error));
            }
            self.owned_frames.push(frame);
        }

        Ok(())
    }

    pub fn map_stack(&mut self) -> Result<(), AddressSpaceError> {
        self.map_segment(
            USER_STACK_BASE,
            &[],
            PAGE_SIZE,
            PteFlags::READ | PteFlags::WRITE | PteFlags::USER,
        )
    }

    pub fn activate(&self) {
        self.page_table.activate();
    }

    pub fn read_user_byte(&self, address: VirtAddr) -> Option<u8> {
        let physical = self.page_table.translate_user_readable(address)?;
        Some(unsafe { read_volatile(physical as *const u8) })
    }

    #[cfg(feature = "test-kernel")]
    pub fn is_user_executable(&self, address: VirtAddr) -> bool {
        self.page_table.translate_user_executable(address).is_some()
    }

    #[cfg(feature = "test-kernel")]
    pub fn is_user_writable(&self, address: VirtAddr) -> bool {
        self.page_table.translate_user_writable(address).is_some()
    }

    pub fn finish_loading(&self) {
        csr::fence_i();
    }

    pub fn destroy(self) {
        self.page_table.destroy_user();
        for frame in self.owned_frames {
            dealloc_frame(frame);
        }
    }
}
