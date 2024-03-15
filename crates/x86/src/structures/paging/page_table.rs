use core::{
    fmt,
    ops::{Index, IndexMut},
};

use bitflags::{bitflags, Flags};
use snafu::Snafu;

use crate::addr::PhysAddr;

use super::frame::PhysFrame;

#[derive(Clone, Copy)]
pub struct PageTableIndex<const ENTRY_COUNT: usize = 512>(u16);

impl<const ENTRY_COUNT: usize> PageTableIndex<ENTRY_COUNT> {
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    pub const fn new_truncate(index: u16) -> Self {
        Self(index % ENTRY_COUNT as u16)
    }
}

impl From<PageTableIndex> for usize {
    fn from(value: PageTableIndex) -> Self {
        usize::from(value.0)
    }
}

#[derive(Debug, Snafu)]
pub enum FrameError {
    #[snafu(display("Frame is not present"))]
    FrameNotPresent,
    #[snafu(display("Frame is to huge"))]
    HugeFrame,
}

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageTableFlags: u64 {
        const PRESENT =         1;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const BIT_9 =           1 << 9;
        const BIT_10 =          1 << 10;
        const BIT_11 =          1 << 11;
        const BIT_52 =          1 << 52;
        const BIT_53 =          1 << 53;
        const BIT_54 =          1 << 54;
        const BIT_55 =          1 << 55;
        const BIT_56 =          1 << 56;
        const BIT_57 =          1 << 57;
        const BIT_58 =          1 << 58;
        const BIT_59 =          1 << 59;
        const BIT_60 =          1 << 60;
        const BIT_61 =          1 << 61;
        const BIT_62 =          1 << 62;
        const NO_EXECUTE =      1 << 63;
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000f_ffff_ffff_f000)
    }

    pub fn frame(&self) -> Result<PhysFrame, FrameError> {
        if !self.flags().contains(PageTableFlags::PRESENT) {
            return Err(FrameError::FrameNotPresent);
        }

        if self.flags().contains(PageTableFlags::HUGE_PAGE) {
            return Err(FrameError::HugeFrame);
        }

        Ok(PhysFrame::containing_address(self.addr()))
    }

    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        self.set_addr(frame.start_address(), flags)
    }

    fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        self.0 = addr.as_u64() | flags.bits();
    }

    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.0 = self.addr().as_u64() | flags.bits();
    }

    fn set_unused(&mut self) {
        self.0 = 0;
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("addr", &self.addr());
        f.field("flags", &self.flags());
        f.finish()
    }
}

#[derive(Debug)]
#[repr(align(4096))]
#[repr(C)]
pub struct PageTable<const ENTRY_COUNT: usize = 512> {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl<const ENTRY_COUNT: usize> PageTable<ENTRY_COUNT> {
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        (0..ENTRY_COUNT).map(move |i| &self.entries[i])
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        let ptr = self.entries.as_mut_ptr();
        (0..ENTRY_COUNT).map(move |i| unsafe { &mut *ptr.add(i) })
    }

    pub fn zero(&mut self) {
        for entry in self.iter_mut() {
            entry.set_unused();
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

pub struct PageOffset(u16);

impl PageOffset {
    pub fn new(offset: u16) -> Self {
        assert!(offset < (1 << 12));
        Self(offset)
    }

    pub const fn new_truncate(offset: u16) -> Self {
        Self(offset % (1 << 12))
    }
}

impl From<PageOffset> for u64 {
    fn from(value: PageOffset) -> Self {
        u64::from(value.0)
    }
}
