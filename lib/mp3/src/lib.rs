mod cache;
mod tracked;
mod samples;

use {
    cache::*,
    tracked::*,
    samples::*,
    puremp3::{Mp3Decoder, SampleRate},
    std::{
        ops::Range,
        time::Duration,
        io::{self, Read, Seek}
    }
};

struct Frame {
    samples: Samples,
    sample_rate: SampleRate
}

impl Frame {
    fn duration(&self) -> Duration {
        let secs = self.samples.len() as f64
            / self.sample_rate.hz() as f64;

        Duration::from_secs_f64(secs)
    }

    fn start_at(&mut self, duration: Duration) {
        self.samples.set_current(self.samples_in(duration))
    }

    fn samples_in(&self, duration: Duration) -> usize {
        let samples = duration.as_secs_f64()
            / self.sample_rate.hz() as f64;
        samples.round() as _
    }
}

impl From<puremp3::Frame> for Frame {
    fn from(frame: puremp3::Frame) -> Self {
        Frame {
            samples: Samples::new(frame.samples, frame.num_samples),
            sample_rate: frame.header.sample_rate
        }
    }
}

struct Current {
    frame: Frame,
    frame_index: usize,
    range: Range<Duration>
}

impl Default for Current {
    fn default() -> Self {
        Current {
            frame: empty_frame(),
            frame_index: 0,
            range: empty_range()
        }
    }
}

fn empty_frame() -> Frame {
    Frame {
        samples: Samples::new([[0.; 1152]; 2], 0),
        sample_rate: SampleRate::Hz8000
    }
}

fn empty_range() -> Range<Duration> {
    <_>::default()..<_>::default()
}

/// Decodes an Mp3 from a reader.
/// Supports seeking through the [SeekableSource](seek::SeekableSource) trait.
pub struct Mp3<R: Read> {
    decoder: Mp3Decoder<Tracked<R>>,
    cache: FrameCache,
    current: Current
}

impl <R: Read> Mp3<R> {
    pub fn new(reader: R) -> Mp3<R> {
        Mp3 {
            decoder: Mp3Decoder::new(Tracked::new(reader)),
            cache: <_>::default(),
            current: <_>::default()
        }
    }

    fn next_frame(&mut self) -> Result<(), puremp3::Error> {
        let pos = self.decoder.get_ref().pos();

        self.current.frame = self.decoder.next_frame()?.into();

        self.current.range.start = self.current.range.end;
        self.current.range.end = self.current.range.start 
            + self.current.frame.duration();

        let cached = CachedFrame {
            range: self.current.range.clone(),
            pos
        };

        self.cache.set(self.current.frame_index, cached);
        self.current.frame_index += 1;

        Ok(())
    }
}

impl <R: Read> Iterator for Mp3<R> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.frame.samples
            .next()
            .or_else(|| {
                self.next_frame().ok()?;
                self.current.frame.samples.next()
            })
    }
}

impl <R: Read> rodio::Source for Mp3<R> {
    fn current_frame_len(&self) -> Option<usize> {
        self.current.frame.samples.len().into()
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.current.frame.sample_rate.hz()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl <R: Read + Seek> seek::SeekableSource for Mp3<R> {
    type Error = puremp3::Error;

    fn seek(&mut self, duration: Duration) -> Result<Duration, Self::Error> {
        if self.current.range.contains(&duration) {
            self.current.frame.start_at(duration - self.current.range.start);
            Ok(duration)
        } else {
            if self.current.range.start > duration {
                self.find_left(duration)
            } else {
                self.find_right(duration)
            }
        }
    }
}

impl <R: Read + Seek> Mp3<R> {
    fn find_left(&mut self, duration: Duration) -> Result<Duration, puremp3::Error> {
        let (index, frame) = self.cache
            .enumerated(..self.current.frame_index)
            .rfind(|(_, frame)| frame.range.start <= duration)
            .unwrap_or_default();

        self.current.frame_index = index;
        self.current.range.end = frame.range.start;

        self.decoder
            .get_mut()
            .seek(io::SeekFrom::Start(frame.pos))?;

        while {
            self.next_frame()?;
            self.current.range.end < duration
        } {}

        self.current.frame.start_at(duration - self.current.range.start);

        Ok(duration)
    }

    fn find_right(&mut self, duration: Duration) -> Result<Duration, puremp3::Error> {
        let (mut index, mut pos) = (self.current.frame_index, self.decoder.get_ref().pos());

        for (idx, frame) in self.cache.enumerated(self.current.frame_index..) {
            if frame.range.contains(&duration) {
                index = idx;
                pos = frame.pos;
                break
            }

            if frame.range.start <= duration {
                index = idx;
                pos = frame.pos;
            } else {
                break
            }
        }

        self.current.frame_index = index;

        self.decoder
            .get_mut()
            .seek(io::SeekFrom::Start(pos))?;
        
        while self.current.range.end < duration {
            match self.next_frame() {
                Err(puremp3::Error::IoError(e)) if e.kind() == io::ErrorKind::UnexpectedEof => 
                    return self
                        .cache
                        .latest()
                        .map(|frame| frame.range.end)
                        .ok_or_else(|| e.into()),
                Err(e) => return Err(e),
                _ => {}
            }
        }

        self.current.frame.start_at(duration - self.current.range.start);

        Ok(duration)
    }
}
