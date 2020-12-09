mod none;
mod windows;

use std::io;

pub trait EventSource: Sized {
    fn new() -> io::Result<Self>;
    fn device_updated(&self) -> bool;
}

#[cfg(windows)]
type Source = windows::Watcher;

#[cfg(not(windows))]
type Source = none::None;

pub struct Watcher(Source);

impl Watcher where Source: EventSource {
    pub fn new() -> io::Result<Watcher> {
        <_>::new().map(Watcher)
    }

    pub fn device_updated(&self) -> bool {
        self.0.device_updated()
    }
}
