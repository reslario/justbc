use {
    crate::tracks::Time,
    std::time::Duration,
    builder::builder_methods,
    bandcamp_api::data::releases::Track,
    gen_tui::{
        style::StyleExt,
        layout::RectExt,
        widgets::{
            progress,
            ProgressBar,
            ScrollToFit,
            ScrollToFitState
        }
    },
    tui::{
        text::Span,
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::StatefulWidget
    }
};

pub struct PlayBar<'a> {
    artist: &'a str,
    track: &'a Track,
    elapsed: Time,
    style: Style,
    bar_style: Style
}

impl <'a> PlayBar<'a> {
    pub fn new(artist: &'a str, track: &'a Track) -> PlayBar<'a> {
        PlayBar {
            artist,
            track,
            elapsed: <_>::default(),
            style: <_>::default(),
            bar_style: <_>::default()
        }
    }

    builder_methods! {
        pub elapsed: impl Into<Time> => elapsed.into();
        pub bar_style: Style;
        pub style: Style
    }

    fn draw_track_info(&self, area: Rect, buf: &mut Buffer, state: &mut PlayBarState) -> Rect {
        let width = area.width / 5;

        let draw = Rect { width, ..area };

        let text = ScrollToFit::default()
            .interval(Duration::from_millis(500));

        let style = self.style.bold();

        text.clone()
            .spans(Span::styled(self.artist, style))
            .render(draw, buf, &mut state.artist);

        text.spans(Span::styled(&self.track.title, style))
            .render(draw.shrink_top(2), buf, &mut state.title);

        area.shrink_left(width)
    }

    fn draw_bar(&self, area: Rect, buf: &mut Buffer) {
        use tui::widgets::Widget;

        let margin = 5.min(area.width / 20);

        let area = area
            .shrink_top(1)
            .shrink_left(margin)
            .shrink_right(margin);
        
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

#[derive(Default)]
pub struct PlayBarState {
    artist: ScrollToFitState,
    title: ScrollToFitState
}

impl <'a> StatefulWidget for PlayBar<'a> {
    type State = PlayBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.draw_track_info(area, buf, state);
        self.draw_bar(area, buf);
    }
}
