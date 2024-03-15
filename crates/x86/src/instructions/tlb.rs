use core::arch::asm;

use crate::addr::VirtAddr;

pub fn flush(addr: VirtAddr) {
    unsafe {
        asm!("invlpg [{}]", in(reg) addr.as_u64(), options(nostack, preserves_flags));
    }
}
