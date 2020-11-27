mod recover;
mod handle;
mod track;

use {
    track::Track,
    handle::Handle,
    snafu::ResultExt,
    std::time::Duration,
    recover::Recoverable,
    rodio::{Source, Sample}
};

#[derive(Debug, snafu::Snafu)]
pub enum Error {
    #[snafu(display("Error playing sound: {}", source))]
    Play { source: PlayError },
    #[snafu(display("Error opening output stream: {}", source))]
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

impl <S> Player<S> 
where 
    S: Source + Iterator + Send + 'static,
    S::Item: Sample + Send
{
    /// Creates a new `Player`.
    pub fn new() -> Player<S> {
        Player {
            handle: None,
            current: None,
            volume: 1.
        }
    }

    /// Plays the provided sound.
    pub fn play(&mut self, sound: S) -> Result<(), Error> {
        let (source, retr) = Recoverable::new(sound);

        let sink = self
            .init_handle()
            .context(Stream)?
            .new_sink()
            .context(Play)?;

        sink.set_volume(self.volume);
        sink.append(source);

        let mut track = Track::new(sink, retr);
        track.play();
        
        self.current.replace(track);

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
            .context(Stream)
            .map_err(UpdateDeviceError::General)?;

        if let Some(track) = self.current.as_mut() {
            let sink = handle
                .new_sink()
                .context(Play)
                .map_err(UpdateDeviceError::General)?;

            track.set_sink(sink)
                .map_err(UpdateDeviceError::Resume)?
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
}

/// Errors that can occur when updating the device.
#[derive(Debug)]
pub enum UpdateDeviceError {
    /// A general playback error occured
    General(Error),
    /// Resuming playback is not possible.
    Resume(track::CannotResume)
}
