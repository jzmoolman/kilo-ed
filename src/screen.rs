use std::io::{stdout, Stdout, Write};
use std::io::Result;
use crossterm::{cursor, style, terminal, QueueableCommand};

pub struct Screen {
    stdout: Stdout,
    width: u16,
    height: u16,
}

impl Screen {
    pub fn new() -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Self {
            stdout: stdout(),
            width,
            height,
        })
    }

    pub fn draw_row(&mut self) -> Result<()> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        for row in 0..self.height {
            if row == self.height /3 {
                let mut welcome = format!("Kilo Editor -- version {VERSION}");
                welcome.truncate(self.width as usize);
                self.stdout
                    .queue(cursor::MoveTo(0,row))?
                    .queue(style::Print(welcome))?;

            } else {
                self.stdout
                    .queue(cursor::MoveTo(0, row))?
                    .queue(style::Print('~'))?;
            }
        }
        self.stdout.queue(cursor::MoveTo(0,0))?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0,0))?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()
    }

    pub fn refresh(&mut self) -> Result<()> {

        self.clear()?;
        self.draw_row()?;
        self.stdout
            .queue(cursor::MoveTo(0,0))?;
        Ok(())
    }

    pub fn cursor_position(&self) -> Result<(u16, u16)> {
        cursor::position()
    }
}

