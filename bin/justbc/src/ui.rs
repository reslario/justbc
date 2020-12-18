use {
    std::error::Error,
    bandcamp_api::data::releases::{Release, Track, Stream},
    crate::state::{State, Core, WidgetState, Active, ExploreState},
    gen_tui::{
        layout::{RectExt, Margin},
        widgets::{WidgetExt, StatefulWidgetExt}
    },
    tui::{
        Frame,
        backend::Backend,
        layout::{Rect, Alignment},
        style::{Color, Style, Modifier},
        widgets::{Borders, Paragraph, Wrap, BorderType}
    },
    bc_tui::{
        nav::NavView,
        tracks::PlayBar,
        releases::ReleaseView
    }
};

const ACCENT: Style = Style {
    fg: Some(Color::Cyan),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty() 
};

#[derive(Copy, Clone)]
struct Layout {
    bottom: Rect,
    left: Rect,
    right: Rect
}

impl From<Rect> for Layout {
    fn from(area: Rect) -> Self {
        let (top, bottom) = area.split_y(area.height - 4);
        let (left, right) = top.split_ratio_x(0.4);

        Layout {
            bottom,
            left,
            right
        }
    }
}

pub fn draw(frame: &mut Frame<impl Backend>, state: &mut State) {
    let area = frame.size();
    let layout = Layout::from(area);

    if let Some(release) = state.core.release.as_ref() {
        draw_playing(release, &state.core, &mut state.widgets, layout, frame)
    } else {
        draw_placeholders(state, layout, frame)
    }

    draw_nav(state, layout, frame);

    if let Some(error) = state.error.as_deref() {
        draw_error(error, frame, area)
    }
}

fn draw_playing(
    release: &Release,
    core: &Core,
    widgets: &mut WidgetState,
    layout: Layout,
    frame: &mut Frame<impl Backend>
) {
    ReleaseView::new(release)
        .playing_style(ACCENT)
        .scrollable()
        .scroll_y(widgets.release_scroll)
        .with_container()
        .margin(Margin::right(1))
        .borders(Borders::RIGHT)
        .render_to(frame, layout.left, &mut widgets.release);

    if let Some(track) = core.queue.current() {
        PlayBar::new(&release.info.artist, track)
            .elapsed(core.player.elapsed())
            .volume(core.player.volume())
            .bar_style(ACCENT)
            .with_container()
            .borders(Borders::TOP)
            .render_to(frame, layout.bottom, &mut widgets.play_bar)
    }
}

fn draw_placeholders(state: &mut State, layout: Layout, frame: &mut Frame<impl Backend>) {
    Paragraph::new("No release")
        .alignment(Alignment::Center)
        .with_container()
        .margin(Margin { right: 1, top: layout.left.height / 2 - 1, ..<_>::default() })
        .borders(Borders::RIGHT)
        .render_to(frame, layout.left);

    PlayBar::new("Nothing playing", &dummy_track())
        .volume(state.core.player.volume())
        .with_container()
        .borders(Borders::TOP)
        .render_to(frame, layout.bottom, &mut state.widgets.play_bar)
}

fn draw_nav(state: &mut State, layout: Layout, frame: &mut Frame<impl Backend>) {
    let mut nav = NavView::default();
    nav = match state.navigation.active {
        Active::Library => nav.library(),
        Active::Explore => match &state.navigation.explore {
            ExploreState::Blank(_) => nav.blank(),
            ExploreState::Loading(_) => nav.loading(),
            ExploreState::Outlet(o) => nav.outlet(o),
            ExploreState::Release(r) => nav.release(r),
            ExploreState::Search(s) => nav.search(s),
            ExploreState::Fan(f) => nav.fan(f)
        }
    };

    nav.highlight_style(ACCENT)
        .with_container()
        .margin(Margin::left(1))
        .render_to(frame, layout.right, &mut state.widgets.nav);
}

fn draw_error(error: &dyn Error, frame: &mut Frame<impl Backend>, area: Rect) {
    const SCALE: u16 = 3;

    let (width, height) = (area.width / SCALE, area.height / SCALE);

    Paragraph::new(error.to_string())
        .wrap(Wrap { trim: true })
        .with_container()
        .title(" An error occured ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .margin(Margin::all(1))
        .clear_area()
        .render_to(frame, area.centered(width, height))
}

fn dummy_track() -> Track {
    Track { 
        title: String::new(),
        stream: Stream { mp3_128: "r://".parse().unwrap() },
        duration: <_>::default() 
    }
}
