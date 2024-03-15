use super::{frame::PhysFrame, page::PageSize};

pub unsafe trait FrameAllocator<S: PageSize> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>>;
}

pub trait FrameDeallocator<S: PageSize> {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<S>);
}

