use core::arch::asm;

use bitflags::{bitflags, Flags};

bitflags! {
    #[repr(transparent)]
    pub struct RFlags: u64 {
        const ID = 1 << 21;
        const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
        const VIRTUAL_INTERRUPT = 1 << 19;
        const ALIGMENT_CHECK = 1 << 18;
        const VIRTUAL_8086_MODE = 1 << 17;
        const RESUME_FLAG = 1 << 16;
        const NESTED_TASK = 1 << 14;
        const IOPL_HIGH = 1 << 13;
        const OVERFLOW_FLAG = 1 << 10;
        const INTERRUPT_FLAG = 1 << 9;
        const TRAP_FLAG = 1 << 9;
        const SIGN_FLAG = 1 << 7;
        const ZERO_FLAG = 1 << 6;
        const AUXILIARY_CARRY_FLAG = 1 << 4;
        const PARITY_FLAG = 1 << 4;
        const CARRY_FLAG = 1;
    }
}

pub fn read() -> RFlags {
    RFlags::from_bits_truncate(read_raw())
}

pub fn read_raw() -> u64 {
    let r: u64;
    unsafe {
        asm!("pushfq; pop {}", out(reg) r, options(nomem, preserves_flags));
    }
    r
}
