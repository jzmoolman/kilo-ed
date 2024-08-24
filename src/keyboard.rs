use std::io;
use crossterm::event::{read, KeyEvent};
use crossterm::event::Event::Key;
use io::Result;

use crate::*;
pub fn editor_read_key() -> Result<KeyEvent> {
    loop {
        if let Ok(event) = read() {
            if let Key(key_event) = event {
                return Ok(key_event);
            }
        } else {
            die("read");
        }
    }

}