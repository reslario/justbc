use {
    crate::tracks::Time,
    bandcamp_api::data::releases::Track,
    builder::builder_methods,
    gen_tui::{
        layout::RectExt,
        style::StyleExt,
        widgets::{progress, ProgressBar, ScrollToFit, ScrollToFitState},
    },
    std::time::Duration,
    tui::{buffer::Buffer, layout::Rect, style::Style, text::Span, widgets::StatefulWidget},
};

pub struct PlayBar<'a> {
    artist: &'a str,
    track: &'a Track,
    elapsed: Time,
    volume: f32,
    style: Style,
    bar_style: Style,
}

impl<'a> PlayBar<'a> {
    pub fn new(artist: &'a str, track: &'a Track) -> PlayBar<'a> {
        PlayBar {
            artist,
            track,
            elapsed: <_>::default(),
            volume: 1.,
            style: <_>::default(),
            bar_style: <_>::default(),
        }
    }

    builder_methods! {
        pub elapsed: impl Into<Time> => elapsed.into();
        pub bar_style: Style;
        pub volume: f32;
        pub style: Style
    }

    fn draw_track_info(&self, area: Rect, buf: &mut Buffer, state: &mut PlayBarState) -> Rect {
        let width = area.width / 5;

        let draw = Rect { width, ..area };

        let text = ScrollToFit::default().interval(Duration::from_millis(500));

        let style = self.style.bold();

        text.clone()
            .spans(Span::styled(self.artist, style))
            .render(draw, buf, &mut state.artist);

        text.spans(Span::styled(&self.track.title, style)).render(
            draw.shrink_top(2),
            buf,
            &mut state.title,
        );

        area.shrink_left(width)
    }

    fn draw_volume(&self, area: Rect, buf: &mut Buffer) -> Rect {
        const WIDTH: u16 = 7;

        let text = format!(
            "{} {:>3}%",
            speaker(self.volume),
            f32::round(self.volume * 100.)
        );

        let Rect { x, y, .. } = area.scale_from_right(WIDTH).shrink_top(1);

        buf.set_span(x, y, &Span::styled(text, self.style), WIDTH);

        area.shrink_right(WIDTH)
    }

    fn draw_bar(&self, area: Rect, buf: &mut Buffer) {
        use tui::widgets::Widget;

        let margin = 5.min(area.width / 20);

        let area = area.shrink_top(1).shrink_left(margin).shrink_right(margin);

        ProgressBar::labeled()
            .max(self.track.duration.into())
            .pos(self.elapsed)
            .symbols(['╴', '─'])
            .margin(margin)
            .placement(progress::Placement::Split)
            .bar_style(self.bar_style)
            .render(area, buf)
    }
}

fn speaker(volume: f32) -> char {
    if volume > 0.49 {
        '🔊'
    } else if volume < 0.01 {
        '🔈'
    } else {
        '🔉'
    }
}

#[derive(Default)]
pub struct PlayBarState {
    artist: ScrollToFitState,
    title: ScrollToFitState,
}

impl<'a> StatefulWidget for PlayBar<'a> {
    type State = PlayBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.draw_track_info(area, buf, state);
        let area = self.draw_volume(area, buf);
        self.draw_bar(area, buf);
    }
}
