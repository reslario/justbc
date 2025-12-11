use {
    builder::builder_methods,
    std::time::{Duration, Instant},
    tui::{
        buffer::Buffer,
        layout::Rect,
        text::{Spans, StyledGrapheme},
        widgets::{Block, StatefulWidget},
    },
};

/// Text that scrolls horizontally to fit into a limited amount of space.
#[derive(Default, Clone)]
pub struct ScrollToFit<'a> {
    spans: Spans<'a>,
    interval: Duration,
    block: Option<Block<'a>>,
}

impl<'a> ScrollToFit<'a> {
    builder_methods! {
        pub spans: impl Into<Spans<'a>> => spans.into();

        /// Sets the interval between scrolling one character.
        pub interval: Duration;

        pub block: Block<'a> => block.into()
    }
}

pub struct ScrollToFitState {
    pos: usize,
    last_scroll: Instant,
}

impl Default for ScrollToFitState {
    fn default() -> Self {
        ScrollToFitState {
            pos: 0,
            last_scroll: Instant::now(),
        }
    }
}

const SPACES: usize = 3;

impl<'a> StatefulWidget for ScrollToFit<'a> {
    type State = ScrollToFitState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.block.map_or(area, super::rendered_block(area, buf));

        let text_width = self.spans.width();

        if text_width <= area.width as usize {
            // no scrolling necessary
            buf.set_spans(area.x, area.y, &self.spans, text_width as _);
            return
        }

        let since_last = state.last_scroll.elapsed();

        if since_last >= self.interval {
            if state.pos + 1 == text_width + SPACES {
                state.pos = 0
            } else {
                state.pos += 1
            }

            state.last_scroll = Instant::now();
        }

        draw_spans(self.spans, area, state.pos, buf)
    }
}

fn draw_spans(spans: Spans, area: Rect, start: usize, buf: &mut Buffer) {
    repeated_graphemes(&spans)
        .skip(start)
        .take(area.width as _)
        .enumerate()
        .for_each(|(offs, grapheme)| draw_grapheme(grapheme, area, offs, buf))
}

type Grapheme<'a> = StyledGrapheme<'a>;

fn repeated_graphemes<'a>(spans: &'a Spans<'a>) -> impl Iterator<Item = Grapheme<'a>> {
    graphemes(&spans)
        .chain(three_spaces())
        .chain(graphemes(&spans))
}

fn graphemes<'a>(Spans(spans): &'a Spans<'a>) -> impl Iterator<Item = Grapheme<'a>> {
    spans
        .iter()
        .flat_map(|span| span.styled_graphemes(<_>::default()))
}

fn three_spaces<'a>() -> impl Iterator<Item = Grapheme<'a>> {
    std::iter::repeat(Grapheme {
        symbol: " ",
        style: <_>::default(),
    })
    .take(SPACES)
}

fn draw_grapheme(grapheme: Grapheme, area: Rect, offset: usize, buf: &mut Buffer) {
    buf.get_mut(area.x + offset as u16, area.y)
        .set_symbol(grapheme.symbol)
        .set_style(grapheme.style);
}
