use {
    crate::symbols,
    std::borrow::Cow,
    builder::builder_methods,
    gen_tui::{
        style::StyleExt,
        layout::RectExt
    },
    bandcamp_api::data::{
        outlets::*,
        releases::ReleaseKind
    },
    tui::{
        text::Span,
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::{StatefulWidget, List, ListItem, ListState, Paragraph, Wrap}
    }
};

pub struct OutletView<'a> {
    outlet: &'a Outlet,
    style: Style,
    highlight_style: Style
}

impl <'a> OutletView<'a> {
    pub fn new(outlet: &'a Outlet) -> OutletView<'a> {
        OutletView {
            outlet,
            style: <_>::default(),
            highlight_style: <_>::default()
        }
    }

    builder_methods! {
        pub style: Style;
        pub highlight_style: Style
    }

    fn draw_info(&self, area: Rect, buf: &mut Buffer) -> Rect {
        use tui::widgets::Widget;

        let info = &self.outlet.info;

        let symbol = match info.kind {
            OutletKind::Label => symbols::LABEL,
            OutletKind::Artist => symbols::ARTIST
        };

        let bold = |s| Span::styled(s, Style::default().bold()).into();

        let title = format!("{} {}", symbol, info.name);

        let mut text = vec![
            bold(title.as_str()),
        ];

        if let Some(bio) = info.bio.as_ref() {
            text.push(" ".into());
            text.push(bold("About"));

            text.extend(bio
                .split('\n')
                .filter(|s| !s.is_empty())
                .map(<_>::into)
            );
        }

        // there seems to be no way to find out how many lines
        // a rendered paragraph takes up, so we append a line
        // with this character in order to find it again later
        const END: &str = "\u{180E}";

        text.push(END.into());

        Paragraph::new(text)
            .style(self.style)
            .wrap(Wrap { trim: true })
            .render(area, buf);

        let lines = (area.y..buf.area.bottom())
            .take_while(|y| buf.get(area.x, *y).symbol != END)
            .count();

        area.shrink_top(lines as _)
            .shrink_top(1)
    }

    fn draw_featured(&self, area: Rect, buf: &mut Buffer, state: &mut OutletViewState) -> Rect {
        let featured = &self.outlet.featured;

        if featured.is_empty() { return area }

        self.draw_titled_list(
            "Featured",
            featured,
            fmt_featured,
            area,
            buf,
            state
        )
    }

    fn draw_releases(&self, area: Rect, buf: &mut Buffer, state: &mut OutletViewState) {
        let releases = &self.outlet.releases;

        if releases.is_empty() { return }

        self.draw_titled_list(
            "Releases",
            releases,
            fmt_release,
            area,
            buf,
            state
        );
    }

    fn draw_titled_list<'r, T: 'r>(
        &self,
        title: &str,
        iter: impl IntoIterator<Item = &'r T>,
        format: impl Fn(&'r T) -> Cow<'r, str>,
        area: Rect,
        buf: &mut Buffer,
        state: &mut OutletViewState
    ) -> Rect {
        buf.set_span(area.x, area.y, &Span::styled(title, self.style.bold()), area.width);

        let draw = area
            .shrink_top(1)
            .shrink_left(2);

        let items = iter
            .into_iter()
            .map(format)
            .map(Span::raw)
            .map(ListItem::new)
            .collect::<Vec<_>>();

        let len = items.len();

        List::new(items)
            .style(self.style)
            .highlight_style(self.highlight_style)
            .render(draw, buf, &mut state.list);

        area.shrink_top(len as _)
            .shrink_top(2)
    }
}

fn fmt_featured(featured: &Featured) -> Cow<str> {
    fmt_release_info(&featured)
}

fn fmt_release(release: &Release) -> Cow<str> {
    let icon = match release.kind {
        ReleaseKind::Track => symbols::TRACK,
        ReleaseKind::Album => symbols::ALBUM
    };

    format!("{} {}", icon, fmt_release_info(&release.info)).into()
}

fn fmt_release_info(info: &ReleaseInfo) -> Cow<str> {
    info.artist
        .as_ref()
        .map(|artist| crate::fmt_release(artist, &info.title))
        .map(<_>::into)
        .unwrap_or_else(|| info.title.as_str().into())
}

#[derive(Default)]
pub struct OutletViewState {
    list: ListState
}

impl std::ops::Deref for OutletViewState {
    type Target = ListState;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl <'a> StatefulWidget for OutletView<'a> {
    type State = OutletViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.draw_info(area, buf);
        let area = self.draw_featured(area, buf, state);
        self.draw_releases(area, buf, state)
    }
}
