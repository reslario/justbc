use {
    std::{fmt, ops},
    builder::builder_methods,
    crate::{
        layout::RectExt,
        widgets::ProgressBar
    },
    tui::{
        text::Span,
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::{Block, Widget}
    }
};

/// Determines the placement of the values of a [labeled progress bar](Labeled).
#[derive(Copy, Clone)]
pub enum Placement {
    /// To the left of the bar
    Left,
    /// To the right of the bar
    Right,
    /// The current value on the left, and the end value on the right
    Split
}

impl Default for Placement {
    fn default() -> Self {
        Placement::Left
    }
}

/// A customisable progress bar that displays the
/// current / end values.
#[derive(Default)]
pub struct Labeled<'a, T, S> {
    max: T,
    pos: T,
    symbols: S,
    placement: Placement,
    margin: u16,
    block: Option<Block<'a>>,
    style: Style,
    bar_style: Style,
}

impl <'a, T, S> Labeled<'a, T, S> 
where
    T: fmt::Display + ops::Div<T, Output = f32>,
    S: AsRef<[char]>
{
    builder_methods! {
        /// Sets the maximum (end) value of the progress bar.
        pub max: T;

        /// Sets the curent position of the progress bar.
        pub pos: T;

        // Sets the symbols this progress bar uses.
        ///
        /// See [ProgressBar::symbols](crate::widgets::ProgressBar::symbols)
        /// for more info.
        pub symbols: S;

        /// Sets the placement of the label(s).
        pub placement: Placement;

        // Sets the margin between the label(s) and the progress bar.
        pub margin: u16;

        pub block: Block<'a> => block.into();

        pub style: Style;

        /// Sets the style of the contained progress bar.
        pub bar_style: Style
    }

    fn draw_left(&self, pos: Span, max: Span, area: Rect, buf: &mut Buffer) -> Rect {
        draw_with_sep(pos, max, area, self.style, buf)
            .shrink_left(self.margin)
    }

    fn draw_right(&self, pos: Span, max: Span, area: Rect, buf: &mut Buffer) -> Rect {
        let width = pos.width()
            + SEP.chars().count()
            + max.width();
        
        let label_area = shrink_to(area, width as _);
        draw_with_sep(pos, max, label_area, self.style, buf);

        area.shrink_right(width as _)
            .shrink_right(self.margin)
    }

    fn draw_split(&self, pos: Span, max: Span, area: Rect, buf: &mut Buffer) -> Rect {
        let width = max.width() as _;
        let max_area = shrink_to(area, width);

        buf.set_span(max_area.x, max_area.y, &max, max_area.width);
        let (x, _) = buf.set_span(area.x, area.y, &pos, area.width);

        area.shrink_left(x - area.x)
            .shrink_left(self.margin)
            .shrink_right(width)
            .shrink_right(self.margin)
    }
}

fn shrink_to(area: Rect, width: u16) -> Rect {
    area.shrink_left(area.width.saturating_sub(width))
}

impl <'a, T, S> Widget for Labeled<'a, T, S>
where
    T: fmt::Display + ops::Div<T, Output = f32>,
    S: AsRef<[char]> + Default
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max = to_span(&self.max, self.style);
        let pos = to_span(&self.pos, self.style);

        let inner = self
            .block
            .as_ref()
            .map_or(area, |block| block.inner(area));

        let inner = match self.placement {
            Placement::Left => self.draw_left(pos, max, inner, buf),
            Placement::Right => self.draw_right(pos, max, inner, buf),
            Placement::Split => self.draw_split(pos, max, inner, buf)
        };

        self.block.map(|block| block.render(area, buf));

        ProgressBar::default()
            .style(self.bar_style)
            .symbols(self.symbols)
            .progress(self.pos / self.max)
            .render(inner, buf)
    }
}

fn to_span(val: impl fmt::Display, style: Style) -> Span<'static> {
    Span::styled(val.to_string(), style)
}

const SEP: &str = " / ";

fn draw_with_sep(pos: Span, max: Span, area: Rect, style: Style, buf: &mut Buffer) -> Rect {
    let slash = Span::styled(SEP, style);

    let (x, y) = buf.set_span(area.x, area.y, &pos, area.width);
    let (x, y) = buf.set_span(x, y, &slash, area.width);
    let (x, y) = buf.set_span(x, y, &max, area.width);

    area.shrink_left(x - area.x)
        .shrink_top(y - area.y)
}
