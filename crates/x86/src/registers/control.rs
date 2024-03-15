use core::arch::asm;

use bitflags::{bitflags, Flags};

use crate::{addr::PhysAddr, structures::paging::frame::PhysFrame};

pub struct Cr3;

bitflags! {
    pub struct Cr3Flags: u64 {
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

impl Cr3 {
    pub fn read() -> (PhysFrame, Cr3Flags) {
        let (frame, value) = Cr3::read_raw();
        let flags = Cr3Flags::from_bits_truncate(value.into());
        (frame, flags)
    }

    pub fn read_raw() -> (PhysFrame, u16) {
        let value: u64;
        unsafe {
            asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        }

        let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);
        let frame = PhysFrame::containing_address(addr);
        (frame, (value & 0xFFF) as u16)
    }
}

