
use std::io:: Result;
use crossterm::{terminal};
use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers};
use errno::errno;

use crate::keyboard::Keyboard;
use crate::screen::Screen;

use kilo_ed::EditorResult;

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
        })
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        loop {

            if self.screen.refresh().is_err() {
                self.die("Clear Screen");
            }
            self.screen.flush()?;
            if self.process_keypress() {
                break;
            }
        }
        terminal::disable_raw_mode()?;
        Ok(())
    }

    // keyboard
    pub fn process_keypress(&mut self) -> bool {
        let c = self.keyboard.read_key();
        match c {
            Ok(KeyEvent{
                   code: KeyCode::Char('q'),
                   modifiers: KeyModifiers::CONTROL, ..
               }) => true,
            Err(EditorResult::KeyReadFail) => {
                self.die("Unable to read from keyboard");
                unreachable!();
            },
            _ => false
        }
    }



    pub fn die<S: Into<String>>(&mut self, message : S) {
        let _ = self.screen.clear();
        let _ =  terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }

}
