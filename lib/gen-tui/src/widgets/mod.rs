pub mod progress;
mod scroll;

pub use {
    scroll::*,
    progress::ProgressBar,
};

fn rendered_block(area: tui::layout::Rect, buf: &mut tui::buffer::Buffer)
-> impl FnMut(tui::widgets::Block) -> tui::layout::Rect + '_ {
    move |block| {
        let inner = block.inner(area);
        tui::widgets::Widget::render(block, area, buf);
        inner
    }
}
