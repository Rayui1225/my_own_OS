//! A memory-backed block device used before a hardware storage driver exists.

use core::ops::Range;

use super::{Block, BlockDevice, BlockError, BlockId, BLOCK_SIZE};

pub struct RamDisk<'a> {
    storage: &'a mut [u8],
}

impl<'a> RamDisk<'a> {
    pub fn new(storage: &'a mut [u8]) -> Result<Self, BlockError> {
        if storage.is_empty() || storage.len() % BLOCK_SIZE != 0 {
            return Err(BlockError::InvalidStorageSize);
        }

        Ok(Self { storage })
    }

    fn block_range(&self, block: BlockId) -> Result<Range<usize>, BlockError> {
        let index = usize::try_from(block.value()).map_err(|_| BlockError::OutOfBounds)?;
        let start = index
            .checked_mul(BLOCK_SIZE)
            .ok_or(BlockError::OutOfBounds)?;
        let end = start
            .checked_add(BLOCK_SIZE)
            .ok_or(BlockError::OutOfBounds)?;
        if end > self.storage.len() {
            return Err(BlockError::OutOfBounds);
        }

        Ok(start..end)
    }
}

impl BlockDevice for RamDisk<'_> {
    fn block_count(&self) -> u64 {
        (self.storage.len() / BLOCK_SIZE) as u64
    }

    fn read_block(&mut self, block: BlockId, output: &mut Block) -> Result<(), BlockError> {
        let range = self.block_range(block)?;
        output.copy_from_slice(&self.storage[range]);
        Ok(())
    }

    fn write_block(&mut self, block: BlockId, input: &Block) -> Result<(), BlockError> {
        let range = self.block_range(block)?;
        self.storage[range].copy_from_slice(input);
        Ok(())
    }
}
