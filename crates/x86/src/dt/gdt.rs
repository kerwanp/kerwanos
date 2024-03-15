use core::{arch::asm, mem::size_of};

use bit_field::BitField;
use bitflags::bitflags;

use crate::{segmentation::SegmentSelector, tss::TaskStateSegment, PrivilegeLevel};

use super::DescriptorTablePointer;

bitflags! {
    pub struct DescriptorFlags: u64 {
        const ACCESSED = 1 << 40;
        const WRITABLE = 1 << 41;
        const CONFORMING = 1 << 42;
        const EXECUTABLE = 1 << 43;
        const USER_SEGMENT = 1 << 44;
        const DPL_RING_3 = 3 << 45;
        const PRESENT = 1 << 47;
        const AVAILABLE = 1 << 52;
        const LONG_MODE = 1 << 53;
        const DEFAULT_SIZE = 1 << 54;
        const GRANULARITY = 1 << 55;
        const LIMIT_0_15 = 0xFFFF;
        const LIMIT_16_19 = 0xF << 48;
        const BASE_0_23         = 0xFF_FFFF << 16;
        const BASE_24_31        = 0xFF << 56;
    }
}

impl DescriptorFlags {
    const COMMON: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );

    pub const KERNEL_CODE64: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::LONG_MODE.bits(),
    );
}

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    pub fn dpl(self) -> PrivilegeLevel {
        let value_low = match self {
            Descriptor::UserSegment(v) => v,
            Descriptor::SystemSegment(v, _) => v,
        };
        let dpl = (value_low & DescriptorFlags::DPL_RING_3.bits()) >> 45;
        PrivilegeLevel::from(dpl as u16)
    }

    pub fn kernel_code_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE64.bits())
    }

    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        let ptr = (tss as *const _) as u64;
        let mut low = DescriptorFlags::PRESENT.bits();

        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));
        Descriptor::SystemSegment(low, high)
    }
}

#[derive(Default)]
#[repr(transparent)]
pub struct Entry(u64);

#[repr(C)]
pub struct GlobalDescriptorTable {
    table: [Entry; 8],
    len: usize,
}

impl Default for GlobalDescriptorTable {
    fn default() -> Self {
        Self {
            table: Default::default(),
            len: 1,
        }
    }
}

impl GlobalDescriptorTable {
    pub fn load(&'static self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) &self.pointer(), options(readonly, nostack, preserves_flags));
        }
    }

    pub fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.len * size_of::<u64>() - 1) as u16,
        }
    }

    pub fn append(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => {
                if self.len > self.table.len().saturating_sub(1) {
                    panic!("GDT is full");
                }
                self.push(value)
            }
            Descriptor::SystemSegment(low, high) => {
                if self.len > self.table.len().saturating_sub(2) {
                    panic!("GDT requires two free spaces to hold a SystemSegment")
                }

                let index = self.push(low);
                self.push(high);
                index
            }
        };

        let dpl = entry.dpl();
        SegmentSelector::new(index as u16, dpl)
    }

    fn push(&mut self, value: u64) -> usize {
        let index = self.len;
        self.table[index] = Entry(value);
        self.len += 1;
        index
    }
}
