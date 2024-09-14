use std::slice::Iter;
use crossterm::style::Color;
use crate::editor_syntax::*;

const KILO_TAB_STOP : usize =  8;


#[derive(Copy, Clone, PartialEq)]
pub enum Highlight {
    Normal,
    Number,
    Match,
}

impl  Highlight {
    pub fn syntax_to_color(&self) -> Color {
        match  self {
            Highlight::Normal => Color::White,
            Highlight::Number => Color::Red,
            Highlight::Match => Color::Blue,
        }
    }
}
pub struct Row {
    pub chars: String,
    pub render: String,
    pub hl:  Vec<Highlight>,
    pub saved_hl:  Vec<Highlight>,
}

impl Row {
    pub fn new(chars: String, flags: EditorFlags) -> Self {
       let mut result = Self {
            chars,
            render: String::new(),
            hl: Vec::new(),
            saved_hl: Vec::new(),
        };
        result.render_row(flags);
        result
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

    pub fn rx_to_cx(&self, rx: usize) -> u16 {
        let mut cur_rx = 0;

        for (cx, c) in self.chars.chars().enumerate() {
            if c == '\t' {
                cur_rx += (KILO_TAB_STOP - 1) - (cur_rx % KILO_TAB_STOP);
            }
            cur_rx += 1;
            if cur_rx > rx {
                return cx as u16;
            }
        }
        self.chars.len() as u16
    }


    pub fn insert_char(&mut self, at: usize, c: char, flags: EditorFlags) {
        if at >= self.chars.len()   {
            self.chars.push(c)
        } else {
            self.chars.insert(at, c);
        }
        self.render_row(flags);
    }

    pub fn del_char(&mut self, at: usize, flags: EditorFlags) -> bool {
        if at >=  self.chars.len() {
           return false;
        }
        self.chars.remove(at);
        self.render_row(flags);
        true
    }

    pub fn split(&mut self, at: usize, flags: EditorFlags) -> String {
        let result = self.chars.split_off(at);
        self.render_row(flags);
        result
    }

    pub fn append_string(&mut self, s: &str, flags: EditorFlags) {
        self.chars.push_str(s);
        self.render_row(flags);
    }

    pub fn render_row(&mut self, flags:EditorFlags) {
        let mut render = String::new();
        let mut idx = 0;
        for c in self.chars.chars() {
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
        self.render = render;
        self.update_syntax(flags);
    }

    fn update_syntax(&mut self, flags: EditorFlags) {
        self.hl = vec![Highlight::Normal; self.render.len()];
        if flags == 0 {
            return
        }

        let mut prev_sep = false;
        let row_iter = self.render.chars().enumerate();
        for (i, c) in row_iter {
            let prev_hl = if i > 0 {
                self.hl[i - 1]
            } else {
                Highlight::Normal
            };

            if flags & highlightflags::NUMBERS != 0 &&
                (c.is_ascii_digit() && (prev_sep || prev_hl == Highlight::Number)
                    || (c == '.' && prev_hl == Highlight::Number)) {
                self.hl[i] = Highlight::Number;
                prev_sep = false;
                continue;
            }

            prev_sep = c.is_separator();
        }
    }

    pub fn highlight_match(&mut self, start: usize, len: usize ) {
        self.saved_hl = self.hl.clone();
        for c in self.hl[start..start+len].iter_mut() {
            *c = Highlight::Match;
        }
    }

    pub fn reset_match(&mut self) {
        self.hl = self.saved_hl.clone();
        self.saved_hl.clear();
    }

    pub fn iter_highlight(&self, start: usize, end: usize) ->  Iter<Highlight>  {
        self.hl[start..end].iter()
    }
}




trait Separator { fn is_separator(&self) -> bool; }

impl Separator for char {
    fn is_separator(&self) -> bool {
        matches!(self, ' ' | ',' | '.' | '(' | ')' | '+' | '-' | '/' | '*' | '=' | '~' |
          '%' | '<' | '>' | '[' | ']' | '{' | '}' | ';')
    }
}