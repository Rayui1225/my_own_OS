//! Parser for the small, fixed-layout userspace image format.

pub const HEADER_SIZE: usize = 32;
pub const MAGIC: &[u8; 4] = b"SBIN";
pub const VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    TruncatedHeader,
    InvalidMagic,
    UnsupportedVersion,
    InvalidHeaderSize,
    ReservedFlags,
    EmptyText,
    SizeOverflow,
    InvalidPayloadSize,
    InvalidEntryPoint,
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleBin<'a> {
    entry_point: usize,
    text: &'a [u8],
    data: &'a [u8],
    bss_size: usize,
}

impl<'a> SimpleBin<'a> {
    pub fn parse(bytes: &'a [u8], text_base: usize) -> Result<Self, ParseError> {
        if bytes.len() < HEADER_SIZE {
            return Err(ParseError::TruncatedHeader);
        }
        if &bytes[0..4] != MAGIC {
            return Err(ParseError::InvalidMagic);
        }
        if read_u16(bytes, 4) != VERSION {
            return Err(ParseError::UnsupportedVersion);
        }
        if read_u16(bytes, 6) as usize != HEADER_SIZE {
            return Err(ParseError::InvalidHeaderSize);
        }
        if read_u32(bytes, 28) != 0 {
            return Err(ParseError::ReservedFlags);
        }

        let entry_point =
            usize::try_from(read_u64(bytes, 8)).map_err(|_| ParseError::InvalidEntryPoint)?;
        let text_size = read_u32(bytes, 16) as usize;
        let data_size = read_u32(bytes, 20) as usize;
        let bss_size = read_u32(bytes, 24) as usize;
        if text_size == 0 {
            return Err(ParseError::EmptyText);
        }

        let text_end = HEADER_SIZE
            .checked_add(text_size)
            .ok_or(ParseError::SizeOverflow)?;
        let payload_end = text_end
            .checked_add(data_size)
            .ok_or(ParseError::SizeOverflow)?;
        if payload_end != bytes.len() {
            return Err(ParseError::InvalidPayloadSize);
        }
        let virtual_text_end = text_base
            .checked_add(text_size)
            .ok_or(ParseError::SizeOverflow)?;
        if !(text_base..virtual_text_end).contains(&entry_point) {
            return Err(ParseError::InvalidEntryPoint);
        }

        Ok(Self {
            entry_point,
            text: &bytes[HEADER_SIZE..text_end],
            data: &bytes[text_end..payload_end],
            bss_size,
        })
    }

    pub fn entry_point(self) -> usize {
        self.entry_point
    }

    pub fn text(self) -> &'a [u8] {
        self.text
    }

    pub fn data(self) -> &'a [u8] {
        self.data
    }

    pub fn bss_size(self) -> usize {
        self.bss_size
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}
