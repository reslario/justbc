pub mod progress;
mod scroll;
mod fit;
pub mod input;
mod spinner;
mod container;
mod clear;

pub use {
    fit::*,
    clear::*,
    scroll::*,
    spinner::*,
    container::*,
    progress::ProgressBar,
    self::input::{TextInput, TextInputState}
};

use tui::{
    text,
    Frame,
    layout::Rect,
    buffer::Buffer,
    backend::Backend,
    widgets::{self, Widget}
};

macro_rules! widget_ext_fns {
    () => {
        /// Makes this widget scrollable.
        fn scrollable(self) -> Scrollable<Self> {
            Scrollable::new(self)
        }
    
        /// Wraps this widget in a container.
        fn with_container<'a>(self) -> Container<'a, Self> {
            Container::new(self)
        }

        fn clear_area(self) -> ClearArea<Self> {
            ClearArea::new(self)
        }
    };
}

pub trait WidgetExt: Sized + widgets::Widget {
    widget_ext_fns!();

    fn render_to(self, frame: &mut Frame<impl Backend>, area: Rect) {
        frame.render_widget(self, area)
    }
}

impl <W: tui::widgets::Widget> WidgetExt for W {}

pub trait StatefulWidgetExt: Sized + widgets::StatefulWidget {
    widget_ext_fns!();

    fn render_to(self, frame: &mut Frame<impl Backend>, area: Rect, state: &mut Self::State) {
        frame.render_stateful_widget(self, area, state)
    }
}

impl <W: tui::widgets::StatefulWidget> StatefulWidgetExt for W {}

fn rendered_block(area: Rect, buf: &mut Buffer)
-> impl FnMut(widgets::Block) -> Rect + '_ {
    move |block| {
        let inner = block.inner(area);
        block.render(area, buf);
        inner
    }
}

pub fn draw_paragraph<'t>(
    text: impl Into<text::Text<'t>>,
    mut cfg: impl FnMut(widgets::Paragraph<'t>) -> widgets::Paragraph<'t>,
    area: Rect,
    buf: &mut Buffer
) -> u16 {
    let mut text = text.into();

    // there seems to be no way to find out how many lines
    // a rendered paragraph takes up, so we append a line
    // with this character in order to find it again later
    const END: &str = "\u{180E}";

    text.extend(std::iter::once(END.into()));

    cfg(widgets::Paragraph::new(text))
        .render(area, buf);

    (area.y..buf.area.bottom())
        .take_while(|y| buf.get(area.x, *y).symbol != END)
        .count()
        as _
}
