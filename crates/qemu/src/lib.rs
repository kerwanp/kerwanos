#![no_std]

use x86::instructions::port::Port;

const EXIT_PORT: u16 = 0xf4;

#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit(code: QemuExitCode) -> ! {
    let mut port = Port::new(EXIT_PORT);
    unsafe {
        port.write(code as u32);
    }

    panic!("Failed to exit QEMU");
}
