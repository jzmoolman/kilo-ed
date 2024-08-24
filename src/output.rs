use std::io::{stdout, Write, Result, Stdout};
use crossterm::{cursor, style, terminal, QueueableCommand};
use errno::errno;

pub fn editor_draw_row(stdout: &mut Stdout) -> Result<()> {
    for row in 0..24 {
        stdout
            .queue(cursor::MoveTo(0, row))?
            .queue(style::Print('~'))?;
    }
    Ok(())
}

pub fn clear_screeen(stdout: &mut Stdout) -> Result<()> {
    stdout
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::MoveTo(0,0))?
        .flush()
}
pub fn editor_refresh_screen() -> Result<()> {
    let mut stdout = stdout();

    clear_screeen(&mut stdout)?;
    editor_draw_row(&mut stdout)?;
    stdout
        .queue(cursor::MoveTo(0,0))?
        .flush()
}

pub fn die<S: Into<String>>(message : S) {
    let mut stdout = stdout();
    let _ = clear_screeen(&mut stdout);
    let _ =  terminal::disable_raw_mode();
    eprintln!("{}: {}", message.into(), errno());
    std::process::exit(1);
}