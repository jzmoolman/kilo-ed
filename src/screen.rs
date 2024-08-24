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
        for row in 0..self.height {
            self.stdout
                .queue(cursor::MoveTo(0, row))?
                .queue(style::Print('~'))?;
        }
        self.stdout.flush()
    }
    pub fn clear(&mut self) -> Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0,0))?
            .flush()
    }
    pub fn refresh(&mut self) -> Result<()> {

        self.clear()?;
        self.draw_row()?;
        self.stdout
            .queue(cursor::MoveTo(0,0))?
            .flush()
    }
}

