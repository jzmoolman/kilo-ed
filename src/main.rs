mod keyboard;
mod input;

use std::io;
use crossterm::{ terminal};
use io::Result;
use errno::errno;

use crate::input::editor_process_keypress;

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    loop {
        if editor_process_keypress() {
            break;
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}

fn die<S: Into<String>>(message : S) {
    let _ =  terminal::disable_raw_mode();
    eprintln!("{}: {}", message.into(), errno());
    std::process::exit(1);
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
