use std::io::{stdout, Stdout, Write};
use std::io::Result;
use crossterm::{cursor, style, terminal, QueueableCommand};

use kilo_ed::*;

const KILO_TAB_STOP : usize =  8;

pub struct Screen {
    stdout: Stdout,
    width: u16,
    height: u16,
}

pub struct Row {
    chars: String,
    render: String,
}

impl Row {
    pub fn new(chars: String) -> Self {
        let mut render = String::new();
        let mut idx = 0;
        for c in chars.chars() {
            match c {
                '\t' => {
                    render.push(' ');
                    idx += 1;
                    while idx % KILO_TAB_STOP != 0 {
                        render.push(' ');
                        idx += 1;
                    }
                }
                _ => {
                    idx += 1;
                    render.push(c);
                },
            }

        }
        Self {
            chars,
            render,
        }
    }
    pub fn len(&self) -> usize {
        self.chars.len()
    }
    pub fn render_len(&self) -> usize {
        self.render.len()
    }

    pub fn cx_to_rx(&self, cx: u16) -> u16 {
        let mut rx = 0;
        for c in self.chars.chars().take(cx as usize) {
            if c == '\t' {
                rx += (KILO_TAB_STOP -1) - (rx % KILO_TAB_STOP);
            }
            rx += 1;
        }
        rx as u16
    }
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

