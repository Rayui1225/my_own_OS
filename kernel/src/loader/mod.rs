//! Loads validated SimpleBin bytes into a process-owned address space.

mod simplebin;

use crate::memory::{
    address_space::{AddressSpace, AddressSpaceError, USER_STACK_TOP},
    align_up_to_page,
    paging::PteFlags,
};

pub use simplebin::ParseError;

pub const USER_TEXT_BASE: usize = 0x0001_0000;
pub const USER_IMAGE_LIMIT: usize = 0x0100_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadError {
    Parse(ParseError),
    ImageTooLarge,
    AddressSpace(AddressSpaceError),
}

pub struct LoadedProgram {
    address_space: AddressSpace,
    entry_point: usize,
    stack_pointer: usize,
}

impl LoadedProgram {
    pub fn entry_point(&self) -> usize {
        self.entry_point
    }

    pub fn stack_pointer(&self) -> usize {
        self.stack_pointer
    }

    #[cfg(feature = "test-kernel")]
    pub fn address_space(&self) -> &AddressSpace {
        &self.address_space
    }

    pub fn into_parts(self) -> (AddressSpace, usize, usize) {
        (self.address_space, self.entry_point, self.stack_pointer)
    }

    #[cfg(feature = "test-kernel")]
    pub fn destroy(self) {
        self.address_space.destroy();
    }
}

pub fn load(bytes: &[u8]) -> Result<LoadedProgram, LoadError> {
    let image = simplebin::SimpleBin::parse(bytes, USER_TEXT_BASE).map_err(LoadError::Parse)?;
    let text_end = USER_TEXT_BASE
        .checked_add(image.text().len())
        .ok_or(LoadError::ImageTooLarge)?;
    let data_base = align_up_to_page(text_end);
    let data_memory_size = image
        .data()
        .len()
        .checked_add(image.bss_size())
        .ok_or(LoadError::ImageTooLarge)?;
    let image_end = data_base
        .checked_add(data_memory_size)
        .ok_or(LoadError::ImageTooLarge)?;
    if text_end > USER_IMAGE_LIMIT || image_end > USER_IMAGE_LIMIT {
        return Err(LoadError::ImageTooLarge);
    }

    let mut address_space = AddressSpace::new().map_err(LoadError::AddressSpace)?;
    let result = (|| {
        address_space.map_segment(
            USER_TEXT_BASE,
            image.text(),
            image.text().len(),
            PteFlags::READ | PteFlags::EXECUTE | PteFlags::USER,
        )?;
        if data_memory_size != 0 {
            address_space.map_segment(
                data_base,
                image.data(),
                data_memory_size,
                PteFlags::READ | PteFlags::WRITE | PteFlags::USER,
            )?;
        }
        address_space.map_stack()?;
        Ok::<(), AddressSpaceError>(())
    })();

    if let Err(error) = result {
        address_space.destroy();
        return Err(LoadError::AddressSpace(error));
    }
    address_space.finish_loading();

    Ok(LoadedProgram {
        address_space,
        entry_point: image.entry_point(),
        stack_pointer: USER_STACK_TOP,
    })
}
