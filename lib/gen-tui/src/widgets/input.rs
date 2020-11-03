use {
    builder::builder_methods,
    crate::{
        style::ColorExt,
        layout::RectExt
    },
    tui::{
        text::Span,
        style::{Style, Color, Modifier},
        layout::Rect,
        buffer::Buffer,
        widgets::{StatefulWidget, Block}
    }
};

#[derive(Default)]
pub struct TextInput<'a> {
    prompt: Option<Span<'a>>,
    block: Option<Block<'a>>,
    style: Style
}

impl <'a> TextInput<'a> {
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
    focus: bool
}

impl TextInputState {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn incr_cursor(&mut self) {
        self.cursor += self
            .text[self.cursor..]
            .char_indices()
            .skip(1)
            .next()
            .unwrap_or_default()
            .0
    }

    pub fn decr_cursor(&mut self) {
        self.cursor = self
            .text[..self.cursor]
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

    fn delete_current(&mut self) {
        self.text.remove(self.cursor);
    }

    pub fn delete_left(&mut self) {
        let old = self.cursor;
        self.decr_cursor();

        if self.cursor != old {
           self.delete_current()
        }
    }

    pub fn delete_right(&mut self) {
        let old = self.cursor;
        self.incr_cursor();

        if self.cursor != old {
            self.delete_left()
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
        self.text[self.offset..]
            .split_at(self.cursor - self.offset)
    }
}

fn width(s: &str) -> usize {
    Span::raw(s).width()
}

impl <'a> StatefulWidget for TextInput<'a> {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self
            .block
            .map_or(area, super::rendered_block(area, buf));

        let area = self
            .prompt
            .map_or(area, |prompt| draw_prompt(prompt, area, buf));

        state.adjust_offset(area);

        let (before, after) = state.draw_text();

        buf.set_style(
            Rect { height: 1, ..area },
            self.style.add_modifier(Modifier::UNDERLINED)
        );

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
    area.shrink_left(x - area.x)
        .shrink_left(1)
}

fn highlight_cursor(x: u16, y: u16, style: Style, buf: &mut Buffer) {
    let (fg, bg) = colors(style);

    buf.get_mut(x, y)
        .set_bg(bg)
        .set_fg(fg);
}

fn colors(style: Style) -> (Color, Color) {
    match (style.fg, style.bg) {
        (Some(fg), Some(bg)) => if fg.is_bright() == bg.is_bright() {
            (fg, bg.invert())
        } else {
            (fg.invert(), bg.invert())
        },
        (None, Some(bg)) => (bg, bg.invert()),
        (Some(fg), None) => (fg, fg.invert()),
        (None, None) => (Color::Black, Color::White)
    }
}
