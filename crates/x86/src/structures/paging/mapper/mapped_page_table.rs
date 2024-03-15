use bitflags::Flags;

use crate::structures::paging::{
    frame::PhysFrame,
    frame_alloc::FrameAllocator,
    page::{Page, Size4KiB},
    page_table::{self, FrameError, PageTable, PageTableEntry, PageTableFlags},
};

use super::{MapToError, MappedFrame, Mapper, MapperFlush, Translate, TranslateResult};

pub unsafe trait PageTableFrameMapping {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable;
}

#[derive(Debug)]
pub struct MappedPageTable<'a, P: PageTableFrameMapping> {
    page_table_walker: PageTableWalker<P>,
    level_4_table: &'a mut PageTable,
}

impl<'a, P: PageTableFrameMapping> MappedPageTable<'a, P> {
    pub unsafe fn new(level_4_table: &'a mut PageTable, mapping: P) -> Self {
        Self {
            level_4_table,
            page_table_walker: unsafe { PageTableWalker::new(mapping) },
        }
    }

    fn map_to_4kib<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let p4 = &mut self.level_4_table;
        let p3 = self.page_table_walker.create_next_table(
            &mut p4[page.p4_index().into()],
            parent_table_flags,
            allocator,
        )?;
        let p2 = self.page_table_walker.create_next_table(
            &mut p3[page.p3_index().into()],
            parent_table_flags,
            allocator,
        )?;
        let p1 = self.page_table_walker.create_next_table(
            &mut p2[page.p2_index().into()],
            parent_table_flags,
            allocator,
        )?;

        if !p1[page.p1_index().into()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p1[page.p1_index().into()].set_frame(frame, flags);
        Ok(MapperFlush::new(page))
    }
}

impl<'a, P: PageTableFrameMapping> Translate for MappedPageTable<'a, P> {
    fn translate(&self, addr: crate::addr::VirtAddr) -> TranslateResult {
        let p4 = &self.level_4_table;
        let p3 = match self
            .page_table_walker
            .next_table(&p4[addr.p4_index().into()])
        {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                panic!("level 4 entry has huge page bit set")
            }
        };

        let p2 = match self
            .page_table_walker
            .next_table(&p3[addr.p3_index().into()])
        {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p3[addr.p3_index().into()];
                let frame = PhysFrame::containing_address(entry.addr());
                let offset = addr.as_u64() & 0o_777_777_7777;
                let flags = entry.flags();
                return TranslateResult::Mapped {
                    frame: MappedFrame::Size1GiB(frame),
                    offset,
                    flags,
                };
            }
        };

        let p1 = match self
            .page_table_walker
            .next_table(&p2[addr.p2_index().into()])
        {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p2[addr.p2_index().into()];
                let frame = PhysFrame::containing_address(entry.addr());
                let offset = addr.as_u64() & 0o777_7777;
                let flags = entry.flags();
                return TranslateResult::Mapped {
                    frame: MappedFrame::Size2MiB(frame),
                    offset,
                    flags,
                };
            }
        };

        let p1_entry = &p1[addr.p1_index().into()];

        if p1_entry.is_unused() {
            return TranslateResult::NotMapped;
        }

        let frame = match PhysFrame::<Size4KiB>::from_start_address(p1_entry.addr()) {
            Ok(frame) => frame,
            Err(_) => return TranslateResult::InvalidFrameAddress(p1_entry.addr()),
        };

        let offset = u64::from(addr.page_offset());
        let flags = p1_entry.flags();
        TranslateResult::Mapped {
            frame: MappedFrame::Size4KiB(frame),
            offset,
            flags,
        }
    }
}

impl<'a, P: PageTableFrameMapping> Mapper<Size4KiB> for MappedPageTable<'a, P> {
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: crate::structures::paging::page::Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: page_table::PageTableFlags,
        parent_table_flags: page_table::PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<super::MapperFlush<Size4KiB>, super::MapToError<Size4KiB>>
    where
        Self: Sized,
        A: crate::structures::paging::frame_alloc::FrameAllocator<Size4KiB> + ?Sized,
    {
        self.map_to_4kib(page, frame, flags, parent_table_flags, frame_allocator)
    }
}

#[derive(Debug)]
enum PageTableWalkError {
    MappedToHugePage,
    NotMapped,
}

#[derive(Debug)]
enum PageTableCreateError {
    FrameAllocationFailed,
    MappedToHugePage,
}

impl From<PageTableCreateError> for MapToError<Size4KiB> {
    fn from(value: PageTableCreateError) -> Self {
        match value {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

#[derive(Debug)]
struct PageTableWalker<P: PageTableFrameMapping> {
    page_table_frame_mapping: P,
}

impl<P: PageTableFrameMapping> PageTableWalker<P> {
    pub unsafe fn new(page_table_frame_mapping: P) -> Self {
        Self {
            page_table_frame_mapping,
        }
    }

    fn next_table<'b>(
        &self,
        entry: &'b PageTableEntry,
    ) -> Result<&'b PageTable, PageTableWalkError> {
        let page_table_ptr = self
            .page_table_frame_mapping
            .frame_to_pointer(entry.frame()?);
        let page_table: &PageTable = unsafe { &*page_table_ptr };
        Ok(page_table)
    }

    fn next_table_mut<'b>(
        &self,
        entry: &'b mut PageTableEntry,
    ) -> Result<&'b mut PageTable, PageTableWalkError> {
        let page_table_ptr = self
            .page_table_frame_mapping
            .frame_to_pointer(entry.frame()?);
        let page_table: &mut PageTable = unsafe { &mut *page_table_ptr };
        Ok(page_table)
    }

    fn create_next_table<'b, A>(
        &self,
        entry: &'b mut PageTableEntry,
        insert_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<&'b mut PageTable, PageTableCreateError>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let created;

        if entry.is_unused() {
            if let Some(frame) = allocator.allocate_frame() {
                entry.set_frame(frame, insert_flags);
                created = true;
            } else {
                return Err(PageTableCreateError::FrameAllocationFailed);
            }
        } else {
            if !insert_flags.is_empty() && !entry.flags().contains(insert_flags) {
                entry.set_flags(entry.flags() | insert_flags);
            }
            created = false;
        }

        let page_table = match self.next_table_mut(entry) {
            Err(PageTableWalkError::MappedToHugePage) => {
                return Err(PageTableCreateError::MappedToHugePage);
            }
            Err(PageTableWalkError::NotMapped) => panic!("entry should be mapped at this point"),
            Ok(page_table) => page_table,
        };

        if created {
            page_table.zero();
        }

        Ok(page_table)
    }
}

impl From<FrameError> for PageTableWalkError {
    fn from(err: FrameError) -> Self {
        match err {
            FrameError::HugeFrame => PageTableWalkError::MappedToHugePage,
            FrameError::FrameNotPresent => PageTableWalkError::NotMapped,
        }
    }
}
