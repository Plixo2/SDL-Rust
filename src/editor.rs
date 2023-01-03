use std::ops::Range;

use sdl2::keyboard::Keycode;

use crate::{font::FontRenderer, ui::{self, UIShader}};

pub struct Editor {
    lines: Vec<String>,
    cursor: Cursor,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            lines: vec![String::from("")],
            cursor: Cursor::default(),
        }
    }
}

struct Cursor {
    line: u32,
    char: u32,
}

impl Cursor {}

impl Default for Cursor {
    fn default() -> Self {
        Cursor { line: 0, char: 0 }
    }
}

impl Editor {
    pub fn handle_input(&mut self, key: Keycode) {
        match key {
            Keycode::Up => {
                self.cursor_vertical(-1);
            }
            Keycode::Down => {
                self.cursor_vertical(1);
            }
            Keycode::End => {
                self.cursor_horizontal(20000);
                self.cursor_clamp_horizontal();
            }
            Keycode::Home => {
                self.cursor_horizontal(-20000);
            }
            Keycode::Left => {
                self.cursor_clamp_horizontal();
                self.cursor_horizontal(-1);
            }
            Keycode::Right => {
                self.cursor_clamp_horizontal();
                self.cursor_horizontal(1);
                self.cursor_clamp_horizontal();
            }
            Keycode::Delete => {
                self.cursor_clamp_horizontal();
                self.delete(
                    self.cursor.line,
                    self.cursor.char as i32..(self.cursor.char + 1) as i32,
                );
            }
            Keycode::Backspace => {
                self.cursor_clamp_horizontal();
                self.delete(
                    self.cursor.line,
                    (self.cursor.char as i32 - 1)..self.cursor.char as i32,
                );
                self.cursor_horizontal(-1);
            }
            _ => {}
        }
        self.cursor_clamp_vertical();
    }

    fn cursor_horizontal(&mut self, amount: i32) {
        self.cursor.char = (self.cursor.char as i32 + amount).max(0) as u32;
    }
    fn cursor_vertical(&mut self, amount: i32) {
        self.cursor.line = (self.cursor.line as i32 + amount).max(0) as u32;
    }
    fn cursor_clamp_horizontal(&mut self) {
        if let Some(line) = self.lines.get(self.cursor.line as usize) {
            self.cursor.char = self.cursor.char.clamp(0, line.len() as u32);
        }
    }
    fn cursor_clamp_vertical(&mut self) {
        self.cursor.line = self.cursor.line.clamp(0, self.lines.len() as u32 - 1);
    }
    pub fn delete(&mut self, line: u32, range: Range<i32>) {
        if let Some(line) = self.lines.get_mut(line as usize) {
            let start = range.start.clamp(0, line.len() as i32) as usize;
            let end = range.end.clamp(0, line.len() as i32) as usize;

            let left = line[..start].to_owned();
            let right = line[end..].to_owned();
            *line = format!("{}{}", left, right);
        }
    }

    pub fn handle_text(&mut self, text: String) {
        let len = text.len();
        let char_at = self.cursor.char as usize;
        let line = self
            .lines
            .get(self.cursor.line as usize)
            .expect("line at cursor");
        let left = if char_at >= line.len() {
            line.to_owned()
        } else {
            line[..char_at].to_owned()
        };
        let right = line[char_at.min(line.len())..].to_owned();
        let line = format!("{}{}{}", left, text, right);
        let line_ref = self
            .lines
            .get_mut(self.cursor.line as usize)
            .expect("line at cursor");
        *line_ref = line;
        self.cursor.char += len as u32;
    }
    pub fn render(&mut self, gui: &mut UIShader, text_renderer: &mut FontRenderer) {
        for (index, line) in self.lines.iter().enumerate() {
            text_renderer.draw(
                &format!("{}:", index),
                5.0,
                index as f32 * 20.0 + 50.0,
                ui::GRAY,
            );
            if index == self.cursor.line as usize {
                gui.draw_rect(30.0, index as f32 * 20.0 + 30.0, 1000.0, 20.0, 0.0, ui::Color::rgba(255, 255, 255, 10));
            }
            text_renderer.draw(line, 30.0, index as f32 * 20.0 + 50.0, ui::WHITE);
        }
        if let Some(line_ref) = self.lines.get(self.cursor.line as usize) {
            let var_name = if self.cursor.char >= line_ref.len() as u32 {
                line_ref.to_owned()
            } else {
                line_ref[..self.cursor.char as usize].to_owned()
            };
            let width = text_renderer.text_width(&var_name);
            gui.draw_rect(29.0 + width, self.cursor.line as f32 * 20.0 + 30.0, 2.0, 20.0, 0.0, ui::GRAY);
        }
    }
    pub fn paste_lines(&mut self, mut lines: Vec<String>) {
        let mut new_lines = vec![];

        self.clamp_cursor();

        //let mut lines = lines;

        let line_cursor = self.cursor.line as usize;

        for (index, line) in self.lines.iter().enumerate() {
            if index == line_cursor {
                new_lines.append(&mut lines);
            }
            new_lines.push(line.clone());
        }

        self.lines = new_lines;
    }
    fn clamp_cursor(&mut self) {
        let lines = self.cursor.line.clamp(0, (self.lines.len() - 1) as u32);
        let line = self.lines.get(0).expect("line at cursor");
        self.cursor.line = lines;
        self.cursor.char = self.cursor.char.clamp(0, line.len() as u32);
    }
}

#[test]
fn test_editor() {
    let mut editor = Editor::default();
    let test_str = "Hello World";
    editor.lines = vec![String::from(test_str)];
    editor.delete(0, 0..(test_str.len() + 10) as i32);
    let first_line = editor.lines.get(0).expect("first line");
    assert_eq!(first_line, &String::from(""));

    editor.lines = vec![String::from(test_str)];
    editor.delete(0, 5..test_str.len() as i32);
    let first_line = editor.lines.get(0).expect("first line");
    assert_eq!(first_line, &String::from("Hello"));

    editor.lines = vec![String::from(test_str)];
    editor.delete(0, 20..(test_str.len() + 10) as i32);
    let first_line = editor.lines.get(0).expect("first line");
    assert_eq!(first_line, &String::from("Hello World"));
}
