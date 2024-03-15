use core::{
    marker::PhantomData,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::addr::VirtAddr;

use super::page_table::PageTableIndex;

pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    const SIZE: u64;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Size4KiB {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Size2MiB {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Size1GiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
}

impl PageSize for Size2MiB {
    const SIZE: u64 = Size4KiB::SIZE * 512;
}

impl PageSize for Size1GiB {
    const SIZE: u64 = Size2MiB::SIZE * 512;
}

#[derive(Debug)]
pub struct AddressNotAligned;

#[derive(Debug)]
pub struct PageRangeInclusive<S: PageSize = Size4KiB> {
    pub start: Page<S>,
    pub end: Page<S>,
}

impl<S: PageSize> Iterator for PageRangeInclusive<S> {
    type Item = Page<S>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.start <= self.end {
            true => {
                let page = self.start;
                let max_page_addr = VirtAddr::new(u64::MAX) - (S::SIZE - 1);
                match self.start.start_address() < max_page_addr {
                    true => self.start += 1,
                    false => self.end -= 1,
                };
                Some(page)
            }
            false => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    pub const SIZE: u64 = S::SIZE;

    pub fn new_containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    pub fn range_inclusive(start: Self, end: Self) -> PageRangeInclusive<S> {
        PageRangeInclusive { start, end }
    }

    pub const fn start_address(self) -> VirtAddr {
        self.start_address
    }

    pub const fn p1_index(&self) -> PageTableIndex {
        self.start_address().p1_index()
    }

    pub const fn p2_index(&self) -> PageTableIndex {
        self.start_address().p2_index()
    }

    pub const fn p3_index(&self) -> PageTableIndex {
        self.start_address().p3_index()
    }

    pub const fn p4_index(&self) -> PageTableIndex {
        self.start_address().p4_index()
    }
}

impl<S: PageSize> Add<u64> for Page<S> {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Page::new_containing_address(self.start_address() + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for Page<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Page<S> {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Page::new_containing_address(self.start_address() - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for Page<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}
