use {
    builder::builder_methods,
    crate::layout::{RectExt, Margin},
    tui::{
        text::Span,
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::{Widget, StatefulWidget, Block, Borders, BorderType}
    }
};

/// Wraps any widget and allows you to add a title and borders to it.
pub struct Container<'a, W> {
    widget: W,
    title: Option<Span<'a>>,
    margin: Margin,
    borders: Borders,
    border_style: Style,
    border_type: BorderType,
    style: Style
}

impl <'a, W: Default> Default for Container<'a, W> {
    fn default() -> Self {
        Container::new(W::default())
    }
}

impl <'a, W> Container<'a, W> {
    pub fn new(widget: W) -> Container<'a, W> {
        Container {
            widget,
            title: None,
            margin: <_>::default(),
            borders: Borders::NONE,
            border_style: <_>::default(),
            border_type: BorderType::Plain,
            style: <_>::default(),
        }
    }

    builder_methods! {
        pub title: impl Into<Span<'a>> => title.into().into();
        /// Determines how much space is left between each border
        /// and the contained widget.
        pub margin: Margin;
        pub borders: Borders;
        pub border_style: Style;
        pub border_type: BorderType;
        pub style: Style
    }

    fn render_with<F>(mut self, mut f: F, area: Rect, buf: &mut Buffer)
    where F: FnMut(W, Rect, &mut Buffer) {
        let block = self
            .title
            .replace(Span::raw(""))
            .map(|title| Block::default().title(title))
            .unwrap_or_default()
            .borders(self.borders)
            .border_type(self.border_type)
            .border_style(self.border_style)
            .style(self.style);

        let inner = block
            .inner(area)
            .shrink(self.margin);

        block.render(area, buf);

        f(self.widget, inner, buf)
    }
}

impl <'a, W: Widget> Widget for Container<'a, W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_with(W::render, area, buf)
    }
}

impl <'a, W: StatefulWidget> StatefulWidget for Container<'a, W> {
    type State = W::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_with(
            |w, area, buf| w.render(area, buf, state),
            area,
            buf
        )
    }
}
