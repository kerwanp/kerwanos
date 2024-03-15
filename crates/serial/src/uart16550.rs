use lazy_static::lazy_static;
use spin::Mutex;
use x86::instructions::interrupts;

use crate::LineStsFlags;
use core::fmt::{self, Write};

pub const SERIAL1_PORT: u16 = 0x3F8;

lazy_static! {
    static ref SERIAL1: Mutex<SerialPort> = Mutex::new(SerialPort::new(SERIAL1_PORT));
}

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop()
        }
    };
}

pub struct SerialPort(u16);

impl SerialPort {
    pub fn new(port: u16) -> Self {
        Self(port)
    }

    fn base_port(&self) -> u16 {
        self.0
    }

    fn enable_interrupt_port(&self) -> u16 {
        self.base_port() + 1
    }

    fn fifo_control_port(&self) -> u16 {
        self.base_port() + 2
    }

    fn line_control_port(&self) -> u16 {
        self.base_port() + 3
    }

    fn modem_control_port(&self) -> u16 {
        self.base_port() + 4
    }

    fn line_sts_port(&self) -> u16 {
        self.base_port() + 5
    }

    fn data_port(&self) -> u16 {
        self.base_port()
    }

    unsafe fn line_sts(&self) -> LineStsFlags {
        LineStsFlags::from_bits_truncate(x86::io::inb(self.line_sts_port()))
    }

    pub fn init(&self) {
        unsafe {
            // Disable interrupt
            x86::io::outb(self.enable_interrupt_port(), 0x00);

            // Enable DLAB
            x86::io::outb(self.line_control_port(), 0x80);

            // Set maximum speed to 38400 bps by configuring DDL and DLM
            x86::io::outb(self.data_port(), 0x03);
            x86::io::outb(self.enable_interrupt_port(), 0x00);

            // Disable DLAB and set data word length to 8 bits
            x86::io::outb(self.line_control_port(), 0x03);

            // Enable FIFO, clear TX/RX queues and set interrupt watermark at 14 bytes
            x86::io::outb(self.fifo_control_port(), 0xc7);

            // Mark data terminal ready, signal request to send and enable auxilliary output #2
            // (used as interrupt line for CPU)
            x86::io::outb(self.modem_control_port(), 0x0b);

            // Enable interrupts
            x86::io::outb(self.enable_interrupt_port(), 0x01);
        }
    }

    pub fn send_raw(&self, data: u8) {
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
            x86::io::outb(self.data_port(), data);
        }
    }

    pub fn send(&self, data: u8) {
        match data {
            8 | 0x7F => {
                self.send_raw(8);
                self.send_raw(b' ');
                self.send_raw(8);
            }
            _ => {
                self.send_raw(data);
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).unwrap();
    });
}
