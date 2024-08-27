use std::io::{stdout, Stdout, Write};
use std::io::Result;
use crossterm::{cursor, style, terminal, QueueableCommand};
use crossterm::style::Color::{Black, White};
use crossterm::style::{Colors, ResetColor};
use kilo_ed::*;
use crate::row::*;

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
            height: height-1,
        })
    }

    pub fn draw_row(&mut self, rows: &[Row], rowoff: u16, coloff: u16) -> Result<()> {
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
                let mut len = rows[filerow].render_len();
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
                    .queue(style::Print(rows[filerow].render[start..end].to_string()))?;
            }
        }
        Ok(())
    }

    pub fn draw_status_bar<T: Into<String>>(&mut self, left: T, right: T) -> Result<()> {
        let left = left.into();
        let right = right.into();
        let left_width = left.len();
        let right_width = right.len();
        let screen_width = self.width;

        let status = format!("{left:0$}", left_width.min(screen_width as usize));
        let mut rstatus = String::new();
        if status.len() < screen_width as usize - right_width {
            let mut len = status.len() as u16;
            while len < screen_width {
                if screen_width - len  < right_width as u16 {
                    rstatus.push_str(right.as_str());
                    break;
                } else {
                    rstatus.push(' ');
                    len += 1;
                }
            }
        }

        // let status = format!("{:-20} - {count}", filename.into());
        self.stdout
            .queue(cursor::MoveTo(0,self.height-1))?
            .queue(style::SetColors(Colors::new(Black, White)))?
            .queue(style::Print(format!("{status}{rstatus}")))?
            .queue(style::ResetColor)?;
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

    pub fn move_to(&mut self, pos: &Position, render_x:u16,  rowoff: u16, coloff: u16) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(render_x - coloff, pos.y - rowoff))?;
        Ok(())
    }

    pub fn bounds(&self) -> Position {
        Position {
            x: self.width,
            y: self.height
        }
    }
}

