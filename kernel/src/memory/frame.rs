pub const PAGE_SIZE: usize = 4096;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

pub type PhysAddr = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryRange {
    pub start: PhysAddr,
    pub end: PhysAddr,
}

impl MemoryRange {
    pub const fn new(start: PhysAddr, end: PhysAddr) -> Self {
        Self { start, end }
    }

    #[allow(dead_code)]
    pub fn contains(self, addr: PhysAddr) -> bool {
        self.start <= addr && addr < self.end
    }

    #[allow(dead_code)]
    pub fn size(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn align_inward(self) -> Self {
        Self {
            start: align_up_to_page(self.start),
            end: align_down_to_page(self.end),
        }
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    start_paddr: PhysAddr,
}

impl Frame {
    pub const fn from_start_address(start_paddr: PhysAddr) -> Self {
        Self { start_paddr }
    }

    pub fn start_address(self) -> PhysAddr {
        self.start_paddr
    }
}

pub const fn align_up_to_page(addr: PhysAddr) -> PhysAddr {
    (addr + PAGE_MASK) & !PAGE_MASK
}

pub const fn align_down_to_page(addr: PhysAddr) -> PhysAddr {
    addr & !PAGE_MASK
}
