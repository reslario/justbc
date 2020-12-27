mod recover;
mod handle;
mod track;
mod tick;

use {
    track::Track,
    handle::Handle,
    rodio::{Source, Sample},
    snafu::{Snafu, ResultExt},
    std::{
        fmt,
        time::Duration,
    }
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false), display("Error playing sound: {}", source))]
    Play { source: PlayError },
    #[snafu(context(false), display("Error opening output stream: {}", source))]
    Stream { source: StreamError }
}

pub use rodio::{PlayError, StreamError};

/// Controls playback of a sound on the default output device
/// and tracks its time.
///
/// Also allows for updating the device it's playing to, should
/// it need to change.
pub struct Player<S> {
    handle: Option<Handle>,
    current: Option<Track<S>>,
    volume: f32
}

impl <S> Default for Player<S> {
    fn default() -> Self {
        Player {
            handle: None,
            current: None,
            volume: 1.
        }
    }
} 

impl <S> Player<S> 
where 
    S: Source + Iterator + Send + 'static,
    S::Item: Sample + Send
{
    /// Creates a new `Player`.
    pub fn new() -> Player<S> {
        <_>::default()
    }

    /// Plays the provided sound.
    pub fn play(&mut self, sound: S) -> Result<(), Error> {
        self.set_source(sound)?;
        self.mut_track(Track::play);

        Ok(())
    }
    
    fn set_source(&mut self, source: S) -> Result<(), Error> {
        let sink = self
            .init_handle()?
            .new_sink()?;

        sink.set_volume(self.volume);
        
        self.current.replace(Track::new(sink, source));

        Ok(())
    }

    fn init_handle(&mut self) -> Result<&mut Handle, StreamError> {
        if self.handle.is_none() {
            self.handle = Handle::new()?.into();
        }
        
        Ok(self.handle.as_mut().unwrap())
    }

    fn ref_track<T>(&self, f: impl Fn(&Track<S>) -> T) -> Option<T> {
        self.current.as_ref().map(f)
    }

    fn mut_track(&mut self, f: impl Fn(&mut Track<S>)) {
        self.current.as_mut().map(f);
    }

    /// Stops the current track.
    /// No effect if nothing is playing.
    pub fn stop(&mut self) {
        self.current = None;
    }

    /// Pauses the current track.
    /// No effect if nothing is playing.
    pub fn pause(&mut self) {
        self.mut_track(Track::pause)
    }

    /// Returns whether the player is paused.
    /// Returns `false` if the player is empty.
    pub fn is_paused(&self) -> bool {
        self.ref_track(Track::is_paused)
            .unwrap_or_default()
    }

    /// Resumes playback of a paused sound, if there is one.
    pub fn resume(&mut self) {
        self.mut_track(Track::play)
    }

    /// Returns how long the current sound has been playing,
    /// or an empty duration if there is none.
    pub fn elapsed(&self) -> Duration {
        self.ref_track(Track::elapsed)
            .unwrap_or_default()
    }

    /// Attempts to resume playback on a new default output device,
    /// if it has changed.
    pub fn update_device(&mut self) -> Result<(), UpdateDeviceError> {
        let handle = Handle::new()
            .map_err(Error::from)?;

        if let Some(track) = self.current.as_mut() {
            let sink = handle
                .new_sink()
                .map_err(Error::from)?;

            track.set_sink(sink)
                .map_err(|_| UpdateDeviceError::Resume)?
        }

        self.handle.replace(handle);

        Ok(())
    }

    /// Gets the volume of the player.
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Sets the volume of the player, which persists between
    /// playing different sounds.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.mut_track(|track| track.set_volume(volume))
    }

    /// Returns whether the player is empty.
    pub fn is_empty(&self) -> bool {
        self.current.is_none()
    }

    /// Returns whether the current track has been playing for
    /// the provided duration.
    pub fn passed(&self, duration: Duration) -> bool {
        duration
            .checked_sub(self.elapsed())
            .map(|diff| diff <= Track::<S>::TICK_INTERVAL)
            .unwrap_or(true)
    }

    /// Seeks to the specified duration in the current track, if one exists.
    pub fn seek(&mut self, duration: Duration) -> Result<(), SeekError<S::Error>>
    where 
        S: seek::SeekableSource,
        S::Error: fmt::Display + snafu::Error
    {
        let paused = self.is_paused();

        if let Some(track) = self.current.take() {
            let mut source = track
                .into_source()
                .map_err(|_| SeekError::ResumePlayback)?;

            let elapsed = source
                .seek(duration)
                .context(Seek)?;

            if paused {
                self.set_source(source)
            } else {
                self.play(source)
            }?;

            self.mut_track(|track| track.set_elapsed(elapsed));
        }

        Ok(())
    }
}

const FAILED_RESUME: &str = "failed to resume playback";

/// Errors that can occur when seeking.
#[derive(Debug, Snafu)]
pub enum SeekError<E: fmt::Display + snafu::Error + 'static> {
    /// An error occured when seeking.
    #[snafu(display("failed to seek: {}", source))]
    Seek { source: E },
    /// A general playback error occured.
    #[snafu(context(false), display("playback error when seeking: {}", source))]
    General { source: Error },
    /// Resuming playback was not possible.
    #[snafu(display("{}", FAILED_RESUME))]
    ResumePlayback
}

/// Errors that can occur when updating the device.
#[derive(Debug, Snafu)]
pub enum UpdateDeviceError {
    /// A general playback error occured.
    #[snafu(context(false), display("{}", source))]
    General { source: Error },
    /// Resuming playback was not possible.
    #[snafu(display("{}", FAILED_RESUME))]
    Resume
}
