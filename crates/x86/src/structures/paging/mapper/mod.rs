use crate::{
    addr::{PhysAddr, VirtAddr},
    instructions::tlb,
};

use super::{
    frame::PhysFrame,
    frame_alloc::FrameAllocator,
    page::{Page, PageSize, Size1GiB, Size2MiB, Size4KiB},
    page_table::PageTableFlags,
};

pub mod mapped_page_table;
pub mod offset_page_table;

#[derive(Debug)]
pub enum MappedFrame {
    Size4KiB(PhysFrame<Size4KiB>),
    Size2MiB(PhysFrame<Size2MiB>),
    Size1GiB(PhysFrame<Size1GiB>),
}

impl MappedFrame {
    pub const fn start_address(&self) -> PhysAddr {
        match self {
            MappedFrame::Size4KiB(frame) => frame.start_address(),
            MappedFrame::Size2MiB(frame) => frame.start_address(),
            MappedFrame::Size1GiB(frame) => frame.start_address(),
        }
    }
}

#[derive(Debug)]
pub enum TranslateResult {
    Mapped {
        frame: MappedFrame,
        offset: u64,
        flags: PageTableFlags,
    },
    NotMapped,
    InvalidFrameAddress(PhysAddr),
}

pub trait Translate {
    fn translate(&self, addr: VirtAddr) -> TranslateResult;
    fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        match self.translate(addr) {
            TranslateResult::NotMapped | TranslateResult::InvalidFrameAddress(_) => None,
            TranslateResult::Mapped { frame, offset, .. } => Some(frame.start_address() + offset),
        }
    }
}

#[derive(Debug)]
pub struct MapperFlush<S: PageSize>(Page<S>);

impl<S: PageSize> MapperFlush<S> {
    pub fn new(page: Page<S>) -> Self {
        Self(page)
    }

    pub fn flush(&self) {
        tlb::flush(self.0.start_address());
    }
}

pub trait Mapper<S: PageSize> {
    unsafe fn map_to<A>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let parent_table_flags = flags
            & (PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE);
        unsafe {
            self.map_to_with_table_flags(page, frame, flags, parent_table_flags, frame_allocator)
        }
    }

    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized;
}

#[derive(Debug)]
pub enum MapToError<S: PageSize> {
    FrameAllocationFailed,
    ParentEntryHugePage,
    PageAlreadyMapped(PhysFrame<S>),
}
