use core::arch::asm;

pub fn enable() {
    unsafe {
        asm!("sti", options(preserves_flags, nostack));
    }
}

pub fn disable() {
    unsafe {
        asm!("cli", options(preserves_flags, nostack));
    }
}

pub fn enable_and_hlt() {
    unsafe {
        asm!("sti; hlt", options(nomem, nostack));
    }
}

pub fn are_enabled() -> bool {
    use crate::registers::rflags::{self, RFlags};
    rflags::read().contains(RFlags::INTERRUPT_FLAG)
}

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_intpt_flag = are_enabled();

    if saved_intpt_flag {
        disable();
    }

    let out = f();

    if saved_intpt_flag {
        enable();
    }

    out
}
