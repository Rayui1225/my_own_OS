//! Sv39 page-table primitives and the initial kernel identity mapping.

use core::ptr::{read_volatile, write_bytes, write_volatile};

use crate::arch::riscv64::csr;
use crate::println;

use super::{
    alloc_frame,
    frame::{align_down_to_page, align_up_to_page, Frame, MemoryRange, PhysAddr, PAGE_SIZE},
    map,
};

pub type VirtAddr = usize;

const SV39_LEVELS: usize = 3;
const ENTRIES_PER_TABLE: usize = 512;
const PAGE_TABLE_FLAGS: usize = PteFlags::VALID.bits();
const PTE_VALID: usize = PteFlags::VALID.bits();
const PTE_LEAF_MASK: usize =
    PteFlags::READ.bits() | PteFlags::WRITE.bits() | PteFlags::EXECUTE.bits();

static mut KERNEL_PAGE_TABLE: Option<PageTable> = None;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PteFlags(usize);

impl PteFlags {
    pub const VALID: Self = Self(1 << 0);
    pub const READ: Self = Self(1 << 1);
    pub const WRITE: Self = Self(1 << 2);
    pub const EXECUTE: Self = Self(1 << 3);
    pub const USER: Self = Self(1 << 4);
    pub const GLOBAL: Self = Self(1 << 5);
    pub const ACCESSED: Self = Self(1 << 6);
    pub const DIRTY: Self = Self(1 << 7);

    pub const fn bits(self) -> usize {
        self.0
    }

    const fn is_leaf(self) -> bool {
        self.0 & PTE_LEAF_MASK != 0
    }

    const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for PteFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.bits() | rhs.bits())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError {
    InvalidAddress,
    AlreadyMapped,
    MappingConflict,
    OutOfFrames,
    NotMapped,
    KernelPageTableUnavailable,
}

pub struct PageTable {
    root: Frame,
}

impl PageTable {
    pub fn new() -> Result<Self, MapError> {
        let root = alloc_zeroed_frame()?;
        Ok(Self { root })
    }

    pub fn root_paddr(&self) -> PhysAddr {
        self.root.start_address()
    }

    pub fn new_user() -> Result<Self, MapError> {
        let mut page_table = Self::new()?;
        let kernel_page_table = unsafe {
            KERNEL_PAGE_TABLE
                .as_ref()
                .ok_or(MapError::KernelPageTableUnavailable)?
        };

        // Root entry zero stays private because it contains both low user mappings and MMIO.
        // The remaining kernel subtrees are supervisor-only and can be shared read-only.
        for index in 1..ENTRIES_PER_TABLE {
            let source = unsafe { table_entry_ptr(kernel_page_table.root_paddr(), index) };
            let destination = unsafe { table_entry_ptr(page_table.root_paddr(), index) };
            unsafe { write_volatile(destination, read_volatile(source)) };
        }

        let device_mapping = page_table.map_page(
            map::UART_BASE,
            map::UART_BASE,
            PteFlags::READ | PteFlags::WRITE,
        );
        #[cfg(feature = "test-kernel")]
        let device_mapping = device_mapping.and_then(|_| {
            page_table.map_page(
                map::QEMU_TEST_FINISHER_BASE,
                map::QEMU_TEST_FINISHER_BASE,
                PteFlags::READ | PteFlags::WRITE,
            )
        });
        if let Err(error) = device_mapping {
            page_table.destroy_user();
            return Err(error);
        }

        Ok(page_table)
    }

