use {
    crate::{
        layout::RectExt,
        buffer::BufferExt,
    },
    tui::{
        layout::Rect,
        buffer::Buffer,
        widgets::{Widget, StatefulWidget}
    }
};

/// Wraps a widget and makes it scrollable by rendering it to an
/// intermediate buffer first.
pub struct Scrollable<W> {
    widget: W,
    x: u16,
    y: u16
}

impl <W> Scrollable<W> {
    pub fn new(widget: W) -> Self {
        Scrollable {
            widget,
            x: 0,
            y: 0
        }
    }

    pub fn scroll(self, x: u16, y: u16) -> Self {
        self.scroll_x(x)
            .scroll_y(y)
    }

    pub fn scroll_x(self, x: u16) -> Self {
        Self { x, ..self }
    }

    pub fn scroll_y(self, y: u16) -> Self {
        Self { y, ..self }
    }

    fn render_with<F>(self, mut f: F, area: Rect, buf: &mut Buffer)
    where F: FnMut(W, Rect, &mut Buffer) {
        if self.x + self.y == 0 {
            return f(self.widget, area, buf)
        }

        let expanded = area
            .grow_right(self.x)
            .grow_bottom(self.y);
        
        let mut intermediate = Buffer::empty(expanded);

        f(self.widget, expanded, &mut intermediate);

        let view = expanded
            .shrink_left(self.x)
            .shrink_top(self.y);

        buf.copy_from(area, intermediate, view)
    }
}

impl <W: Widget> Widget for Scrollable<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_with(W::render, area, buf)
    }
}

impl <W: StatefulWidget> StatefulWidget for Scrollable<W> {
    type State = W::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_with(
            |w, area, buf| w.render(area, buf, state),
            area,
            buf
        )
    }
}
