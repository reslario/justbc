mod list;
mod details;

use {
    builder::builder_methods,
    gen_tui::{
        widgets::WidgetExt,
        layout::{RectExt, Margin}
    },
    bandcamp_api::data::search::SearchResult,
    tui::{
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::{StatefulWidget, Borders}
    }
};

pub struct ResultView<'a> {
    results: &'a [SearchResult],
    style: Style,
    highlight_style: Style
}

impl <'a> ResultView<'a> {
    pub fn new(results: &'a [SearchResult]) -> ResultView<'a> {
        ResultView {
            results,
            style: <_>::default(),
            highlight_style: <_>::default(),
        }
    }

    builder_methods! {
        pub style: Style;
        pub highlight_style: Style
    }

    fn draw_details(&self, index: usize, area: Rect, buf: &mut Buffer) -> Rect {
        use tui::widgets::Widget;

        const MIN_HEIGHT: u16 = 10;

        let height = area.height - self.results.len() as u16;

        let ((rem, draw), borders) = if height < MIN_HEIGHT {
            (
                area.split_x(area.width - 25.min(area.width / 3)),
                Borders::LEFT
            )
        } else {
            (
                area.split_y(area.height - MIN_HEIGHT),
                Borders::TOP
            )
        };

        buf.set_style(draw, self.style);

        details::ResultDetails::new(&self.results[index])
            .style(self.style)
            .with_container()
            .borders(borders)
            .margin(Margin { left: 1, ..<_>::default() })
            .render(draw, buf);

        rem
    }
}

pub type ResultViewState = list::ResultListState;

impl <'a> StatefulWidget for ResultView<'a> {
    type State = ResultViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = state
            .selected()
            .map_or(area, |index| self.draw_details(index, area, buf));
        
        list::ResultList::new(self.results)
            .style(self.style)
            .highlight_style(self.highlight_style)
            .render(area, buf, state);
    }
}
