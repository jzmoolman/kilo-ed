use std::io:: Result;
use std::path::Path;
use std::time::{Instant, Duration};
use crossterm::{terminal};
use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers};
use errno::errno;

use crate::keyboard::*;
use crate::screen::*;
use crate::row::*;

use kilo_ed::*;

#[derive(Copy, Clone)]
pub enum EditorKey {
    Left ,
    Right,
    Up,
    Down,
}

pub struct Editor {
    filename: String,
    status_msg: String,
    status_time: Instant,
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
    render_x: u16,
    rows: Vec<Row>,
    rowoff: u16,
    coloff: u16,
}

impl Editor {
    pub fn with_file <P: AsRef<Path> + ToString>(filename: P) -> Result<Self> {
        let fn_filename = filename.to_string();
        let lines:Vec<String> = std::fs::read_to_string(filename)
            .expect("Unable to open file")
            .split('\n')
            .map(|x| x.into())
            .collect();
       Editor::build(&lines, fn_filename)
    }

    pub fn new() -> Result<Self> {
        Editor::build(&[], "")
    }

    fn build<T: Into<String>>(data: &[String], filename: T) -> Result<Self> {
        Ok(Self {
            filename: filename.into(),
            status_msg: String::from("HELP: Ctrl-Q = Quit"),
            status_time: Instant::now(),
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Position::default(),
            render_x: 0,
            rows: if data.is_empty() {
                Vec::new()
            } else {
                let mut rows = Vec::new();
                for line in  data {
                     let row = Row::new(line.to_string());
                     rows.push(row);
                };
                if rows.last().unwrap().len() == 0 {
                    rows.pop();
                }
                rows
            },
            rowoff: 0,
            coloff: 0,
        })
    }

    // keyboard
    pub fn process_keypress(&mut self) -> Result<bool> {
        if let Ok(c) = self.keyboard.read() {
            match c {
               /*
                * Control-q
                */
               KeyEvent {
                   code: KeyCode::Char('q'),
                   modifiers: KeyModifiers::CONTROL, ..
               } => return Ok(true),
               KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: KeyModifiers::CONTROL, ..
                } => {}, // TODO
                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::CONTROL, ..
                } => {}, // TODO

               KeyEvent {
                   code: KeyCode::Char(key),
                   modifiers: KeyModifiers::NONE, ..
               } => self.insert_char(key),
               KeyEvent { code, .. } => match code {
                   KeyCode::Delete => {},  // TODO
                   KeyCode::Backspace => {},
                   KeyCode::Esc => {},
                   KeyCode::Home => self.move_to_home(),
                   KeyCode::End => self.move_to_end(),
                   KeyCode::Up => { self.move_cursor(EditorKey::Up); },
                   KeyCode::Down => { self.move_cursor(EditorKey::Down); }
                   KeyCode::Left => { self.move_cursor(EditorKey::Left); }
                   KeyCode::Right => { self.move_cursor(EditorKey::Right); }
                   KeyCode::PageUp | KeyCode::PageDown => {
                       let bounds = self.screen.bounds();
                       match code {
                           KeyCode::PageUp => self.cursor.y = self.rowoff,
                           KeyCode::PageDown => self.cursor.y  = (self.rowoff + bounds.y-1).min(self.rows.len() as u16),
                           _ => panic!("rust compiler broke")
                       }

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
            self.screen.move_to(&self.cursor, self.render_x, self.rowoff, self.coloff)?;
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
        if self.cursor.y < self.rows.len() as u16 {
            self.cursor.x = self.current_row_len();
        }
    }

    pub fn move_cursor(&mut self, key: EditorKey)  {
        match key {
            EditorKey::Left => {
                if self.cursor.x != 0 {
                    self.cursor.x -= 1;
                } else if self.cursor.y > 0  {
                    self.cursor.y -= 1;
                    self.cursor.x =  self.current_row_len();
                }
                self.cursor.x = self.cursor.x.saturating_sub(1);
            },
            EditorKey::Right => {
                if self.cursor.y < self.rows.len() as u16 {
                    let ind = self.cursor.y as usize;
                    if  self.cursor.x  <  self.rows[ind].len() as u16 {
                        self.cursor.x += 1;
                    } else if self.cursor.y < self.rows.len() as u16{
                        self.cursor.y += 1;
                        self.cursor.x = 0;
                    }
                }
            } ,
            EditorKey::Up => {
                self.cursor.y  = self.cursor.y.saturating_sub(1);
            },
            EditorKey::Down if self.cursor.y < self.rows.len() as u16  => self.cursor.y +=1,
            _ => {}
        }

        self.cursor.x = self.cursor.x.min(self.current_row_len());
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor.y == self.rows.len() as u16 {
            self.rows.push(Row::new("".to_string()));
        }

        self.rows[self.cursor.y as usize].insert_char(self.cursor.x as usize, c);
        self.cursor.x += 1;
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.scroll();
        self.screen.clear()?;
        self.screen.draw_row(&self.rows, self.rowoff, self.coloff)?;

        if !self.status_msg.is_empty() && self.status_time.elapsed() > Duration::from_secs(5) {
                self.status_msg.clear();
        }

        self.screen.draw_status_bar(format!("{:20} - {} lines ", self.filename, self.rows.len()),
                                    format!("{}/{}",self.cursor.y, self.rows.len()),
        self.status_msg.clone())
    }

    fn scroll(&mut self) {
        let bounds = self.screen.bounds();

        self.render_x = if self.cursor.y < (self.rows.len() as u16) {
            self.rows[self.cursor.y as usize].cx_to_rx(self.cursor.x)
        } else {
            0
        };

        if self.cursor.y < self.rowoff  {
            self.rowoff = self.cursor.y;
        }
        if self.cursor.y >= self.rowoff  + bounds.y {
            self.rowoff = self.cursor.y - bounds.y + 1;
        }

        if self.render_x < self.coloff {
            self.coloff = self.render_x;
        }
        if self.render_x >= self.coloff + bounds.x {
            self.coloff = self.render_x - bounds.x + 1
        }
    }

    pub fn current_row_len(&self) -> u16 {
        if self.cursor.y  >= (self.rows.len() as u16) {
            0
        } else {
            self.rows[self.cursor.y as usize].len() as u16
        }
    }

    // pub fn set_status_msg<T: Into<String>>(&mut self, msg: T) {
    //     self.status_time = Instant::now();
    //     self.status_msg = msg.into();
    // }



}
