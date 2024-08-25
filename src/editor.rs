use std::collections::HashMap;
use std::io:: Result;
use crossterm::{terminal};
use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers};
use errno::errno;

use crate::keyboard::Keyboard;
use crate::screen::Screen;

use kilo_ed::*;

#[derive(Copy, Clone)]
pub enum EditorKey {
    ArrowLeft ,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
    keymap: HashMap<char, EditorKey>,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let mut keymap = HashMap::new();
        keymap.insert('w', EditorKey::ArrowUp);
        keymap.insert('s', EditorKey::ArrowDown);
        keymap.insert('a', EditorKey::ArrowRight);
        keymap.insert('d', EditorKey::ArrowLeft);
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            // cursor: Position::default(),
            cursor: Position { x: 10, y : 12},
            keymap,
        })
    }


    // keyboard
    pub fn process_keypress(&mut self) -> Result<bool> {
        if let Ok(c) = self.keyboard.read() {
            match c {
               KeyEvent {
                   code: KeyCode::Char('q'),
                   modifiers: KeyModifiers::CONTROL, ..
               } => return Ok(true),
               KeyEvent { code:KeyCode::Up,.. } => { self.move_cursor(EditorKey::ArrowUp);},
               KeyEvent { code:KeyCode::Down,.. } => { self.move_cursor(EditorKey::ArrowDown);}
               KeyEvent { code:KeyCode::Left,.. } => { self.move_cursor(EditorKey::ArrowLeft);},
               KeyEvent { code:KeyCode::Right,.. } => { self.move_cursor(EditorKey::ArrowRight);},
               KeyEvent { code:KeyCode::Char(key), .. } => {
                    match key {
                        'w'| 'a' | 'd'| 's' => {
                            let c  = *self.keymap.get(&key).unwrap();
                            self.move_cursor(c);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        } else {
            self.die("Unable to read from keyboard");
            unreachable!();
        }
        Ok(false)
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
        let _ = self.screen.clear();
        terminal::disable_raw_mode()?;
        Ok(())
    }


    pub fn die<S: Into<String>>(&mut self, message : S) {
        let _ = self.screen.clear();
        let _ =  terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }

    pub fn move_cursor(&mut self, key: EditorKey)  {
        let bounds = self.screen.bounds();
        match key {
            EditorKey::ArrowLeft => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            },
            EditorKey::ArrowRight if self.cursor.x <= bounds.x-1 => self.cursor.x += 1,
            EditorKey::ArrowUp => {
                self.cursor.y  = self.cursor.y.saturating_sub(1);
            },
            EditorKey::ArrowDown if self.cursor.y < bounds.y-1 => self.cursor.y +=1,
            _ => {}
        }
    }

}
