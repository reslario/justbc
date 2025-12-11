use {
    crate::symbols,
    bandcamp_api::data::{outlets::*, releases::ReleaseKind},
    builder::builder_methods,
    gen_tui::{layout::RectExt, style::StyleExt, widgets},
    std::borrow::Cow,
    tui::{
        buffer::Buffer,
        layout::Rect,
        style::Style,
        text::Span,
        widgets::{List, ListItem, ListState, StatefulWidget, Wrap},
    },
};

pub struct OutletView<'a> {
    outlet: &'a Outlet,
    style: Style,
    highlight_style: Style,
}

impl<'a> OutletView<'a> {
    pub fn new(outlet: &'a Outlet) -> OutletView<'a> {
        OutletView {
            outlet,
            style: <_>::default(),
            highlight_style: <_>::default(),
        }
    }

    builder_methods! {
        pub style: Style;
        pub highlight_style: Style
    }

    fn draw_info(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let info = &self.outlet.info;

        let symbol = match info.kind {
            OutletKind::Label => symbols::LABEL,
            OutletKind::Artist => symbols::ARTIST,
        };

        let bold = |s| Span::styled(s, Style::default().bold()).into();

        let title = format!("{} {}", symbol, info.name);

        let mut text = vec![bold(title.as_str())];

        if let Some(bio) = info.bio.as_ref() {
            text.push(" ".into());
            text.push(bold("About"));

            text.extend(bio.split('\n').filter(|s| !s.is_empty()).map(<_>::into));
        }

        let height = widgets::draw_paragraph(
            text,
            |p| p.style(self.style).wrap(Wrap { trim: true }),
            area,
            buf,
        );

        area.shrink_top(height).shrink_top(1)
    }

    fn draw_releases(&self, area: Rect, buf: &mut Buffer, state: &mut OutletViewState) {
        let releases = &self.outlet.discography;

        if releases.is_empty() {
            return
        }

        buf.set_span(
            area.x,
            area.y,
            &Span::styled("Releases", self.style.bold()),
            area.width,
        );

        let draw = area.shrink_top(1).shrink_left(2);

        let items = releases
            .iter()
            .map(fmt_release)
            .map(Span::raw)
            .map(ListItem::new)
            .collect::<Vec<_>>();

        List::new(items)
            .style(self.style)
            .highlight_style(self.highlight_style)
            .render(draw, buf, state)
    }
}

fn fmt_release(release: &Release) -> String {
    let icon = match release.kind {
        ReleaseKind::Track => symbols::TRACK,
        ReleaseKind::Album => symbols::ALBUM,
    };

    format!("{} {}", icon, fmt_release_info(release))
}

fn fmt_release_info(info: &Release) -> Cow<'_, str> {
    info.artist
        .as_ref()
        .map(|artist| crate::fmt_release(artist, &info.title))
        .map(<_>::into)
        .unwrap_or_else(|| info.title.as_str().into())
}

pub type OutletViewState = ListState;

impl<'a> StatefulWidget for OutletView<'a> {
    type State = OutletViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.draw_info(area, buf);
        self.draw_releases(area, buf, state)
    }
}
