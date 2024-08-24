use std::io;
use io::Result;

mod keyboard;
mod input;
mod output;
mod editor;

use crossterm::{ terminal};
use crate::editor::Editor;

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    let editor = Editor::new()?;
    loop {
        if editor.refresh_screen().is_err() {
            editor.die("Clear Screen");
        }
        if editor.process_keypress() {
            break;
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}






//
// c = None;
// match  poll(Duration::from_millis(100)) {
// Ok(true) => {
//
// if let Ok(event) = read() {
// if let Key(key_event) = event {
// c = Some(key_event);
// }
// } else {
// die("read error");
// }
// }
// Ok(false) => {}
// _ =>  die("poll error")
// }
//
// if let Some(c) = c {
// if c.code == KeyCode::Char('q') && c.modifiers.contains(KeyModifiers::CONTROL) {
// break;
// } else {
// println!("{c:?}\r");
// }
// } else {
// println!("No Key\r");
// }
