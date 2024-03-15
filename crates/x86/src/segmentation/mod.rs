use core::arch::asm;

use crate::{dt::DescriptorTable, PrivilegeLevel};

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    // TODO: Missing TI??
    pub fn new(addr: u16, rpl: PrivilegeLevel) -> Self {
        SegmentSelector(addr << 3 | (rpl as u16))
    }
}

pub struct CS;

// TODO: Use trait
impl CS {
    // TODO: Transform to macro
    pub fn get_reg() -> SegmentSelector {
        let selector: u16;
        unsafe {
            asm!(
                // Used to copy the `cs` value to the `{0:x}` address (`selector` address)
                "mov {0:x}, cs",
                // Outputs the value of (`cs` value) `selector`
                out(reg) selector,
                options(nomem, nostack, preserves_flags)
            );
        }

        SegmentSelector(selector) // TODO:
    }

    // TODO: Transform to macro
    pub fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!(
                // Push the `sel` value to the stack
                "push {sel}",
                // Load effective address (lea) of 1f + rip (1 byte forward + current instruction pointer) into tmp
                "lea {tmp}, [1f + rip]",
                // Push the value of tmp onto the stack
                "push {tmp}",
                // Return Far with Quick (return to a different privilege level), using the value on top of the stack
                "retfq",
                // Label definition for the forward reference
                "1:",
                // Input operand: assign the value of sel.0 to a general-purpose register
                sel = in(reg) u64::from(sel.0),
                // Late output operand: tmp is assigned the result of the lea instruction
                tmp = lateout(reg) _,
                options(preserves_flags),
            );
        }
    }
}
