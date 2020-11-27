use tui::{
    layout::Rect,
    buffer::Buffer,
    widgets::{Widget, StatefulWidget, Clear}
};

pub struct ClearArea<W> {
    widget: W
}

impl <W> ClearArea<W> {
    pub fn new(widget: W) -> ClearArea<W> {
        ClearArea { widget }
    }
}

impl <W: Widget> Widget for ClearArea<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        self.widget.render(area, buf)
    }
}

impl <W: StatefulWidget> StatefulWidget for ClearArea<W> {
    type State = W::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);
        self.widget.render(area, buf, state)
    }
}
