use core::fmt::{self, Write};

use lazy_static::lazy_static;
use spin::Mutex;
use vga::{Char, CharStyle, BUFFER_HEIGHT, BUFFER_WIDTH};

pub use vga::Color;

lazy_static! {
    pub static ref TTY: Mutex<Tty> = Mutex::new(Tty::default());
}

#[derive(Debug, Default)]
pub struct Tty {
    row: usize,
    column: usize,
    style: CharStyle,
}

impl Tty {
    pub fn style(&self) -> CharStyle {
        self.style
    }

    pub fn set_style(&mut self, background: Color, foreground: Color) -> &mut Self {
        self.style = CharStyle::new(background, foreground);
        self
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn set_row(&mut self, row: usize) -> &mut Self {
        self.row = row;
        self
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn set_column(&mut self, column: usize) -> &mut Self {
        self.column = column;
        self
    }

    pub fn clear_last_line(&mut self) -> &mut Self {
        for x in 0..BUFFER_WIDTH {
            vga::write(
                Char::new(' ', CharStyle::new(Color::Black, Color::White)),
                x,
                BUFFER_HEIGHT - 1,
            )
        }
        self
    }

    pub fn write_char(&mut self, char: char) -> &mut Self {
        self.write_byte(char as u8);
        self
    }

    pub fn write_byte(&mut self, byte: u8) -> &mut Self {
        match byte {
            b'\n' => {
                self.new_line();
            }
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line();
                }

                vga::write(Char::new(byte as char, self.style), self.row, self.column);
                self.column += 1;
            }
        };
        self
    }

    pub fn clear_char(&mut self) -> &mut Self {
        vga::write(Char::new(' ', self.style), self.row, self.column - 1);
        self.column -= 1;
        self
    }

    pub fn new_line(&mut self) -> &mut Self {
        self.column = 0;
        self.row += 1;
        self
    }

    pub fn scroll(&mut self) -> &mut Self {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = vga::read(row, col);
                vga::write(char, row - 1, col);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column = 0;
        self
    }

    pub fn clear_row(&mut self, row: usize) -> &mut Self {
        let blank = Char::new(' ', self.style);
        for col in 0..BUFFER_WIDTH {
            vga::write(blank, row, col)
        }
        self
    }

    pub fn write_row(&mut self, str: &str, row: usize) -> &mut Self {
        self.clear_row(row);
        self.set_column(0);
        self.set_row(row);
        self.write_str(str);
        self
    }
}

impl fmt::Write for Tty {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for char in s.chars() {
            self.write_char(char);
        }

        Ok(())
    }
}
