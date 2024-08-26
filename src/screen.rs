use std::io::{stdout, Stdout, Write};
use std::io::Result;
use crossterm::{cursor, style, terminal, QueueableCommand};

use kilo_ed::*;

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

    pub fn draw_row(&mut self, rows: &[String], rowoff: u16, coloff: u16) -> Result<()> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        for row in 0..self.height {
            let filerow = (row  + rowoff) as usize;
            if filerow >= rows.len() {
                if rows.is_empty() && row == self.height /3 {
                    let mut welcome = format!("Kilo Editor -- version {VERSION}");
                    welcome.truncate(self.width as usize);
                    if welcome.len() < self.width as usize {
                        let leftmost = (self.width as usize - welcome.len())/2;
                        self.stdout.queue(cursor::MoveTo(0,row))?
                            .queue(style::Print('~'))?
                            .queue(cursor::MoveTo(leftmost as u16,row))?
                            .queue(style::Print(welcome))?;
                    } else {
                        self.stdout
                        .queue(cursor::MoveTo(0,row))?
                        .queue(style::Print(welcome))?;
                    }
                } else {
                    self.stdout
                        .queue(cursor::MoveTo(0, row))?
                        .queue(style::Print('~'))?;
                }
            } else {
                let mut len = rows[filerow].len();
                if (len as u16) < coloff {
                    continue;
                }
                len -= coloff as usize;
                let start = coloff as usize;
                let end = start +  if len > self.width as usize {
                    self.width as usize
                } else {
                    len
                };

                self.stdout
                    .queue(cursor::MoveTo(0,row))?
                    .queue(style::Print(rows[filerow as usize][start..end].to_string()))?;
            }
        }
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

    pub fn move_to(&mut self, pos: &Position, rowoff: u16, coloff: u16) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(pos.x - coloff, pos.y - rowoff))?;
        Ok(())
    }

    pub fn bounds(&self) -> Position {
        Position {
            x: self.width,
            y: self.height
        }
    }
}

