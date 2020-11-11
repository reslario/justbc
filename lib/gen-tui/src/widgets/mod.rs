pub mod progress;
mod scroll;
mod fit;
pub mod input;
mod spinner;
mod container;

pub use {
    fit::*,
    scroll::*,
    spinner::*,
    container::*,
    self::input::{TextInput, TextInputState},
    progress::ProgressBar
};

use tui::{
    widgets,
    layout::Rect,
    buffer::Buffer
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
    };
}

pub trait WidgetExt: Sized {
    widget_ext_fns!();
}

impl <W: tui::widgets::Widget> WidgetExt for W {}

pub trait StatefulWidgetExt: Sized {
    widget_ext_fns!();
}

impl <W: tui::widgets::StatefulWidget> StatefulWidgetExt for W {}

fn rendered_block(area: Rect, buf: &mut Buffer)
-> impl FnMut(widgets::Block) -> Rect + '_ {
    move |block| {
        let inner = block.inner(area);
        widgets::Widget::render(block, area, buf);
        inner
    }
}
