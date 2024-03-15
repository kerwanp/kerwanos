use crate::{
    addr::VirtAddr,
    structures::paging::{frame::PhysFrame, page::Size4KiB, page_table::PageTable},
};

use super::{
    mapped_page_table::{MappedPageTable, PageTableFrameMapping},
    Mapper, Translate,
};

#[derive(Debug)]
struct PhysOffset {
    offset: VirtAddr,
}

unsafe impl PageTableFrameMapping for PhysOffset {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        let virt = self.offset + frame.start_address().as_u64();
        virt.as_mut_ptr()
    }
}

pub struct OffsetPageTable<'a> {
    inner: MappedPageTable<'a, PhysOffset>,
}

impl<'a> OffsetPageTable<'a> {
    pub unsafe fn new(level_4_table: &'a mut PageTable, phys_offset: VirtAddr) -> Self {
        let phys_offset = PhysOffset {
            offset: phys_offset,
        };

        Self {
            inner: unsafe { MappedPageTable::new(level_4_table, phys_offset) },
        }
    }
}

impl<'a> Translate for OffsetPageTable<'a> {
    fn translate(&self, addr: VirtAddr) -> super::TranslateResult {
        self.inner.translate(addr)
    }
}

impl<'a> Mapper<Size4KiB> for OffsetPageTable<'a> {
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: crate::structures::paging::page::Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: crate::structures::paging::page_table::PageTableFlags,
        parent_table_flags: crate::structures::paging::page_table::PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<super::MapperFlush<Size4KiB>, super::MapToError<Size4KiB>>
    where
        Self: Sized,
        A: crate::structures::paging::frame_alloc::FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner.map_to_with_table_flags(
                page,
                frame,
                flags,
                parent_table_flags,
                frame_allocator,
            )
        }
    }
}
