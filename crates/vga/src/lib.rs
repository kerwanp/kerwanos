#![no_std]

use core::fmt::{self, Write};

use lazy_static::lazy_static;
use spin::Mutex;
use x86::instructions::interrupts;

const ADDRESS: usize = 0xb8000;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = {
        let buffer = Buffer::new(ADDRESS);
        let writer = Writer::new(buffer);
        Mutex::new(writer)
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct CharStyle(u8);

impl CharStyle {
    pub fn new(background: Color, foreground: Color) -> Self {
        CharStyle(((background as u8) << 4) | (foreground as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Char {
    char: u8,
    style: CharStyle,
}

impl From<char> for Char {
    fn from(value: char) -> Self {
        Self {
            char: value as u8,
            style: CharStyle::new(Color::Black, Color::White),
        }
    }
}

impl Char {
    pub fn new(char: char, style: CharStyle) -> Self {
        Self {
            char: char as u8,
            style,
        }
    }
}

#[repr(transparent)]
pub struct Buffer([[Char; BUFFER_WIDTH]; BUFFER_HEIGHT]);

impl Buffer {
    fn new(addr: usize) -> &'static mut Self {
        unsafe { &mut *(addr as *mut Buffer) }
    }
}

impl Buffer {
    pub fn write(&mut self, char: Char, x: usize, y: usize) {
        self.0[y][x] = char
    }
}

pub struct Writer {
    buffer: &'static mut Buffer,
    column_cursor: usize,
    style: CharStyle,
}

impl Writer {
    pub fn new(buffer: &'static mut Buffer) -> Self {
        Self {
            buffer,
            column_cursor: 0,
            style: CharStyle::new(Color::Black, Color::White),
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_cursor >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_cursor;
                let style = self.style;

                self.buffer.0[row][col] = Char { char: byte, style };
                self.column_cursor += 1;
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Char {
            char: b' ',
            style: self.style,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.0[row][col] = blank;
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.0[row][col];
                self.buffer.0[row - 1][col] = char;
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_cursor = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}