    pub fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: PteFlags,
    ) -> Result<(), MapError> {
        if !is_sv39_address(virt) || !is_page_aligned(virt) || !is_page_aligned(phys) {
            return Err(MapError::InvalidAddress);
        }

        let leaf = self.find_or_create_leaf_entry(virt)?;
        let current = unsafe { read_volatile(leaf) };
        if current & PTE_VALID != 0 {
            return Err(MapError::AlreadyMapped);
        }

        let mut entry_flags = flags | PteFlags::VALID | PteFlags::ACCESSED;
        if flags.bits() & PteFlags::WRITE.bits() != 0 {
            // A writable mapping is ready for stores even on hardware that does not update D itself.
            entry_flags = entry_flags | PteFlags::DIRTY;
        }

        let value = phys_to_ppn(phys) | entry_flags.bits();
        unsafe { write_volatile(leaf, value) };
        Ok(())
    }

    pub fn unmap_page(&mut self, virt: VirtAddr) -> Result<(), MapError> {
        if !is_sv39_address(virt) || !is_page_aligned(virt) {
            return Err(MapError::InvalidAddress);
        }

        let leaf = self.find_leaf_entry(virt)?;
        let current = unsafe { read_volatile(leaf) };
        if current & PTE_VALID == 0 || !PteFlags(current).is_leaf() {
            return Err(MapError::NotMapped);
        }

        unsafe { write_volatile(leaf, 0) };
        csr::sfence_vma();
        Ok(())
    }

    pub fn translate_addr(&self, virt: VirtAddr) -> Option<PhysAddr> {
        let entry = self.leaf_entry(virt)?;
        Some(ppn_to_phys(entry) | (virt & (PAGE_SIZE - 1)))
    }

    pub fn translate_user_readable(&self, virt: VirtAddr) -> Option<PhysAddr> {
        self.translate_user_with_flags(virt, PteFlags::READ)
    }

    #[cfg(feature = "test-kernel")]
    pub fn translate_user_executable(&self, virt: VirtAddr) -> Option<PhysAddr> {
        self.translate_user_with_flags(virt, PteFlags::EXECUTE)
    }

    #[cfg(feature = "test-kernel")]
    pub fn translate_user_writable(&self, virt: VirtAddr) -> Option<PhysAddr> {
        self.translate_user_with_flags(virt, PteFlags::WRITE)
    }

    fn translate_user_with_flags(&self, virt: VirtAddr, required: PteFlags) -> Option<PhysAddr> {
        let entry = self.leaf_entry(virt)?;
        let flags = PteFlags(entry);
        if !flags.contains(PteFlags::USER) || !flags.contains(required) {
            return None;
        }

        Some(ppn_to_phys(entry) | (virt & (PAGE_SIZE - 1)))
    }

    pub fn activate(&self) {
        csr::enable_sv39(self.root_paddr());
    }

    pub fn destroy_user(self) {
        unsafe {
            let root_entry = table_entry_ptr(self.root_paddr(), 0);
            let value = read_volatile(root_entry);
            if value & PTE_VALID != 0 && !PteFlags(value).is_leaf() {
                let child = Frame::from_start_address(ppn_to_phys(value));
                dealloc_private_subtree(child, SV39_LEVELS - 2);
            }
        }
        super::dealloc_frame(self.root);
    }

    fn leaf_entry(&self, virt: VirtAddr) -> Option<usize> {
        if !is_sv39_address(virt) {
            return None;
        }

        let leaf = self.find_leaf_entry(align_down_to_page(virt)).ok()?;
        let entry = unsafe { read_volatile(leaf) };
        if entry & PTE_VALID == 0 || !PteFlags(entry).is_leaf() {
            return None;
        }

        Some(entry)
    }

    fn find_or_create_leaf_entry(&mut self, virt: VirtAddr) -> Result<*mut usize, MapError> {
        let mut table_paddr = self.root_paddr();

        for level in (1..SV39_LEVELS).rev() {
            let entry = unsafe { table_entry_ptr(table_paddr, vpn_index(virt, level)) };
            let value = unsafe { read_volatile(entry) };

            if value & PTE_VALID == 0 {
                let next_table = alloc_zeroed_frame()?;
                unsafe {
                    write_volatile(
                        entry,
                        phys_to_ppn(next_table.start_address()) | PAGE_TABLE_FLAGS,
                    )
                };
                table_paddr = next_table.start_address();
            } else if PteFlags(value).is_leaf() {
                return Err(MapError::MappingConflict);
            } else {
                table_paddr = ppn_to_phys(value);
            }
        }

        unsafe { Ok(table_entry_ptr(table_paddr, vpn_index(virt, 0))) }
    }

    fn find_leaf_entry(&self, virt: VirtAddr) -> Result<*mut usize, MapError> {
        let mut table_paddr = self.root_paddr();

        for level in (1..SV39_LEVELS).rev() {
            let entry = unsafe { table_entry_ptr(table_paddr, vpn_index(virt, level)) };
            let value = unsafe { read_volatile(entry) };
            if value & PTE_VALID == 0 || PteFlags(value).is_leaf() {
                return Err(MapError::NotMapped);
            }
            table_paddr = ppn_to_phys(value);
        }

        unsafe { Ok(table_entry_ptr(table_paddr, vpn_index(virt, 0))) }
    }
}

