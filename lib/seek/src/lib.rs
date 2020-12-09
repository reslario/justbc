use std::time::Duration;

/// An audio source that supports seeking to a specific duration.
pub trait SeekableSource {
    type Error;

    fn seek(&mut self, duration: Duration) -> Result<Duration, Self::Error>;
}
