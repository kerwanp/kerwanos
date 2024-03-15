use core::{
    fmt,
    ops::{Add, Sub},
};

use crate::structures::paging::page_table::{PageOffset, PageTableIndex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        let v = Self::new_truncate(addr);
        match v.0 == addr {
            true => v,
            false => panic!("virtual address must be sign extended in bits 48 to 64"),
        }
    }

    pub const fn new_truncate(addr: u64) -> Self {
        Self(((addr << 16) as i64 >> 16) as u64)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self.0 as u16)
    }

    pub const fn as_ptr<T>(self) -> *const T {
        self.as_u64() as *const T
    }

    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    pub const fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12) as u16)
    }

    pub const fn p2_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9) as u16)
    }

    pub const fn p3_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9) as u16)
    }

    pub const fn p4_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9 >> 9) as u16)
    }

    pub fn align_down(&self, align: u64) -> VirtAddr {
        VirtAddr::new_truncate(align_down(self.0, align))
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 + rhs)
    }
}

impl Sub<u64> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 - rhs)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        let p = Self::new_truncate(addr);
        if p.0 == addr {
            return p;
        }

        panic!("physical addresses must not have any bits in the range 52 to 64 set");
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn new_truncate(addr: u64) -> Self {
        Self(addr % (1 << 52))
    }

    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        PhysAddr(align_down(self.0, align.into()))
    }

    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.align_down(align).as_u64() == self.as_u64()
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0 + rhs)
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PhysAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

pub const fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}

pub const fn align_up(addr: u64, align: u64) -> u64 {
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        match (addr | align_mask).checked_add(1) {
            Some(aligned) => aligned,
            None => panic!("attempt to add with overflow"),
        }
    }
}
