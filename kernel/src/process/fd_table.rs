//! File descriptors owned by one process.

use crate::{
    driver::block::BlockDevice,
    fs::{FileHandle, FsError, SimpleFs},
};

const FIRST_FILE_FD: usize = 3;
const FILE_SLOTS: usize = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FdError {
    Filesystem(FsError),
    BadFd,
    TableFull,
}

impl From<FsError> for FdError {
    fn from(error: FsError) -> Self {
        Self::Filesystem(error)
    }
}

pub struct FdTable {
    files: [Option<FileHandle>; FILE_SLOTS],
}

impl FdTable {
    pub const fn new() -> Self {
        Self {
            files: [None; FILE_SLOTS],
        }
    }

    pub fn open<D: BlockDevice>(
        &mut self,
        fs: &mut SimpleFs<D>,
        path: &str,
    ) -> Result<usize, FdError> {
        let slot = self
            .files
            .iter()
            .position(Option::is_none)
            .ok_or(FdError::TableFull)?;
        let file = fs.open(path)?;
        self.files[slot] = Some(file);
        Ok(FIRST_FILE_FD + slot)
    }

    pub fn read<D: BlockDevice>(
        &mut self,
        fs: &mut SimpleFs<D>,
        fd: usize,
        output: &mut [u8],
    ) -> Result<usize, FdError> {
        let file = self.file_mut(fd)?;
        Ok(fs.read(file, output)?)
    }

    pub fn close(&mut self, fd: usize) -> Result<(), FdError> {
        let slot = fd.checked_sub(FIRST_FILE_FD).ok_or(FdError::BadFd)?;
        let file = self.files.get_mut(slot).ok_or(FdError::BadFd)?;
        file.take().ok_or(FdError::BadFd)?;
        Ok(())
    }

    pub fn close_all(&mut self) {
        self.files.fill(None);
    }

    fn file_mut(&mut self, fd: usize) -> Result<&mut FileHandle, FdError> {
        let slot = fd.checked_sub(FIRST_FILE_FD).ok_or(FdError::BadFd)?;
        self.files
            .get_mut(slot)
            .and_then(Option::as_mut)
            .ok_or(FdError::BadFd)
    }
}
