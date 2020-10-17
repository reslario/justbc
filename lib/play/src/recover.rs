use {
    std::sync::mpsc,
    rodio::{Source, Sample}
};

/// Enables a [Source](rodio::Source) to be recovered after playing it through a [Sink].
///
/// Normally, a [Source] cannot be accessed again after supplying it to a [Sink], which
/// can be annoying when you want to continue a partially played sound from another [Sink].
/// Wrapping it in this type lets you work around that.
///
/// When this type is dropped, for example by calling [stop](rodio::Sink::stop) on its [Sink]
/// or letting its [Sink] go out of scope, it is sent to its corresponding [Retriever](Retriever)
/// instance and can be retrieved from there.  
///
/// [Source]: rodio::Source
/// [Sink]: rodio::Sink
pub struct Recoverable<S> {
    source: Option<S>,
    sender: mpsc::SyncSender<S>
}

impl <S> Recoverable<S> {
    /// Creates a recoverable version of the provided source
    /// and a [Retriever](Retriever) to retrieve it.
    pub fn new(source: S) -> (Recoverable<S>, Retriever<S>) {
        let (sender, receiver) = mpsc::sync_channel(1);

        (
            Recoverable { source: source.into(), sender },
            Retriever { receiver }
        )
    }
}

impl <S> Drop for Recoverable<S> {
    fn drop(&mut self) {
        if let Some(src) = self.source.take() {
            self.sender
                .send(src)
                // if the receiver was dropped, they probably
                // didn't want the source back anyway
                .ok();
        }
    }
}

impl <S> Source for Recoverable<S>
where 
    S: Source + Iterator,
    S::Item: Sample    
{
    fn current_frame_len(&self) -> Option<usize> {
        self.source
            .as_ref()
            .and_then(<_>::current_frame_len)
    }

    fn channels(&self) -> u16 {
        self.source
            .as_ref()
            .map(<_>::channels)
            .unwrap_or_default()
    }

    fn sample_rate(&self) -> u32 {
        self.source
            .as_ref()
            .map(<_>::sample_rate)
            .unwrap_or_default()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.source
            .as_ref()
            .and_then(<_>::total_duration)
    }
}

impl <S: Iterator> Iterator for Recoverable<S> {
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.source
            .as_mut()
            .and_then(<_>::next)
    }
}

/// Used to retrieve a [Recovered](Recoverable) [Source](rodio::Source).
pub struct Retriever<S> {
    receiver: mpsc::Receiver<S>
}

impl <S> Retriever<S> {
    /// Waits for the source to become available again, blocking
    /// the current thread.
    pub fn wait(&self) -> Result<S, mpsc::RecvError> {
        self.receiver.recv()
    }

    /// Attempts to retrieve the recovered source without blocking.
    /// returns `None` if it isn't available (yet).
    #[allow(dead_code)]
    pub fn try_get(&self) -> Option<S> {
        self.receiver
            .try_recv()
            .ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn recover() {
        let (source, rec) = Recoverable::new(rodio::source::Zero::<f32>::new(13, 12));

        {
            let (sink, _) = rodio::Sink::new_idle();
            sink.append(source);
            sink.play();
            std::thread::sleep(std::time::Duration::from_millis(500));
            sink.stop();
        }

        assert!(rec.try_get().is_some())
    }
}
