use {
    std::time::Duration,
    rodio::{Sink, Source, Sample},
    crate::{
        tick::{Ticks, Ticking},
        recover::{Recoverable, Retriever},
    }
};

/// Resuming playback failed.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CannotResume;

/// Controls playback of a sound and tracks its time.
pub struct Track<S> {
    sink: Sink,
    retriever: Retriever<Ticking<S>>,
    ticks: Ticks
}

impl <S> Track<S>
where 
    S: Source + Iterator + Send + 'static,
    S::Item: Sample + Send
{
    const TICK_INTERVAL: Duration = Duration::from_millis(100);

    /// Creates a new Track using the provided `Sink`
    /// and prepares the provided `Source`.
    pub fn new(sink: Sink, source: S) -> Track<S> {
        let ticks = Ticks::default();
        let source = Ticking::new(source, Self::TICK_INTERVAL, ticks.clone());
        let (source, retriever) = Recoverable::new(source);

        sink.append(source);

        Track {
            sink,
            retriever,
            ticks
        }
    }

    /// Starts or resumes playing the track. No effect if it's already playing.
    pub fn play(&mut self) {
        self.sink.play()
    }

    /// Pauses the track. No effect if it's already paused.
    pub fn pause(&mut self) {
        self.sink.pause();
    }

    /// Returns whether the track is paused.
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Returns how long this track has been playing.
    pub fn elapsed(&self) -> Duration {
       Self::TICK_INTERVAL * self.ticks.get()
    }

    /// Attempts to resume playback on a different `Sink`.
    pub fn set_sink(&mut self, sink: Sink) -> Result<(), CannotResume> {
        sink.set_volume(self.sink.volume());

        let paused = self.sink.is_paused();
        self.pause();

        self.sink = sink;
        self.retriever
            .wait()
            .map(|source| {
                let (source, retr) = Recoverable::new(source);

                self.retriever = retr;
                self.sink.append(source);

                if !paused {
                    self.play()
                }
            })
            .map_err(|_| CannotResume)
    }

    /// Sets the volume of the track.
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume)
    }

    /// Sets the elapsed time directly.
    pub fn set_elapsed(&mut self, elapsed: Duration) {
        let ticks = elapsed.as_millis() 
            / Self::TICK_INTERVAL.as_millis();

        self.ticks.set(ticks as _)
    }

    /// Consumes the track and returns the source it was playing.
    pub fn into_source(mut self) -> Result<S, CannotResume> {
        self.pause();
        drop(self.sink);

        self.retriever
            .wait()
            .map(Ticking::into_inner)
            .map_err(|_| CannotResume)
    }
}
