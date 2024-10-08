use std::io:: Result;
use std::path::Path;
use std::time::{Instant, Duration};
use crossterm::{terminal};
use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers};
use errno::errno;
use crate::keyboard::*;
use crate::screen::*;
use crate::row::*;
use crate::editor_syntax::*;

use kilo_ed::*;
use crate::editor::SearchDirection::{Backward, Forward};

enum PromptKey {
    Enter,
    Escape,
    Next,
    Previous,
    Char(char),
}

enum SearchDirection {
    Forward,
    Backward,
}

const KILO_QUIT_TIMES: usize = 3;

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
    dirty: bool,
    rows: Vec<Row>,
    rowoff: u16,
    coloff: u16,
    quit_time: usize,
    last_match: Option<usize>,
    direction: SearchDirection,
    saved_hl: Option<usize>,
    hldb: Vec<EditorSyntax>,
    syntax_ind: Option<usize>,   // index into hldb
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
        let filename: String = filename.into();
        let hldb = EditorSyntax::new();
        let syntax_ind = Editor::find_highlight(&hldb, filename.as_str());
        let syntax_: EditorSyntax;
        let syntax = if let Some(ind) = syntax_ind  {
            syntax_ = hldb[ind].clone();
            Some(&syntax_)
        } else {
            None
        };

        Ok(Self {
            filename,
            status_msg: String::from("HELP: Ctrl-S = Save | Ctrl-Q = Quit"),
            status_time: Instant::now(),
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Position::default(),
            render_x: 0,
            dirty: false,
            rows: if data.is_empty() {
                Vec::new()
            } else {
                let mut rows = Vec::new();
                for line in  data {
                     let row = Row::new(line.to_string(), syntax);
                     rows.push(row);
                };
                if rows.last().unwrap().len() == 0 {
                    rows.pop();
                }
                rows
            },
            rowoff: 0,
            coloff: 0,
            quit_time: KILO_QUIT_TIMES,
            last_match: None,
            direction: Forward,
            saved_hl: None,
            hldb,
            syntax_ind,
        })
    }

    // keyboard
    pub fn process_keypress(&mut self) -> Result<bool> {
        if let Ok(c) = self.keyboard.read() {
            match c {
               KeyEvent {
                   code: KeyCode::Char('q'),
                   modifiers: KeyModifiers::CONTROL, ..
               } => {
                   if self.dirty && self.quit_time > 0 {
                       self.set_status_msg(
                           format!("Warning!!!  File has unsaved changes.\
                                   Press Ctrl-Q {} more time to quit", self.quit_time));
                       self.quit_time -= 1;
                       return Ok(false)
                   } else  {
                       return Ok(true)
                   }
               },
               KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::CONTROL, ..
               } => self.save(),
                KeyEvent {
                    code: KeyCode::Char('f'),
                    modifiers: KeyModifiers::CONTROL, ..
                } => self.find(),
               KeyEvent {
                   code: KeyCode::Char('l'),
                   modifiers: KeyModifiers::CONTROL, ..
               } => {}, // DO NOTHING
               KeyEvent {
                   code: KeyCode::Char('h'),
                   modifiers: KeyModifiers::CONTROL, ..
               } => self.del_char(),

                KeyEvent {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::NONE, ..
                } => self.insert_char(key),
               KeyEvent {
                   code: KeyCode::Tab,
                   modifiers: KeyModifiers::NONE, ..
               } => self.insert_char('\t'),
                KeyEvent {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::SHIFT, ..
                } => self.insert_char(key),

               KeyEvent { code, .. } => match code {
                   KeyCode::Delete => {
                       self.move_cursor(EditorKey::Right);
                       self.del_char();
                   },
                   KeyCode::Backspace => {
                       self.del_char();
                   },
                   KeyCode::Enter => self.insert_newline(),
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
                   _ => { self.set_status_msg("NOTHING")}
               }
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
        self.quit_time = KILO_QUIT_TIMES;
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
                    self.cursor.x = self.current_row_len();
                }
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
        let syntax = self.get_syntax_data();  // get a clone of syntax
        let syntax = if let Some(syntax) = &syntax {
            Some(syntax)
        } else {
            None
        };


        if self.cursor.y == self.rows.len() as u16 {
            self.insert_row(self.cursor.y as usize, String::new());
        }

        self.rows[self.cursor.y as usize].insert_char(self.cursor.x as usize, c, syntax);
        self.cursor.x += 1;
        self.dirty = true;
    }

    pub fn del_char(&mut self) {

        if self.cursor.y == self.rows.len() as u16 {
            return;
        }
        if self.cursor.x == 0 && self.cursor.y == 0 {
            return;
        }

        let syntax: Option<EditorSyntax> = self.get_syntax_data();
        let syntax = if let Some(syntax) = &syntax {
            Some(syntax)
        } else {
            None
        };

        let current_row = self.cursor.y as usize;
        if self.cursor.x > 0 {
            if self.rows[current_row].del_char(self.cursor.x as usize-1, syntax) {
                self.cursor.x -= 1;
                self.dirty = true;
           }
        } else {
            self.cursor.x = self.rows[current_row-1].len() as u16;
            if let Some(row) = self.del_row(current_row) {
                self.rows[current_row-1].append_string(&row, syntax);
                self.cursor.y -= 1;
                self.dirty = true;
            }
        }
    }

    pub fn insert_row(&mut self, at: usize, s: String) {
        if at > self.rows.len() {
           return;
        }
        let syntax = self.get_syntax_data();
        let syntax = if let Some(syntax) = &syntax {
            Some(syntax)
        } else {
            None
        };
        self.rows.insert(at,Row::new(s, syntax));
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let syntax = self.get_syntax_data();
        let syntax = if let Some(syntax) = &syntax {
            Some(syntax)
        } else {
            None
        };

        let row = self.cursor.y as usize;
        if self.cursor.x == 0 {
            self.insert_row(row, "".to_string());
        } else {
            let new_row_str = self.rows[row].split(self.cursor.x as usize, syntax);
            self.insert_row(row+1, new_row_str);
        }
        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    pub fn del_row(&mut self, at: usize) -> Option<String> {
        if at > self.rows.len() {
            None
        } else{
            self.dirty = true;
            Some(self.rows.remove(at).chars)
        }
    }


    pub fn refresh_screen(&mut self) -> Result<()> {
        self.scroll();
        self.screen.clear()?;
        self.screen.draw_row(&self.rows, self.rowoff, self.coloff)?;

        if !self.status_msg.is_empty() && self.status_time.elapsed() > Duration::from_secs(5) {
                self.status_msg.clear();
        }

        self.screen.draw_status_bar(
            format!("{:20} - {} lines {}",
                    if self.filename.is_empty() {"[No Name]"} else { &self.filename },
                    self.rows.len(),
                    if self.dirty {
                "{Modified}" } else { "" }
            ),
            format!("{} | {}/{}",
                if let Some(ft)= self.syntax_ind {
                    &self.hldb[ft].filetype
                } else {
                    "No FileType"
                }
                ,self.cursor.y, self.rows.len()),
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

    pub fn rows_to_string(&self) -> String {
        let mut buf  = String::new();
        for row in &self.rows {
            buf.push_str(row.chars.as_str());
            buf.push('\n');
        }
        buf
    }

    pub fn save(&mut self) {
        if self.filename.is_empty() {
            if let Some(filename) = self.prompt("Save as".to_string(), None) {
                self.filename = filename;
            } else {
                self.set_status_msg("Save aborted");
                return;
            };
            self.select_syntax_highlight();
            //self.syntax = Editor::find_syntax(&self.hldb, &self.filename);
            // self.syntax = Editor::select_syntax_highlight(&self.hldb, &self.filename);
        }

        let buf = self.rows_to_string();
        let len = buf.len();
        if std::fs::write(&self.filename, &buf).is_ok() {
            self.set_status_msg(&format!("{len} bytes written to disk"));
            self.dirty = false;
        } else {
            self.set_status_msg(&format!("Can;t save I/O error: {}", errno()));
        }
    }

    fn prompt(&mut self, prompt_str: String, _callback: Option<fn(&mut Editor, &str, PromptKey)>) -> Option<String> {
        let mut buffer = String::from("");

        loop {
            self.set_status_msg(format!("{}: {}", prompt_str, buffer));
            let _ = self.refresh_screen();
            let _ = self.screen.flush();
            if let Ok(c) = self.keyboard.read() {
                let mut prompt_key: Option<PromptKey> = None;
                match c {
                    KeyEvent {
                        code: KeyCode::Esc,
                        ..
                    } =>  {
                        if let Some(callback) = _callback {
                            callback(self, &buffer, PromptKey::Escape);
                        }
                        self.set_status_msg("");
                        return  None;
                    },
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } | KeyEvent {
                        code : KeyCode::Char('h'),
                        modifiers: KeyModifiers::CONTROL,
                        ..}

                        =>  {
                        buffer.pop();
                    }

                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,..
                    } => {
                        if let Some(callback) = _callback {
                            callback(self, &buffer, PromptKey::Enter);
                        }
                        self.set_status_msg("");
                        return Some(buffer);
                    }
                    KeyEvent {
                        code: KeyCode::Left | KeyCode::Up,
                        ..} => {
                            if let Some(callback) = _callback {
                               callback(self, &buffer, PromptKey::Previous);
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Right | KeyCode::Down,
                        ..} => {
                        if let Some(callback) = _callback {
                            callback(self, &buffer, PromptKey::Next);
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers,
                        ..
                    } => {
                        if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                            prompt_key = Some(PromptKey::Char(c));
                            buffer.push(c);
                        }
                    }
                    _ =>  {}
                }
                if let Some(callback) = _callback {
                    if let Some(key) = prompt_key {
                        callback(self, &buffer, key);
                    }
                }
            };
        }
    }

   fn find_callback(&mut self, query: &str, event: PromptKey) {
       if let Some(saved_hl) = self.saved_hl {
         self.rows[saved_hl].reset_match();
           self.saved_hl = None;
       }

       match event {
          PromptKey::Escape | PromptKey::Enter => {
              self.last_match = None;
              self.direction = Forward;
              return;
          }
          PromptKey::Next => self.direction = Forward,
          PromptKey::Previous => self.direction = Backward,
          _ => {
              self.last_match = None;
              self.direction = Forward;
          }
       }

       let mut current = if let Some(ind) = self.last_match {
           ind
       } else {
           self.direction = Forward;
           0
       };

       for _ in 0..self.rows.len() {
           match self.direction {
               Forward => {
                   current += 1;
                   if current >= self.rows.len() {
                       current = 0;
                   }
               },
               Backward => {
                   if current == 0 {
                       current = self.rows.len()-1;
                   } else {
                       current -= 1;
                   }
               }
           }

           if let Some(ind) = self.rows[current].render.find(query) {
               self.last_match = Some(current);
               self.cursor.y = current as u16;
               self.cursor.x = self.rows[current].rx_to_cx(ind);
               self.rowoff = self.rows.len() as u16;
               self.rows[current].highlight_match(ind, query.len());
               self.saved_hl = Some(current);
               break;
           }
       }
   }

    pub fn find(&mut self) {
        let save_cursor= self.cursor;
        let save_coloff = self.coloff;
        let save_rowoff= self.rowoff;

        if self.prompt("Search(ESC/Arrows/Enter)".to_string(), Some(Editor::find_callback)).is_none() {
            self.cursor = save_cursor;
            self.coloff = save_coloff;
            self.rowoff = save_rowoff;
        };
    }

    pub fn set_status_msg<T: Into<String>>(&mut self, msg: T) {
        self.status_time = Instant::now();
        self.status_msg = msg.into();
    }

    pub fn get_syntax_data(&self) -> Option<EditorSyntax> {
        if let Some(ind) = self.syntax_ind {
            Some(self.hldb[ind].clone())
        } else {
            None
        }
    }

    fn select_syntax_highlight(&mut self) {
        let old_syntax = self.syntax_ind;
        self.syntax_ind = Editor::find_highlight(&self.hldb, self.filename.as_str());
        if self.syntax_ind != old_syntax {
            let syntax = self.get_syntax_data();
            let syntax = if let Some(syntax) = &syntax {
                Some(syntax)
            } else {
                None
            };

            for row in self.rows.iter_mut() {
                row.update_syntax(syntax);
            }
        }
    }

    fn find_highlight(hldb: &[EditorSyntax], filename: &str) -> Option<usize>{
        if filename.is_empty() {
            return None;
        }

        match filename.split(".").collect::<Vec<&str>>().last() {
            None => None,
            Some(ext ) => {
                for (j,entry ) in hldb.iter().enumerate() {
                    for ext_ in entry.filematch.iter() {
                        eprintln!("{ext}:{ext_}");
                        if ext == ext_ {
                            return Some(j);
                        }
                    }
                }
                None
            }
        }
    }
}