
use std::io::{stdout, Result, Stdout, Write};
use crossterm::{cursor, style, terminal, QueueableCommand};
use crossterm::event::{read, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::Event::Key;
use errno::errno;

pub struct Editor {
    width: u16,
    height: u16,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Self {
            width,
            height,

        })
    }

    // keyboard

    pub fn process_keypress(&self) -> bool {
        let c = self.read_key();
        match c {
            Ok(KeyEvent{
                   code: KeyCode::Char('q'),
                   modifiers: KeyModifiers::CONTROL, ..
               }) => true,
            _ => false
        }
    }

    pub fn read_key(&self) -> Result<KeyEvent> {
        loop {
            if let Ok(event) = read() {
                if let Key(key_event) = event {
                    return Ok(key_event);
                }
            } else {
                self.die("read");
            }
        }

    }

    // output

    pub fn draw_row(&self,stdout: &mut Stdout) -> Result<()> {
        for row in 0..self.height {
            stdout
                .queue(cursor::MoveTo(0, row))?
                .queue(style::Print('~'))?;
        }
        Ok(())
    }

    pub fn clear_screeen(&self,stdout: &mut Stdout) -> Result<()> {
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0,0))?
            .flush()
    }
    pub fn refresh_screen(&self) -> Result<()> {
        let mut stdout = stdout();

        self.clear_screeen(&mut stdout)?;
        self.draw_row(&mut stdout)?;
        stdout
            .queue(cursor::MoveTo(0,0))?
            .flush()
    }

    pub fn die<S: Into<String>>(&self, message : S) {
        let mut stdout = stdout();
        let _ = self.clear_screeen(&mut stdout);
        let _ =  terminal::disable_raw_mode();
        eprintln!("{}: {}", message.into(), errno());
        std::process::exit(1);
    }



}
