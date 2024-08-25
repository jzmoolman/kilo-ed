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
    Left ,
    Right,
    Up,
    Down,
}

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
    keymap: HashMap<char, EditorKey>,
    rows: Vec<String>,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let mut keymap = HashMap::new();
        keymap.insert('k', EditorKey::Up);
        keymap.insert('j', EditorKey::Down);
        keymap.insert('l', EditorKey::Right);
        keymap.insert('h', EditorKey::Left);
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Position::default(),
            keymap,
            rows: vec!["Hello, world!".to_string()],
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
               KeyEvent {
                   code:KeyCode::Char(key), .. } => {
                   match key {
                       'h'| 'j' | 'k'| 'l' => {
                           let c  = *self.keymap.get(&key).unwrap();
                           self.move_cursor(c);
                       }
                       _ => {}
                   }
               }
               KeyEvent { code, .. } => match code {
                   KeyCode::Home => self.move_to_home(),
                   KeyCode::End => self.move_to_end(),
                   KeyCode::Up => { self.move_cursor(EditorKey::Up); },
                   KeyCode::Down => { self.move_cursor(EditorKey::Down); }
                   KeyCode::Left => { self.move_cursor(EditorKey::Left); }
                   KeyCode::Right => { self.move_cursor(EditorKey::Right); }
                   KeyCode::PageUp | KeyCode::PageDown => {
                       let bounds = self.screen.bounds();
                       for _ in 0..bounds.y {
                           self.move_cursor(
                               if code == KeyCode::PageUp {
                                   EditorKey::Up
                               } else {
                                   EditorKey::Down
                               }
                           );
                       }
                   }
                   _ => {}
               }
                // _ => {}
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

            if self.refresh_screen().is_err() {
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

    pub fn move_to_home(&mut self) {
        self.cursor.y = 0;
    }

    pub fn move_to_end(&mut self) {
        let position = self.screen.bounds();
        self.cursor.x = position.x;
    }

    pub fn move_cursor(&mut self, key: EditorKey)  {
        let bounds = self.screen.bounds();
        match key {
            EditorKey::Left => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            },
            EditorKey::Right if self.cursor.x < bounds.x => self.cursor.x += 1,
            EditorKey::Up => {
                self.cursor.y  = self.cursor.y.saturating_sub(1);
            },
            EditorKey::Down if self.cursor.y < bounds.y-1 => self.cursor.y +=1,
            _ => {}
        }
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.screen.clear()?;
        self.screen.draw_row(&self.rows)

        // self.stdout
        //     .queue(cursor::MoveTo(0,0))?;
        // Ok(())
    }

    // pub fn cursor_position(&self) -> Result<(u16, u16)> {
    //     cursor::position()
    // }k

}
