use {
    crate::{
        layout::RectExt,
        style::{ColorExt, StyleExt},
    },
    builder::builder_methods,
    tui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
        text::Span,
        widgets::{Block, StatefulWidget},
    },
};

#[derive(Default)]
pub struct TextInput<'a> {
    prompt: Option<Span<'a>>,
    block: Option<Block<'a>>,
    style: Style,
}

impl<'a> TextInput<'a> {
    builder_methods! {
        /// Sets the prompt displayed to the left of the input field.
        pub prompt: impl Into<Span<'a>> => prompt.into().into();

        pub block: Block<'a> => block.into();
        pub style: Style
    }
}
#[derive(Default)]
pub struct TextInputState {
    text: String,
    cursor: usize,
    offset: usize,
    focus: bool,
}

impl TextInputState {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn incr_cursor(&mut self) {
        self.cursor += self.text[self.cursor..]
            .char_indices()
            .nth(1)
            .unwrap_or_default()
            .0
    }

    pub fn decr_cursor(&mut self) {
        self.cursor = self.text[..self.cursor]
            .char_indices()
            .rev()
            .next()
            .unwrap_or_default()
            .0
    }

    pub fn insert(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);

        self.cursor += ch.len_utf8();
    }

    fn delete_current(&mut self) -> char {
        self.text.remove(self.cursor)
    }

    pub fn delete_left(&mut self) -> Option<char> {
        let old = self.cursor;
        self.decr_cursor();

        if self.cursor != old {
            self.delete_current().into()
        } else {
            None
        }
    }

    pub fn delete_right(&mut self) -> Option<char> {
        if self.cursor != self.text.len() {
            self.delete_current().into()
        } else {
            None
        }
    }

    pub fn focus(&mut self) {
        self.focus = true;
        self.cursor = self.text.len();
    }

    pub fn unfocus(&mut self) {
        self.focus = false;
    }

    fn adjust_offset(&mut self, area: Rect) {
        if self.cursor < self.offset {
            self.offset = self.cursor
        } else {
            let text = &self.text[self.offset..self.cursor];
            if width(text) > area.width as usize {
                let chars = &mut text.chars();
                while width(chars.as_str()) >= area.width as usize {
                    chars.next();
                }
                self.offset = self.cursor - chars.as_str().len()
            }
        }
    }

    fn draw_text(&self) -> (&str, &str) {
        self.text[self.offset..].split_at(self.cursor - self.offset)
    }
}

/// Input handling using the `ìnput` crate.
/// Requires the `handle-input` feature.
#[cfg(feature = "handle-input")]
impl TextInputState {
    fn left_char(&self) -> Option<char> {
        self.text[..self.cursor].chars().next_back()
    }

    fn right_char(&self) -> Option<char> {
        self.text[self.cursor..].chars().next()
    }

    fn delete_section_left(&mut self) {
        let pred = match self.predicate(Self::left_char) {
            Some(p) => p,
            None => return,
        };

        let idx = self.text[..self.cursor].find(pred).unwrap_or_default();

        self.text.replace_range(idx..self.cursor, "");
        self.cursor = idx;
    }

    fn delete_section_right(&mut self) {
        let pred = match self.predicate(Self::right_char) {
            Some(p) => p,
            None => return,
        };

        let end = self.text[self.cursor..]
            .find(|c| !pred(c))
            .map(|idx| self.cursor + idx)
            .unwrap_or_else(|| self.text.len());

        self.text.replace_range(self.cursor..end, "");
    }

    fn predicate(&self, ch: fn(&Self) -> Option<char>) -> Option<fn(char) -> bool> {
        ch(self).map(|ch| {
            if ch.is_whitespace() {
                char::is_whitespace
            } else if ch.is_ascii_punctuation() {
                |c: char| c.is_ascii_punctuation()
            } else {
                |ch| !is_word_boundary(ch)
            }
        })
    }

    /// Handles the given key and optionally returns a [Message](Message)
    pub fn handle_key(&mut self, key: input::Key) -> Option<Message> {
        use input::keys::KeyCode::*;

        match key.code {
            Enter => return Message::Confirm.into(),
            Esc => return Message::Cancel.into(),
            Char(c) => self.insert(c),
            Backspace if key.ctrl() => self.delete_section_left(),
            Backspace => {
                self.delete_left();
            }
            Delete if key.ctrl() => self.delete_section_right(),
            Delete => {
                self.delete_right();
            }
            Left => self.decr_cursor(),
            Right => self.incr_cursor(),
            _ => {}
        }

        None
    }
}

#[cfg(feature = "handle-input")]
fn is_word_boundary(ch: char) -> bool {
    ch.is_whitespace() || ch.is_ascii_punctuation()
}

#[cfg(feature = "handle-input")]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Message {
    Confirm,
    Cancel,
}

fn width(s: &str) -> usize {
    Span::raw(s).width()
}

impl<'a> StatefulWidget for TextInput<'a> {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.block.map_or(area, super::rendered_block(area, buf));

        let area = self
            .prompt
            .map_or(area, |prompt| draw_prompt(prompt, area, buf));

        state.adjust_offset(area);

        let (before, after) = state.draw_text();

        buf.set_style(Rect { height: 1, ..area }, self.style.underlined());

        let (x, y) = buf.set_span(area.x, area.y, &before.into(), area.width);

        if state.focus {
            highlight_cursor(x, y, self.style, buf);
        }

        if x != area.right() {
            buf.set_span(x, y, &after.into(), area.width - (x - area.x));
        }
    }
}

fn draw_prompt(prompt: Span, area: Rect, buf: &mut Buffer) -> Rect {
    let (x, _) = buf.set_span(area.x, area.y, &prompt, area.width);
    area.shrink_left(x - area.x).shrink_left(1)
}

fn highlight_cursor(x: u16, y: u16, style: Style, buf: &mut Buffer) {
    let (fg, bg) = colors(style);

    buf.get_mut(x, y).set_bg(bg).set_fg(fg);
}

fn colors(style: Style) -> (Color, Color) {
    match (style.fg, style.bg) {
        (Some(fg), Some(bg)) => {
            if fg.is_bright() == bg.is_bright() {
                (fg, bg.invert())
            } else {
                (fg.invert(), bg.invert())
            }
        }
        (None, Some(bg)) => (bg, bg.invert()),
        (Some(fg), None) => (fg, fg.invert()),
        (None, None) => (Color::Black, Color::White),
    }
}
