use core::marker::PhantomData;

use crate::addr::PhysAddr;

use super::page::{AddressNotAligned, PageSize, Size4KiB};

#[derive(Debug, Clone, Copy)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    start_address: PhysAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    pub const fn start_address(self) -> PhysAddr {
        self.start_address
    }

    pub fn from_start_address(addr: PhysAddr) -> Result<PhysFrame, AddressNotAligned> {
        if !addr.is_aligned(S::SIZE) {
            return Err(AddressNotAligned);
        }

        Ok(PhysFrame {
            start_address: addr,
            size: PhantomData,
        })
    }
}
