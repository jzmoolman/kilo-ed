use std::io;
use io::Result;

mod editor;
mod keyboard;
mod screen;

use crate::editor::Editor;

fn main() -> Result<()> {
    let mut editor = Editor::new()?;
    editor.start()?;
    Ok(())
}