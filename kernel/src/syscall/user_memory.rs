#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserMemoryError {
    InvalidAddress,
}

pub trait UserMemory {
    fn read_byte(&self, address: usize) -> Result<u8, UserMemoryError>;
}

pub struct CurrentUserMemory;

impl UserMemory for CurrentUserMemory {
    fn read_byte(&self, address: usize) -> Result<u8, UserMemoryError> {
        crate::process::read_current_user_byte(address).ok_or(UserMemoryError::InvalidAddress)
    }
}

#[cfg(feature = "test-kernel")]
pub struct SliceUserMemory<'a> {
    base: usize,
    bytes: &'a [u8],
}

#[cfg(feature = "test-kernel")]
impl<'a> SliceUserMemory<'a> {
    pub const fn new(base: usize, bytes: &'a [u8]) -> Self {
        Self { base, bytes }
    }
}

#[cfg(feature = "test-kernel")]
impl UserMemory for SliceUserMemory<'_> {
    fn read_byte(&self, address: usize) -> Result<u8, UserMemoryError> {
        let offset = address
            .checked_sub(self.base)
            .ok_or(UserMemoryError::InvalidAddress)?;
        self.bytes
            .get(offset)
            .copied()
            .ok_or(UserMemoryError::InvalidAddress)
    }
}
