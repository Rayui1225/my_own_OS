//! A process-owned Sv39 address space for the first fixed user image.

use core::ptr::{copy_nonoverlapping, read_volatile, write_bytes};

use super::{
    alloc_frame, dealloc_frame,
    paging::{MapError, PageTable, PteFlags, VirtAddr},
    Frame, PAGE_SIZE,
};
use crate::arch::riscv64::csr;

pub const USER_CODE_BASE: VirtAddr = 0x0001_0000;
pub const USER_STACK_TOP: VirtAddr = 0x4000_0000;
const USER_STACK_BASE: VirtAddr = USER_STACK_TOP - PAGE_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressSpaceError {
    ImageTooLarge,
    OutOfFrames,
    Mapping(MapError),
}

pub struct AddressSpace {
    page_table: PageTable,
    code_frame: Frame,
    stack_frame: Frame,
}

impl AddressSpace {
    pub fn new(image: &[u8]) -> Result<Self, AddressSpaceError> {
        if image.is_empty() || image.len() > PAGE_SIZE {
            return Err(AddressSpaceError::ImageTooLarge);
        }

        let code_frame = alloc_frame().ok_or(AddressSpaceError::OutOfFrames)?;
        let stack_frame = match alloc_frame() {
            Some(frame) => frame,
            None => {
                dealloc_frame(code_frame);
                return Err(AddressSpaceError::OutOfFrames);
            }
        };

        unsafe {
            write_bytes(code_frame.start_address() as *mut u8, 0, PAGE_SIZE);
            copy_nonoverlapping(
                image.as_ptr(),
                code_frame.start_address() as *mut u8,
                image.len(),
            );
            write_bytes(stack_frame.start_address() as *mut u8, 0, PAGE_SIZE);
        }
        csr::fence_i();

        let mut page_table = match PageTable::new_user() {
            Ok(page_table) => page_table,
            Err(error) => {
                dealloc_frame(stack_frame);
                dealloc_frame(code_frame);
                return Err(AddressSpaceError::Mapping(error));
            }
        };

        let map_result = page_table
            .map_page(
                USER_CODE_BASE,
                code_frame.start_address(),
                PteFlags::READ | PteFlags::EXECUTE | PteFlags::USER,
            )
            .and_then(|_| {
                page_table.map_page(
                    USER_STACK_BASE,
                    stack_frame.start_address(),
                    PteFlags::READ | PteFlags::WRITE | PteFlags::USER,
                )
            });

        if let Err(error) = map_result {
            page_table.destroy_user();
            dealloc_frame(stack_frame);
            dealloc_frame(code_frame);
            return Err(AddressSpaceError::Mapping(error));
        }

        Ok(Self {
            page_table,
            code_frame,
            stack_frame,
        })
    }

    pub fn activate(&self) {
        self.page_table.activate();
    }

    pub fn read_user_byte(&self, address: VirtAddr) -> Option<u8> {
        let physical = self.page_table.translate_user_readable(address)?;
        Some(unsafe { read_volatile(physical as *const u8) })
    }

    pub fn destroy(self) {
        self.page_table.destroy_user();
        dealloc_frame(self.stack_frame);
        dealloc_frame(self.code_frame);
    }
}
