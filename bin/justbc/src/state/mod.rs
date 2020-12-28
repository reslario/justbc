mod explore;
mod core;

pub use {
    self::core::Core,
    explore::ExploreState
};

use {
    play::Player,
    fetch::Fetcher,
    explore::Explore,
    crate::play::Queue,
    media_keys::MediaKey,
    input::binds::Bindings,
    self::core::{Audio, Stream, Focus},
    gen_tui::widgets::input::Message as InputMessage,
    std::{
        ops::Add,
        error::Error,
        time::Duration
    },
    bc_tui::{
        nav::NavViewState,
        tracks::PlayBarState,
        releases::ReleaseViewState,
    },
    bandcamp_api::data::{
        fans::Fan,
        search::Search,
        outlets::Outlet,
        releases::Release
    }
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Active {
    Library,
    Explore
}

impl Active {
    fn cycle(&mut self) {
        *self = if *self == Active::Library {
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

pub struct State<'a> {
    pub core: Core<'a>,
    pub navigation: Navigation,
    pub widgets: WidgetState,
    pub error: Option<Box<dyn Error>>
}

impl <'a> State<'a> {
    pub fn new(bindings: &'a Bindings, fetcher: Fetcher) -> State {
        State {
            core: Core {
                bindings,
                fetcher,
                focus: <_>::default(),
                queue: <_>::default(),
                next: <_>::default(),
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
            CycleTabs => self.navigation.active.cycle(),
            SelectionDown => self.selection_down(),
            SelectionUp => self.selection_up(),
            ScrollDown => self.scroll_down(),
            ScrollUp => self.scroll_up(),
            Confirm => self.confirm(),
            VolumeUp => self.update_volume(Self::VOL_STEP),
            VolumeDown => self.update_volume(-Self::VOL_STEP),
            NextTrack => self.step_track(Queue::advance),
            PrevTrack => self.step_track(Queue::regress),
            TogglePlay => self.toggle_play(),
            SkipAhead => self.seek(<_>::add),
            SkipBack => self.seek(saturating_sub),
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
                    self.toggle_play()
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
                        self.core.fetcher.query::<Search, _>(input.text());
                        self.navigation.explore = ExploreState::loading();
                        self.focus(Focus::NavBody)
                    },
                    Active::Library => {}
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
            fetch::Response::Fan(fan) => {
                self.try_do(|this| match fan {
                    Ok(mut fan) => {
                        match &mut this.navigation.explore {
                            ExploreState::Fan(existing) if existing.id == fan.id => {
                                existing.collection.append(&mut fan.collection);
                                this.widgets.nav.fan().set_loading(false);
                            },
                            _ => {
                                this.navigation.explore = ExploreState::Fan(fan);
                                this.widgets.nav.fan().collection.select(FIRST);
                            }
                        }

                        Ok(())
                    },
                    Err(e) => {
                        this.navigation.explore = ExploreState::blank();
                        Err(e.into())
                    }
                });
            },
            fetch::Response::Track(stream) => {
                self.try_do(|this| Ok(this
                    .core
                    .next
                    .set(Audio::new(Stream::new(stream?)?))
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

    const VOL_STEP: f32 = 0.05;

    fn update_volume(&mut self, by: f32) {
        let new = (self.core.player.volume() + by)
            .min(1.)
            .max(0.);

        self.core.player.set_volume(new)
    }

    fn step_track(&mut self, op: impl Fn(&mut Queue)) {
        self.widgets.release.play(self.core.step_track(op))
    }

    fn toggle_play(&mut self) {
        if !self.core.toggle_play() {
            self.widgets.release.play(None)
        }
    }

    fn stop(&mut self) {
        self.core.player.stop();
        self.widgets.release.play(None)
    }

    fn try_play(&mut self, audio: Audio) {
        self.try_do(|this| this
            .core
            .player
            .play(audio)
            .map_err(<_>::into)
        );
    }

    pub fn update_device(&mut self) {
        self.try_do(|this| this
            .core
            .player
            .update_device()
            .map_err(<_>::into)
        );
    }

    const SEEK: Duration = Duration::from_secs(5);

    fn seek(&mut self, op: impl Fn(Duration, Duration) -> Duration) {
        let new = op(self.core.player.elapsed(), Self::SEEK)
            .min(self.core.queue
                .current()
                .map(|track| track.duration)
                .unwrap_or_default()
            );

        self.try_do(|this| this.core.player.seek(new).map_err(<_>::into));
    }

    pub fn handle_media_key(&mut self, key: MediaKey) {
        match key {
            MediaKey::Stop => self.stop(),
            MediaKey::PlayPause => self.toggle_play(),
            MediaKey::NextTrack => self.step_track(Queue::advance),
            MediaKey::PrevTrack => self.step_track(Queue::regress)
        }
    }

    fn finished_current_track(&self) -> bool {
        self.core
            .queue
            .current()
            .map(|track| self.core.player.passed(track.duration))
            .unwrap_or_default()
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

        self.core.maybe_fetch_next();

        if !self.core.player.is_paused() {
            self.widgets.release.play(self.core.queue.index())
        }
    }
}

fn saturating_sub(a: Duration, b: Duration) -> Duration {
    a.checked_sub(b).unwrap_or_default()
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
