use core::arch::asm;

use crate::segmentation::SegmentSelector;

pub mod interrupts;
pub mod port;
pub mod tlb;

pub unsafe fn load_tss(sel: SegmentSelector) {
    unsafe {
        asm!("ltr {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
    }
}

pub fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}
