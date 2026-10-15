//! Read-only SimpleFS backed by 512-byte block I/O.

use core::{cmp::min, str};

use crate::driver::block::{BlockDevice, BlockError, BlockId, BLOCK_SIZE};

pub const IMAGE: &[u8] = include_bytes!("../../../fs/image/simplefs.img");

const MAGIC: &[u8; 4] = b"SFS1";
const VERSION: u32 = 1;
const INODE_SIZE: usize = 32;
const ENTRY_SIZE: usize = 32;
const NAME_SIZE: usize = 27;
const INODES_PER_BLOCK: usize = BLOCK_SIZE / INODE_SIZE;
const FILE: u8 = 1;
const DIRECTORY: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsError {
    Device(BlockError),
    CorruptImage,
    InvalidPath,
    NotFound,
    NotFile,
    NotDirectory,
}

impl From<BlockError> for FsError {
    fn from(error: BlockError) -> Self {
        Self::Device(error)
    }
}

#[derive(Clone, Copy)]
struct Inode {
    kind: u8,
    size: u32,
    start_block: u32,
    block_count: u32,
}

#[derive(Clone, Copy)]
pub struct FileHandle {
    inode: Inode,
    offset: usize,
}

pub struct SimpleFs<D: BlockDevice> {
    device: D,
    total_blocks: u32,
    inode_count: u32,
    root_inode: u32,
}

impl<D: BlockDevice> SimpleFs<D> {
    pub fn mount(mut device: D) -> Result<Self, FsError> {
        let mut block = [0; BLOCK_SIZE];
        device.read_block(BlockId::new(0), &mut block)?;
        if &block[0..4] != MAGIC || read_u32(&block, 4) != VERSION {
            return Err(FsError::CorruptImage);
        }
        let total_blocks = read_u32(&block, 8);
        let inode_count = read_u32(&block, 12);
        let root_inode = read_u32(&block, 16);
        if total_blocks < 3
            || u64::from(total_blocks) != device.block_count()
            || inode_count == 0
            || inode_count as usize > INODES_PER_BLOCK
            || root_inode >= inode_count
            || read_u32(&block, 20) != 1
        {
            return Err(FsError::CorruptImage);
        }

        let mut fs = Self {
            device,
            total_blocks,
            inode_count,
            root_inode,
        };
        if fs.inode(root_inode)?.kind != DIRECTORY {
            return Err(FsError::CorruptImage);
        }
        Ok(fs)
    }

    pub fn open(&mut self, path: &str) -> Result<FileHandle, FsError> {
        let inode_id = self.resolve(path)?;
        let inode = self.inode(inode_id)?;
        if inode.kind != FILE {
            return Err(FsError::NotFile);
        }
        Ok(FileHandle { inode, offset: 0 })
    }

    pub fn read(&mut self, file: &mut FileHandle, output: &mut [u8]) -> Result<usize, FsError> {
        let size = file.inode.size as usize;
        let length = min(output.len(), size.saturating_sub(file.offset));
        let mut copied = 0;
        let mut block = [0; BLOCK_SIZE];
        while copied < length {
            let position = file.offset + copied;
            let block_index = position / BLOCK_SIZE;
            let block_offset = position % BLOCK_SIZE;
            self.device.read_block(
                BlockId::new(u64::from(file.inode.start_block) + block_index as u64),
                &mut block,
            )?;
            let chunk = min(length - copied, BLOCK_SIZE - block_offset);
            output[copied..copied + chunk]
                .copy_from_slice(&block[block_offset..block_offset + chunk]);
            copied += chunk;
        }
        file.offset += copied;
        Ok(copied)
    }

