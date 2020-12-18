use {
    crate::symbols,
    builder::builder_methods,
    gen_tui::{
        widgets,
        layout::RectExt,
        style::StyleExt
    },
    tui::{
        style::Style,
        layout::Rect,
        buffer::Buffer,
        text::Span,
        widgets::{StatefulWidget, List, ListState, ListItem, Wrap},
    },
    bandcamp_api::data::{
        releases::ReleaseKind,
        fans::{Fan, Collected}
    }
};

pub struct FanView<'a> {
    fan: &'a Fan,
    style: Style,
    highlight_style: Style
}

impl <'a> FanView<'a> {
    pub fn new(fan: &'a Fan) -> FanView<'a> {
        FanView {
            fan,
            style: <_>::default(),
            highlight_style: <_>::default()
        }
    }

    builder_methods! {
        pub style: Style;
        pub highlight_style: Style
    }
}

#[derive(Default)]
pub struct FanViewState {
    pub collection: ListState,
    loading: bool
}

impl FanViewState {
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading
    }
}

impl <'a> StatefulWidget for FanView<'a> {
    type State = FanViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = span_with_icon(symbols::FAN, Span::styled(&self.fan.name, self.style.bold()), area, buf);
        let area = span_with_icon('📍', Span::styled(&self.fan.location, self.style), area, buf)
            .shrink_top(1);

        let bold = |s| Span::styled(s, Style::default().bold());

        let mut text = vec![bold("About").into()];
            
        text.extend(self.fan.bio
            .split('\n')
            .map(<_>::into)
        );

        let height = widgets::draw_paragraph(
            text,
            |p| p.style(self.style).wrap(Wrap { trim: true }),
            area,
            buf
        );

        let area = area.shrink_top(height + 1);

        buf.set_span(area.x, area.y, &bold("Collection"), area.width);

        let last = if state.loading {
            "Loading more..."
        } else {
            "Load more..."
        }.into();

        let items = self.fan
            .collection
            .iter()
            .map(fmt_collected)
            .chain(std::iter::once(last))
            .map(ListItem::new)
            .collect::<Vec<_>>();

        List::new(items)
            .style(self.style)
            .highlight_style(self.highlight_style)
            .render(area.shrink_top(2), buf, &mut state.collection)
    }
}

fn span_with_icon(icon: char, span: Span, area: Rect, buf: &mut Buffer) -> Rect {
    buf.get_mut(area.x, area.y).set_char(icon);
    buf.set_span(area.x + 3, area.y, &span, area.width);
    area.shrink_top(1)
}

fn fmt_collected(collected: &Collected) -> String {
    format!(
        "{} {}",
        icon(collected),
        crate::release_fmt(&collected.artist, &collected.title)
    )
}

fn icon(collected: &Collected) -> char {
    match collected.kind {
        ReleaseKind::Album => symbols::ALBUM,
        ReleaseKind::Track => symbols::TRACK
    }
}
