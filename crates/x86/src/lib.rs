#![no_std]
#![feature(abi_x86_interrupt)]

pub mod addr;
pub mod dt;
pub mod instructions;
pub mod interruptions;
pub mod io;
pub mod registers;
pub mod segmentation;
pub mod structures;
pub mod tss;

#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl From<u16> for PrivilegeLevel {
    // TODO: Maybe a shorter way?
    fn from(value: u16) -> Self {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("Invalid privilege level"),
        }
    }
}
