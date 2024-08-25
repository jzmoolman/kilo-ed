
use std::io:: Result;
use crossterm::{terminal};
use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers};
use errno::errno;

use crate::keyboard::Keyboard;
use crate::screen::Screen;

use kilo_ed::*;


pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Position::default(),
        })
    }


    // keyboard
    pub fn process_keypress(&mut self) -> Result<bool> {
        let c = self.keyboard.read_key();
        match c {
            Ok(KeyEvent{
                   code: KeyCode::Char('q'),
                   modifiers: KeyModifiers::CONTROL, ..
               }) => Ok(true),
            Err(EditorResult::KeyReadFail) => {
                self.die("Unable to read from keyboard");
                unreachable!();
            },
            _ => Ok(false)
        }
    }

    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        loop {

            if self.screen.refresh().is_err() {
                self.die("Clear Screen");
            }
            self.screen.move_to(&self.cursor)?;
            self.screen.flush()?;
            if self.process_keypress()? {
                break;
            }
        }
        terminal::disable_raw_mode()?;
        Ok(())
    }


    pub fn die<S: Into<String>>(&mut self, message : S) {
        let _ = self.screen.clear();
        let _ =  terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }

}
