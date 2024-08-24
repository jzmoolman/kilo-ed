use crossterm::event::{read, KeyEvent};
use crossterm::event::Event::Key;

use kilo_ed::EditorResult;
use kilo_ed::StdResult;

pub struct Keyboard;

impl Keyboard {
    pub fn read_key(&self) -> StdResult<KeyEvent, EditorResult> {
        loop {
            if let Ok(event) = read() {
                if let Key(key_event) = event {
                    return Ok(key_event);
                }
            } else {
                return Err(EditorResult::KeyReadFail);
            }
        }
    }
}

