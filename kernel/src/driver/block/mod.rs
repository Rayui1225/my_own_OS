//! Block-sized storage I/O shared by ramdisk and future hardware drivers.

mod ramdisk;

pub use ramdisk::RamDisk;

pub const BLOCK_SIZE: usize = 512;
pub type Block = [u8; BLOCK_SIZE];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockId(u64);

impl BlockId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockError {
    InvalidStorageSize,
    OutOfBounds,
}

pub trait BlockDevice {
    fn block_count(&self) -> u64;

    fn read_block(&mut self, block: BlockId, output: &mut Block) -> Result<(), BlockError>;

    fn write_block(&mut self, block: BlockId, input: &Block) -> Result<(), BlockError>;
}
