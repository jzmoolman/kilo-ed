use std::io;
use io::Result;

mod editor;
mod keyboard;
mod screen;
mod row;
mod editor_syntax;

use crate::editor::Editor;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let mut editor = if args.len() >= 2 {
        Editor::with_file(args.nth(1).unwrap())?
    } else {
        Editor::new()?
    };
    editor.start()?;
    Ok(())
}