use {
    rodio::{Source, Sample},
    std::{
        time::Duration,
        sync::{
            Arc,
            atomic::{AtomicU32, Ordering}
        }
    }
};

#[derive(Default, Clone)]
pub struct Ticks {
    counter: Arc<AtomicU32>
}

impl Ticks {
    fn incr(&self) {
        self.counter.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get(&self) -> u32 {
        self.counter.load(Ordering::SeqCst)
    }

    pub fn set(&self, counter: u32) {
        self.counter.store(counter, Ordering::SeqCst)
    }
}

/// Tracks the elapsed time of a `Source`
/// by incrementing a shared counter.
pub struct Ticking<S> {
    source: S,
    interval: u32,
    remaining: u32,
    sample_rate: u32,
    ticks: Ticks
}

impl <S> Ticking<S>
where
    S: Source + Iterator,
    S::Item: Sample 
{
    pub fn new(source: S, interval: Duration, ticks: Ticks) -> Ticking<S> {
        let sample_rate = source.sample_rate();

        let interval = interval.as_millis() as u32
            * sample_rate
            / 1000 
            * source.channels() as u32;

        let interval = interval.max(1);

        Ticking {
            source,
            interval,
            remaining: interval,
            sample_rate,
            ticks,
        }
    }

    fn update_sample_rate(&mut self) {
        let new = self.source.sample_rate();

        if new == self.sample_rate { return }

        let ratio = new as f32 / self.sample_rate as f32;

        self.interval = (self.interval as f32 * ratio) as u32;
        self.remaining = (self.remaining as f32 * ratio) as u32;

        self.sample_rate = new;
    }

    pub fn into_inner(self) -> S {
        self.source
    }
} 

impl <S> Iterator for Ticking<S>
where
    S: Source + Iterator,
    S::Item: Sample
{
    type Item = S::Item;

    fn next(&mut self) -> Option<S::Item> {
        self.update_sample_rate();

        if self.remaining == 0 {
            self.ticks.incr();
            self.remaining = self.interval;
        }

        self.remaining -= 1;

        self.source.next()
    }
}

impl <S> Source for Ticking<S>
where
    S: Source + Iterator,
    S::Item: Sample 
{
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}
