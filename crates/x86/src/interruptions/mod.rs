use core::arch::asm;

pub fn int3() {
    unsafe { asm!("int3", options(nomem, nostack)) }
}