    pub fn readdir(&mut self, path: &str, mut visit: impl FnMut(&str)) -> Result<(), FsError> {
        let inode_id = self.resolve(path)?;
        let inode = self.inode(inode_id)?;
        if inode.kind != DIRECTORY {
            return Err(FsError::NotDirectory);
        }
        let mut block = [0; BLOCK_SIZE];
        for entry_index in 0..inode.size as usize / ENTRY_SIZE {
            if entry_index % (BLOCK_SIZE / ENTRY_SIZE) == 0 {
                self.device.read_block(
                    BlockId::new(
                        u64::from(inode.start_block)
                            + (entry_index * ENTRY_SIZE / BLOCK_SIZE) as u64,
                    ),
                    &mut block,
                )?;
            }
            let offset = (entry_index * ENTRY_SIZE) % BLOCK_SIZE;
            let name = self.entry_name(&block[offset..offset + ENTRY_SIZE])?;
            visit(name);
        }
        Ok(())
    }

    fn resolve(&mut self, path: &str) -> Result<u32, FsError> {
        if !path.starts_with('/') || path.contains("//") || (path.len() > 1 && path.ends_with('/'))
        {
            return Err(FsError::InvalidPath);
        }
        let mut current = self.root_inode;
        for component in path.split('/').skip(1).filter(|part| !part.is_empty()) {
            if component == "." || component == ".." || component.len() > NAME_SIZE {
                return Err(FsError::InvalidPath);
            }
            let dir = self.inode(current)?;
            if dir.kind != DIRECTORY {
                return Err(FsError::NotDirectory);
            }
            current = self.find_entry(dir, component)?.ok_or(FsError::NotFound)?;
        }
        Ok(current)
    }

    fn find_entry(&mut self, dir: Inode, wanted: &str) -> Result<Option<u32>, FsError> {
        let mut block = [0; BLOCK_SIZE];
        for index in 0..dir.size as usize / ENTRY_SIZE {
            if index % (BLOCK_SIZE / ENTRY_SIZE) == 0 {
                self.device.read_block(
                    BlockId::new(
                        u64::from(dir.start_block) + (index * ENTRY_SIZE / BLOCK_SIZE) as u64,
                    ),
                    &mut block,
                )?;
            }
            let offset = (index * ENTRY_SIZE) % BLOCK_SIZE;
            let entry = &block[offset..offset + ENTRY_SIZE];
            if self.entry_name(entry)? == wanted {
                return Ok(Some(read_u32(entry, 0)));
            }
        }
        Ok(None)
    }

    fn entry_name<'a>(&self, entry: &'a [u8]) -> Result<&'a str, FsError> {
        let inode_id = read_u32(entry, 0);
        let length = entry[4] as usize;
        if inode_id >= self.inode_count || length == 0 || length > NAME_SIZE {
            return Err(FsError::CorruptImage);
        }
        let name = str::from_utf8(&entry[5..5 + length]).map_err(|_| FsError::CorruptImage)?;
        if name == "." || name == ".." || name.contains('/') {
            return Err(FsError::CorruptImage);
        }
        Ok(name)
    }

    fn inode(&mut self, id: u32) -> Result<Inode, FsError> {
        if id >= self.inode_count {
            return Err(FsError::CorruptImage);
        }
        let mut block = [0; BLOCK_SIZE];
        self.device.read_block(BlockId::new(1), &mut block)?;
        let offset = id as usize * INODE_SIZE;
        let bytes = &block[offset..offset + INODE_SIZE];
        let inode = Inode {
            kind: bytes[0],
            size: read_u32(bytes, 4),
            start_block: read_u32(bytes, 8),
            block_count: read_u32(bytes, 12),
        };
        let end = inode
            .start_block
            .checked_add(inode.block_count)
            .ok_or(FsError::CorruptImage)?;
        if !matches!(inode.kind, FILE | DIRECTORY)
            || inode.start_block < 2
            || end > self.total_blocks
            || inode.size as u64 > u64::from(inode.block_count) * BLOCK_SIZE as u64
            || (inode.kind == DIRECTORY && inode.size as usize % ENTRY_SIZE != 0)
        {
            return Err(FsError::CorruptImage);
        }
        Ok(inode)
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
