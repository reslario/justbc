use {
    play::Player,
    fetch::Fetcher,
    std::cell::Cell,
    crate::play::Queue,
    input::binds::Bindings,
    bandcamp_api::data::releases::{Release, Track}
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Focus {
    Release,
    Search,
    NavBody
}

impl Default for Focus {
    fn default() -> Self {
        Focus::NavBody
    }
}

pub type Stream = stream::AudioStream<Box<bc_track::TrackStream>>;
pub type Audio = mp3::Mp3<Stream>;

#[derive(Default)]
pub struct Next {
    track: Option<Audio>,
    pending: Cell<bool>
}

impl Next {
    fn needed(&self) -> bool {
        !self.pending.get()
            && self.track.is_none()
    }

    pub fn set(&mut self, track: Audio) {
        self.track.replace(track);
        self.pending.set(false);
    }

    fn clear(&mut self) {
        self.track = None;
        self.pending.set(false);
    }

    pub fn take(&mut self) -> Option<Audio> {
        self.track
            .take()
            .map(|track| {
                self.pending.set(false);
                track
            })
    }
}

pub struct Core {
    pub(super) bindings: Bindings,
    pub(super) fetcher: Fetcher,
    pub(super) focus: Focus,
    pub queue: Queue,
    pub(super) next: Next,
    pub player: Player<Audio>,
    pub release: Option<Release>,
}

impl Core {
    pub fn set_release(&mut self, release: Release, start_track: usize) {
        self.player.stop();
        self.next.clear();
        self.queue.clone_tracks(&release.tracks);
        self.queue.set_track(start_track);
        if let Some(track) = self.queue.current() {
            self.fetch_track(track)
        }
        self.release.replace(release);
    }

    pub fn play(&mut self, track: usize) {
        self.player.stop();
        self.queue.set_track(track);
        if let Some(track) = self.queue.current() {
            self.fetch_track(track)
        }
    }

    pub fn step_track(&mut self, op: impl Fn(&mut Queue)) -> Option<usize> {
        op(&mut self.queue);

        let track = match self.queue.current() {
            Some(track) => Some(track),
            None => {
                self.queue.set_track(0);
                self.queue.current()
            }
        }?;

        self.player.stop();
        self.fetch_track(track);
        self.queue.index().into()
    }

    pub fn fetch_track(&self, track: &Track) {
        self.next.pending.set(true);
        self.fetcher.fetch_track(track.stream.mp3_128.clone())
    }

    pub fn toggle_play(&mut self) -> bool {
        if self.player.is_paused() {
            self.player.resume();
            true
        } else {
            self.player.pause();
            false
        }
    }

    pub fn maybe_fetch_next(&self) {
        if self.next.needed() {
            if let Some(track) = self.queue.prepare_next(self.player.elapsed()) {
                self.fetch_track(track)
            }
        }
    }
}
