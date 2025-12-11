use {
    crate::tracks,
    bandcamp_api::data::releases::{Release, Track},
    builder::builder_methods,
    gen_tui::{layout::RectExt, style::StyleExt},
    std::time::Duration,
    tui::{
        buffer::Buffer,
        layout::Rect,
        style::Style,
        text::{Span, Spans, Text},
        widgets::{List, ListItem, ListState, Paragraph, StatefulWidget, Wrap},
    },
};

pub struct ReleaseView<'a> {
    release: &'a Release,
    style: Style,
    playing_style: Style,
}

impl<'a> ReleaseView<'a> {
    pub fn new(release: &'a Release) -> Self {
        Self {
            release,
            style: <_>::default(),
            playing_style: <_>::default(),
        }
    }

    builder_methods! {
        pub style: Style;
        pub playing_style: Style
    }

    fn date_duration(&self) -> String {
        use std::ops::Div;

        let date = self.release.info.release_date.fmt_long();

        if self.release.tracks.len() == 1 {
            date.to_string()
        } else {
            let minutes = self
                .release
                .tracks
                .iter()
                .map(|track| track.duration)
                .sum::<Duration>()
                .as_secs_f32()
                .div(60.)
                .round() as u16;

            format!(
                "{} {} {} tracks, {} minute{}",
                date,
                tui::symbols::DOT,
                self.release.tracks.len(),
                minutes,
                if minutes > 1 { 's' } else { '​' }
            )
        }
    }

    fn draw_heading(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let style = self.style.bold();

        let title = Span::styled(&self.release.info.title, style);
        let (_, y) = buf.set_span(area.x, area.y, &title, area.width);

        let artist = Span::styled(&self.release.info.artist, style);
        let (_, y) = buf.set_span(area.x, y + 1, &artist, area.width);

        let date_duration = Span::styled(self.date_duration(), self.style);
        let (_, y) = buf.set_span(area.x, y + 1, &date_duration, area.width);

        area.shrink_top(y - area.y + 1)
    }

    fn draw_track_list(&self, area: Rect, buf: &mut Buffer, state: &mut ReleaseViewState) -> Rect {
        let mut track_list = self
            .release
            .tracks
            .iter()
            .map(|track| track_text(track, area.width - 2, self.style))
            .map(ListItem::new)
            .collect::<Vec<_>>();

        state.highlight_playing(&mut track_list, self.playing_style);

        let height = track_list.len();

        List::new(track_list)
            .style(self.style)
            .highlight_symbol(state.highlight_symbol())
            .render(area, buf, &mut state.track_list);

        area.shrink_top(height as _)
    }

    fn draw_rest(&self, area: Rect, buf: &mut Buffer) {
        use tui::widgets::Widget;

        let lines = self
            .titled_section(Self::about, "About")
            .chain(self.titled_section(Self::credits, "Credits"))
            .collect();

        Paragraph::new(Text { lines })
            .style(self.style)
            .wrap(Wrap { trim: true })
            .render(area, buf)
    }

    fn titled_section<'s, F>(&'s self, f: F, title: &'s str) -> impl Iterator<Item = Spans<'s>>
    where
        F: Fn(&'s Self) -> Option<&'s str>,
    {
        f(self)
            .into_iter()
            .flat_map(move |section| titled(section.split('\n'), title))
    }

    fn about(&self) -> Option<&str> {
        self.release.info.about.as_deref()
    }

    fn credits(&self) -> Option<&str> {
        self.release.info.credits.as_deref()
    }
}

fn track_text(track: &Track, width: u16, style: Style) -> Spans {
    const SPACE: &str = "   ";
    const SPACES: usize = SPACE.len();

    let time = tracks::Time::from(track.duration).to_string();
    let time = Span::styled(time, style);

    let rem = usize::from(width)
        .saturating_sub(time.width())
        .saturating_sub(SPACES);

    let mut title = Span::styled(&track.title, style);
    let title_width = title.width();

    let space = if title_width > rem {
        trim_title(title.content.to_mut(), title_width - rem);

        Span::styled(SPACE, style)
    } else {
        Span::styled(" ".repeat(rem - title_width + SPACES), style)
    };

    Spans(vec![title, space, time])
}

fn trim_title(title: &mut String, amount: usize) {
    for _ in 0..amount + 1 {
        title.pop();
    }

    title.push('…')
}

fn titled<'a>(
    lines: impl Iterator<Item = &'a str>,
    title: &'a str,
) -> impl Iterator<Item = Spans<'a>> {
    let bold = Style::default().bold();

    with_newline(Span::styled(title, bold).into())
        .chain(lines.map(<_>::into))
        .chain(std::iter::once(newline()))
}

fn with_newline(spans: Spans) -> impl Iterator<Item = Spans> {
    use std::iter;

    iter::once(spans).chain(iter::once(newline()))
}

fn newline() -> Spans<'static> {
    Span::styled("", <_>::default()).into()
}

#[derive(Default, Debug, Clone)]
pub struct ReleaseViewState {
    track_list: ListState,
    playing: Option<usize>,
}

impl std::ops::Deref for ReleaseViewState {
    type Target = ListState;

    fn deref(&self) -> &Self::Target {
        &self.track_list
    }
}

impl std::ops::DerefMut for ReleaseViewState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.track_list
    }
}

impl ReleaseViewState {
    pub fn play(&mut self, track: impl Into<Option<usize>>) {
        self.playing = track.into();
    }

    pub fn playing(&self) -> Option<usize> {
        self.playing
    }

    pub fn selection_down(&mut self) {
        let new = self.selected().map(|sel| sel + 1).unwrap_or_default();

        self.select(new.into())
    }

    pub fn selection_up(&mut self) {
        let new = self.selected().map(|sel| sel - 1);

        self.select(new)
    }

    fn highlight_symbol(&self) -> &'static str {
        match (self.track_list.selected(), self.playing) {
            // all hail the phoenician number two
            (Some(s), Some(p)) if s == p => "𐤚⠀",
            (Some(_), _) => "▶ ",
            _ => "  ",
        }
    }

    fn highlight_playing(&self, tracks: &mut Vec<ListItem>, style: Style) {
        use std::mem;

        if let Some(index) = self.playing {
            let mut playing = ListItem::new(Text { lines: vec![] });

            mem::swap(&mut tracks[index], &mut playing);
            mem::swap(&mut tracks[index], &mut playing.style(style))
        }
    }
}

impl<'a> StatefulWidget for ReleaseView<'a> {
    type State = ReleaseViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.draw_heading(area, buf);
        let area = self.draw_track_list(area.shrink_top(1), buf, state);
        self.draw_rest(area.shrink_top(1), buf);
    }
}

#[cfg(test)]
mod test {
    use {super::*, bandcamp_api::data::releases::Stream, std::time::Duration};

    #[test]
    fn track_text() {
        let mut track = Track {
            title: "short".into(),
            stream: Stream {
                mp3_128: "a://b.c".parse().unwrap(),
            },
            duration: Duration::from_secs(66),
        };

        assert_eq!("short                       1:06", formatted(&track));

        track.title = "way too long to fit into 30 columns".into();

        assert_eq!("way too long to fit into…   1:06", formatted(&track));

        track.title = "just the right length! :)".into();

        assert_eq!("just the right length! :)   1:06", formatted(&track));

        track.title = "just one char too long! :(".into();

        assert_eq!("just one char too long! …   1:06", formatted(&track));
    }

    fn formatted(track: &Track) -> String {
        super::track_text(&track, 32, <_>::default())
            .0
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }
}
