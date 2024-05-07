#![no_std]

pub mod commands;

extern crate alloc;

use alloc::{format, string::String};
use core::fmt::Write;
use futures_util::StreamExt;
use kernel::{
    task::keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, ScancodeStream},
    tty::{Color, TTY},
};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref SHELL: Mutex<Shell> = Mutex::new(Shell::new());
}

pub struct Shell {
    cmd: String,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            cmd: String::from(""),
        }
    }

    pub async fn init(&mut self) {
        self.render();
        self.handle_keypresses().await;
    }

    async fn handle_keypresses(&mut self) {
        let mut scancodes = ScancodeStream::new();
        let mut keyboard =
            Keyboard::new(ScancodeSet1::new(), layouts::Azerty, HandleControl::Ignore);

        while let Some(scancode) = scancodes.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => self.handle_keypress(character),
                        DecodedKey::RawKey(key) => {}
                    }
                }
            }
        }
    }

    fn handle_keypress(&mut self, char: char) {
        match char as u8 {
            b'\n' => self.enter(),
            0x8 => {
                self.cmd.pop();
                self.render();
            }
            _ => {
                self.cmd.push(char);
                self.render();
            }
        }
    }

    fn render(&self) {
        let mut tty = TTY.lock();
        let row = tty.row();
        tty.write_row(&format!("> {}", self.cmd), row);
    }

    fn enter(&mut self) {
        let res = commands::run(&self.cmd);
        TTY.lock().write_str(&format!("\n{}\n", res));
        self.cmd = String::from("");
        self.render();
    }
}

pub async fn init() {
    SHELL.lock().init().await;
}
