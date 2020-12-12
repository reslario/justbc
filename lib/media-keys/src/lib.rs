#[cfg(not(windows))]
mod none;

#[cfg(windows)]
mod windows;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MediaKey {
    PlayPause,
    Stop,
    NextTrack,
    PrevTrack
}

#[cfg(windows)]
type Inner = windows::Listener;

#[cfg(not(windows))]
type Inner = none::None;

#[cfg(windows)]
pub type Error = windows::Error;

#[cfg(not(windows))]
pub type Error = none::Error;

pub struct Listener(Inner);

impl Listener {
    pub fn new() -> Result<Listener, Error> {
        Inner::new()
            .map(Listener)
    }

    pub fn keys(&self) -> impl Iterator<Item = MediaKey> + '_ {
        self.0.keys()
    }
}
