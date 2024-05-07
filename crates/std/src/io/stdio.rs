use core::fmt::{self, Write};

use kernel::tty::TTY;
use x86::instructions::interrupts;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    interrupts::without_interrupts(|| {
        TTY.lock().write_fmt(args).unwrap();
    })
}
