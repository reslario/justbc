mod explore;

pub use explore::ExploreState;

use {
    play::Player,
    fetch::Fetcher,
    explore::Explore,
    std::error::Error,
    crate::play::Queue,
    input::binds::Bindings,
    gen_tui::widgets::input::Message as InputMessage,
    bandcamp_api::{
        pages::SearchArgs,
        data::{
            search::Search,
            outlets::Outlet,
            releases::{Release, Track}
        }
    },
    bc_tui::{
        nav::NavViewState,
        releases::ReleaseViewState,
        tracks::PlayBarState
    }
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Focus {
    Release,
    Search,
    NavBody
}

impl Default for Focus {
    fn default() -> Self {
        Focus::NavBody
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Active {
    Library,
    Explore
}

impl Active {
    fn next(self) -> Active {
        if self == Active::Library {
            Active::Explore
        } else {
            Active::Library
        }
    }
}

impl Default for Active {
    fn default() -> Self {
        Active::Explore
    }
}

#[derive(Default)]
pub struct Navigation {
    pub library: (),
    pub explore: ExploreState,
    pub active: Active
}

#[derive(Default)]
pub struct WidgetState {
    pub nav: NavViewState,
    pub play_bar: PlayBarState,
    pub release: ReleaseViewState,
    pub release_scroll: u16
}

type Stream = stream::AudioStream<reqwest::blocking::Response>;
type Audio = rodio::Decoder<Stream>;

pub struct Core {
    bindings: Bindings,
    fetcher: Fetcher,
    focus: Focus,
    pub queue: Queue,
    next: Option<Audio>,
    pub player: Player<Audio>,
    pub release: Option<Release>,
}

impl Core {
    fn set_release(&mut self, release: Release, start_track: usize) {
        self.player.stop();
        self.next = None;
        self.queue.clone_tracks(&release.tracks);
        self.queue.set_track(start_track);
        if let Some(track) = self.queue.current() {
            self.fetch_track(track)
        }
        self.release.replace(release);
    }

    fn play(&mut self, track: usize) {
        self.player.stop();
        self.queue.set_track(track);
        if let Some(track) = self.queue.current() {
            self.fetch_track(track)
        }
    }

    fn fetch_track(&self, track: &Track) {
        self.fetcher.fetch_track(track.file.mp3_128.clone())
    }
}

pub struct State {
    pub core: Core,
    pub navigation: Navigation,
    pub widgets: WidgetState,
    pub error: Option<Box<dyn Error>>
}

impl State {
    pub fn new(bindings: Bindings, fetcher: Fetcher) -> State {
        State {
            core: Core {
                bindings,
                fetcher,
                focus: <_>::default(),
                queue: <_>::default(),
                next: None,
                player: Player::new(),
                release: None,
            },
            navigation: <_>::default(),
            widgets: <_>::default(),
            error: None
        }
    }

    pub fn input(&mut self, key: input::Key) {
        self.error = None;

        if self.core.focus == Focus::Search {
            self.handle_typing(key)
        } else if let Some(cmd) = self.core.bindings.get(key) {
            self.handle_command(cmd)
        }
    }

    fn handle_command(&mut self, command: input::Command) {
        use input::Command::*;

        match command {
            FocusRelease => self.focus(Focus::Release),
            FocusNav => self.focus(Focus::NavBody),
            FocusSearchBar => self.focus(Focus::Search),
            TabExplore => {
                self.navigation.active = Active::Explore;
                self.focus(Focus::NavBody)
            },
            CycleTabs => self.navigation.active = self.navigation.active.next(),
            SelectionDown => self.selection_down(),
            SelectionUp => self.selection_up(),
            ScrollDown => self.scroll_down(),
            ScrollUp => self.scroll_up(),
            Confirm => self.confirm(),
            VolumeUp => self.update_volume(0.05),
            VolumeDown => self.update_volume(-0.05),
            _ => {}
        }
    }

    fn sync_focus(focus: Focus, core: &mut Core, widgets: &mut WidgetState) {
        if focus == Focus::Search {
            widgets.nav.input.focus()
        } else {
            widgets.nav.input.unfocus()
        }

        core.focus = focus
    }

    fn focus(&mut self, focus: Focus) {
        Self::sync_focus(focus, &mut self.core, &mut self.widgets)
    }

    fn selection_down(&mut self) {
        match self.core.focus {
            Focus::NavBody => match self.navigation.active {
                Active::Explore => self.navigation.explore
                    .selection_down(&mut self.widgets),
                Active::Library => {}
            },
            Focus::Release => if let Some(rel) = self.core.release.as_ref() {
                if can_select_down(self.widgets.release.selected(), rel.tracks.len()) {
                    self.widgets.release.selection_down()
                }
            },
            _ => {}
        }
        
    }

    fn selection_up(&mut self) {
        match self.core.focus {
            Focus::NavBody => match self.navigation.active {
                Active::Explore => self.navigation.explore
                    .selection_up(&mut self.widgets),
                Active::Library => {}
            },
            Focus::Release => if self.core.release.is_some() 
                && can_select_up(self.widgets.release.selected()) {
                    self.widgets.release.selection_up()
            },
            _ => {}
        }
        
    }

    fn scroll_down(&mut self) {
        match self.core.focus {
            Focus::NavBody => match self.navigation.active {
                Active::Explore => self.widgets.nav.scroll_down(),
                Active::Library => {}
            },
            Focus::Release => self.widgets.release_scroll = 
                self.widgets.release_scroll.saturating_add(1),
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
        match self.core.focus {
            Focus::NavBody => match self.navigation.active {
                Active::Explore => self.widgets.nav.scroll_up(),
                Active::Library => {}
            },
            Focus::Release => self.widgets.release_scroll = 
                self.widgets.release_scroll.saturating_sub(1),
            _ => {}
        }
    }

    fn confirm(&mut self) {
        match self.core.focus {
            Focus::NavBody => match self.navigation.active {
                Active::Explore => if let Some(xp) = self.navigation.explore
                    .confirm(&mut self.core, &mut self.widgets) {
                        self.navigation.explore = xp;
                    },
                Active::Library => {}
            },
            Focus::Release => if let Some(track) = self.widgets.release.selected() {
                if self.widgets.release.playing() == Some(track) {
                    if self.core.player.is_paused() {
                        self.core.player.resume()
                    } else {
                        self.core.player.pause()
                    }
                } else {
                    self.core.play(track)
                }
            },
            _ => {}
        }
    }

    fn handle_typing(&mut self, key: input::Key) {
        if self.core.focus != Focus::Search { return }

        let input = &mut self.widgets.nav.input;

        if let Some(msg) = input.handle_key(key) {
            match msg {
                InputMessage::Cancel => self.focus(Focus::NavBody),
                InputMessage::Confirm => match self.navigation.active {
                    Active::Explore => {
                        self.core.fetcher.query::<Search, _, _>(&SearchArgs::query(input.text()));
                        self.navigation.explore = ExploreState::loading();
                        self.focus(Focus::NavBody)
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn set_response(&mut self, response: fetch::Response) {
        const FIRST: Option<usize> = Some(0);

        match response {
            fetch::Response::Search(s) => if self.try_set_explore(s, ExploreState::Search) {
                self.widgets.nav.results().select(FIRST)
            },
            fetch::Response::Outlet(o) => if self.try_set_explore(o, ExploreState::Outlet) {
                self.widgets.nav.outlet().select(FIRST)
            },
            fetch::Response::Release(r) => if self.try_set_explore(r, ExploreState::Release) {
                self.widgets.nav.release().select(FIRST)
            },
            fetch::Response::Track(resp) => {
                self.try_do(|this| Ok(this
                    .core
                    .next
                    .replace(Audio::new_mp3(Stream::new(resp?)?)?)
                ));
            }
        }
    }
    
    fn try_set_explore<T, E, F>(&mut self, result: Result<T, E>, map: F) -> bool
    where 
        E: Error + 'static,
        F: FnOnce(T) -> ExploreState
    {
        match result {
            Ok(val) => {
                self.navigation.explore = map(val);
                true
            },
            Err(e) => {
                self.error.replace(e.into());
                self.navigation.explore = ExploreState::blank();
                false
            }
        }
    }

    fn try_do<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, Box<dyn Error>>) -> Option<T> {
        match f(self) {
            Ok(val) => val.into(),
            Err(e) => {
                self.error.replace(e);
                None
            }
        }
    }

    fn update_volume(&mut self, by: f32) {
        let new = (self.core.player.volume() + by)
            .min(1.)
            .max(0.);

        self.core.player.set_volume(new)
    }

    fn try_play(&mut self, audio: Audio) {
        self.try_do(|this| this
            .core
            .player
            .play(audio)
            .map_err(<_>::into)
        );
    }

    fn finished_current_track(&self) -> bool {
        self.core.queue.finished_current(self.core.player.elapsed())
    }

    pub fn update(&mut self) {
        if self.finished_current_track() {
            if let Some(audio) = self.core.next.take() {
                self.try_play(audio);
                self.core.queue.advance()
            } else {
                self.core.player.pause()
            }
        } else if self.core.player.is_empty() {
            if let Some(audio) = self.core.next.take() {
                self.try_play(audio)
            }
        }

        if let Some(track) = self.core.queue.prepare_next(self.core.player.elapsed()) {
            self.core.fetch_track(track)
        }

        self.widgets.release.play(self.core.queue.index())
    }
}

fn can_select_down(selected: Option<usize>, len: usize) -> bool {
    selected
        .map(|sel| sel + 1)
        .unwrap_or_default()
        < len
}

fn can_select_up(selected: Option<usize>) -> bool {
    selected.unwrap_or_default() > 0
}
