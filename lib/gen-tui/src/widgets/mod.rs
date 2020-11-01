pub mod progress;
mod scroll;
mod fit;

pub use {
    fit::*,
    scroll::*,
    progress::ProgressBar,
};

fn rendered_block(area: tui::layout::Rect, buf: &mut tui::buffer::Buffer)
pub trait WidgetExt: Sized {
    fn scrollable(self) -> Scrollable<Self> {
        Scrollable::new(self)
    }
}

impl <W: tui::widgets::Widget> WidgetExt for W {}

pub trait StatefulWidgetExt: Sized {
    fn scrollable(self) -> Scrollable<Self> {
        Scrollable::new(self)
    }
}

impl <W: tui::widgets::StatefulWidget> StatefulWidgetExt for W {}

-> impl FnMut(tui::widgets::Block) -> tui::layout::Rect + '_ {
    move |block| {
        let inner = block.inner(area);
        tui::widgets::Widget::render(block, area, buf);
        inner
    }
}
