#![no_std]

extern crate alloc;

use alloc::{format, string::String};
use futures_util::StreamExt;
use kernel::task::keyboard::ScancodeStream;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use vga::WRITER;

pub struct Terminal {
    command: String,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            command: String::from(""),
        }
    }

    pub async fn init(&mut self) {
        self.handle_keypresses().await;
    }

    pub async fn handle_keypresses(&mut self) {
        let mut scancodes = ScancodeStream::new();
        let mut keyboard =
            Keyboard::new(ScancodeSet1::new(), layouts::Azerty, HandleControl::Ignore);

        while let Some(scancode) = scancodes.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    self.key_press(key);
                }
            }
        }
    }

    pub fn key_press(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode('\n') => self.enter_cmd(),
            DecodedKey::Unicode(character) => {
                self.command = format!("{}{}", self.command, character);
                vga::print!("{}", character);
            }
            DecodedKey::RawKey(key) => {}
        }
    }

    pub fn enter_cmd(&mut self) {
        // vga::WRITER.lock().clear_row(row)
    }
}

pub async fn init() {
    let mut terminal = Terminal::new();
    terminal.init().await;
}
