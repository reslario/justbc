use {
    crate::recover::Retriever,
    rodio::{Sink, Source, Sample},
    std::time::{Instant, Duration}
};

/// Resuming playback failed.
/// Contains the elapsed time until this error occured.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CannotResume {
    pub elapsed: Duration
}

/// Controls playback of a sound and tracks its time.
pub struct Track<S> {
    sink: Sink,
    retriever: Retriever<S>,
    elapsed: Duration,
    last_play: Option<Instant>
}

impl <S> Track<S>
where 
    S: Source + Iterator + Send + 'static,
    S::Item: Sample + Send
{
    /// Creates a new Track using the provided `Sink`.
    /// The `Retriever` is used to enable resuming playback
    /// on a different `Sink` using [set_sink](Track::set_sink).
    pub fn new(sink: Sink, retriever: Retriever<S>) -> Track<S> {
        Track {
            sink,
            retriever,
            elapsed: <_>::default(),
            last_play: None
        }
    }

    /// Starts playing the track. No effect if it's already playing.
    pub fn play(&mut self) {
        self.elapsed = self.elapsed();
        self.last_play.replace(Instant::now());
        self.sink.play()
    }

    /// Pauses the track. No effect if it's already paused.
    pub fn pause(&mut self) {
        self.sink.pause();

        self.elapsed = self.elapsed();
        self.last_play = None;
    }

    /// Returns how long this track has been playing.
    pub fn elapsed(&self) -> Duration {
        self.elapsed + self
            .last_play
            .as_ref()
            .map(Instant::elapsed)
            .unwrap_or_default()
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
                self.sink.append(source);
                if !paused {
                    self.play()
                }
            })
            .map_err(|_| CannotResume { elapsed: self.elapsed })
    }

    /// Stops playback and returns the `Sink` that was used.
    pub fn stop(mut self) -> Sink {
        self.pause();
        self.sink.stop();
        self.sink
    }

    /// Sets the volume of the track.
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume)
    }
}
