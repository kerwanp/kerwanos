use core::{arch::asm, marker::PhantomData};

pub struct Port<T> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T> Port<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            phantom: PhantomData,
        }
    }
}

// TODO: Manage other sizes
impl<T: PortWrite> Port<T> {
    pub unsafe fn write(&mut self, value: T) {
        unsafe { T::write_to_port(self.port, value) }
    }
}

impl<T: PortRead> Port<T> {
    pub unsafe fn read(&mut self) -> T {
        unsafe { T::read_from_port(self.port) }
    }
}

pub trait PortWrite {
    unsafe fn write_to_port(port: u16, value: Self);
}

pub trait PortRead {
    unsafe fn read_from_port(port: u16) -> Self;
}

impl PortWrite for u8 {
    unsafe fn write_to_port(port: u16, value: Self) {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

impl PortWrite for u16 {
    unsafe fn write_to_port(port: u16, value: Self) {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

impl PortWrite for u32 {
    unsafe fn write_to_port(port: u16, value: Self) {
        asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
    }
}

impl PortRead for u8 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u8;
        unsafe {
            asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
        }
        value
    }
}

impl PortRead for u16 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u16;
        unsafe {
            asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        }
        value
    }
}