pub fn init() -> Result<(), MapError> {
    let mut page_table = PageTable::new()?;
    let sections = map::kernel_sections();

    map_identity_range(
        &mut page_table,
        sections.text,
        PteFlags::READ | PteFlags::EXECUTE,
    )?;
    map_identity_range(&mut page_table, sections.rodata, PteFlags::READ)?;
    map_identity_range(
        &mut page_table,
        sections.data,
        PteFlags::READ | PteFlags::WRITE,
    )?;
    map_identity_range(
        &mut page_table,
        sections.boot_stack,
        PteFlags::READ | PteFlags::WRITE,
    )?;
    map_identity_range(
        &mut page_table,
        sections.bss,
        PteFlags::READ | PteFlags::WRITE,
    )?;

    let free_ram_start = align_up_to_page(map::kernel_reserved_range().end);
    map_identity_range(
        &mut page_table,
        MemoryRange::new(free_ram_start, map::RAM_END),
        PteFlags::READ | PteFlags::WRITE,
    )?;
    map_identity_range(
        &mut page_table,
        MemoryRange::new(map::UART_BASE, map::UART_BASE + PAGE_SIZE),
        PteFlags::READ | PteFlags::WRITE,
    )?;
    #[cfg(feature = "test-kernel")]
    map_identity_range(
        &mut page_table,
        MemoryRange::new(
            map::QEMU_TEST_FINISHER_BASE,
            map::QEMU_TEST_FINISHER_BASE + PAGE_SIZE,
        ),
        PteFlags::READ | PteFlags::WRITE,
    )?;

    let translated = page_table
        .translate_addr(map::RAM_START)
        .ok_or(MapError::NotMapped)?;
    assert_eq!(translated, map::RAM_START);

    println!("[vm] kernel page table initialized");
    csr::enable_sv39(page_table.root_paddr());
    println!("[vm] mmu enabled");
    println!("[vm] translate {:#x} -> {:#x}", map::RAM_START, translated);

    unsafe {
        // The single kernel mapper remains available for later kernel-only mappings.
        KERNEL_PAGE_TABLE = Some(page_table);
    }

    Ok(())
}

pub fn map_kernel_page(virt: VirtAddr, phys: PhysAddr, flags: PteFlags) -> Result<(), MapError> {
    unsafe {
        let page_table = KERNEL_PAGE_TABLE
            .as_mut()
            .ok_or(MapError::KernelPageTableUnavailable)?;
        page_table.map_page(virt, phys, flags)?;
    }

    csr::sfence_vma();
    Ok(())
}

pub fn activate_kernel_page_table() -> Result<(), MapError> {
    let page_table = unsafe {
        KERNEL_PAGE_TABLE
            .as_ref()
            .ok_or(MapError::KernelPageTableUnavailable)?
    };
    page_table.activate();
    Ok(())
}

fn map_identity_range(
    page_table: &mut PageTable,
    range: MemoryRange,
    flags: PteFlags,
) -> Result<(), MapError> {
    let start = align_down_to_page(range.start);
    let end = align_up_to_page(range.end);

    for addr in (start..end).step_by(PAGE_SIZE) {
        page_table.map_page(addr, addr, flags)?;
    }

    Ok(())
}

fn alloc_zeroed_frame() -> Result<Frame, MapError> {
    let frame = alloc_frame().ok_or(MapError::OutOfFrames)?;

    unsafe {
        // Before and after MMU activation this frame is reachable at its physical address.
        write_bytes(frame.start_address() as *mut u8, 0, PAGE_SIZE);
    }

    Ok(frame)
}

fn is_page_aligned(addr: usize) -> bool {
    addr & (PAGE_SIZE - 1) == 0
}

fn is_sv39_address(addr: VirtAddr) -> bool {
    let upper = addr >> 39;
    upper == 0 || upper == (usize::MAX >> 39)
}

fn vpn_index(virt: VirtAddr, level: usize) -> usize {
    (virt >> (12 + level * 9)) & 0x1ff
}

fn phys_to_ppn(phys: PhysAddr) -> usize {
    (phys >> 12) << 10
}

fn ppn_to_phys(entry: usize) -> PhysAddr {
    (entry >> 10) << 12
}

unsafe fn table_entry_ptr(table_paddr: PhysAddr, index: usize) -> *mut usize {
    debug_assert!(index < ENTRIES_PER_TABLE);
    (table_paddr as *mut usize).add(index)
}

unsafe fn dealloc_private_subtree(table: Frame, level: usize) {
    let table_paddr = table.start_address();
    if level > 0 {
        for index in 0..ENTRIES_PER_TABLE {
            let value = read_volatile(table_entry_ptr(table_paddr, index));
            if value & PTE_VALID != 0 && !PteFlags(value).is_leaf() {
                let child = Frame::from_start_address(ppn_to_phys(value));
                dealloc_private_subtree(child, level - 1);
            }
        }
    }
    super::dealloc_frame(table);
}
