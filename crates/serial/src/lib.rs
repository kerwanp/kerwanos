#![no_std]

use bitflags::bitflags;
pub mod uart16550;

bitflags! {
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        const OUTPUT_EMPTY = 1 << 5;
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart16550::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
