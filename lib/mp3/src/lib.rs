mod cache;
mod decode;
mod samples;
mod span;

use {
    cache::*,
    decode::*,
    samples::*,
    span::FrameSpan,
    std::{
        io::{self, Read, Seek},
        mem,
        time::Duration,
    },
};

const MICROS_PER_SEC: u32 = Duration::from_secs(1).as_micros() as _;

pub struct Frame {
    pub samples: Samples,
    pub channels: u16,
    pub sample_rate: u32,
    pub pos: u64,
}

impl Frame {
    fn micros(&self) -> u32 {
        let samples = self.samples.len() / self.channels;

        let secs = samples as f64 / self.sample_rate as f64;

        let micros = secs * MICROS_PER_SEC as f64;

        micros.floor() as u32
    }

    fn start_at(&mut self, duration: Duration) {
        self.samples.set_current(self.samples_in(duration))
    }

    fn samples_in(&self, duration: Duration) -> u16 {
        let samples = duration.as_secs_f64() / self.sample_rate as f64;
        samples.round() as _
    }
}

struct Current {
    frame: Frame,
    frame_index: usize,
    span: FrameSpan,
}

impl Default for Current {
    fn default() -> Self {
        Current {
            frame: empty_frame(),
            frame_index: 0,
            span: FrameSpan::empty(),
        }
    }
}

fn empty_frame() -> Frame {
    Frame {
        samples: <_>::default(),
        sample_rate: 44100,
        channels: 2,
        pos: 0,
    }
}

/// Decodes an Mp3 from a reader.
/// Supports seeking through the [SeekableSource](seek::SeekableSource) trait.
pub struct Mp3<R: Read> {
    decoder: Decoder<R>,
    cache: FrameCache,
    current: Current,
}

impl<R: Read> Mp3<R> {
    pub fn new(reader: R) -> Mp3<R> {
        Mp3 {
            decoder: Decoder::new(reader),
            cache: <_>::default(),
            current: <_>::default(),
        }
    }

    fn next_frame(&mut self) -> Result<(), io::Error> {
        self.current.frame = {
            let samples = mem::take(&mut self.current.frame.samples);
            self.decoder.next_frame(samples.into_buf())?
        };

        self.current.span = FrameSpan::new(self.current.span.end(), self.current.frame.micros());

        let cached = CachedFrame {
            span: self.current.span,
            pos: self.current.frame.pos,
        };

        self.cache.set(self.current.frame_index, cached);
        self.current.frame_index += 1;

        Ok(())
    }
}

impl<R: Read> Iterator for Mp3<R> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.frame.samples.next().or_else(|| {
            self.next_frame().ok()?;
            self.current.frame.samples.next()
        })
    }
}

impl<R: Read> rodio::Source for Mp3<R> {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.current.frame.samples.len() as _)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.current.frame.sample_rate as _
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl<R: Read + Seek> seek::SeekableSource for Mp3<R> {
    type Error = io::Error;

    fn seek(&mut self, duration: Duration) -> Result<Duration, Self::Error> {
        if self.current.span.contains(duration) {
            self.current
                .frame
                .start_at(duration - self.current.span.start_duration());
            Ok(duration)
        } else if self.current.span.start_duration() > duration {
            self.find_left(duration)
        } else {
            self.find_right(duration)
        }
    }
}

impl<R: Read + Seek> Mp3<R> {
    fn find_left(&mut self, duration: Duration) -> Result<Duration, io::Error> {
        let (index, frame) = self
            .cache
            .enumerated(..self.current.frame_index)
            .rfind(|(_, frame)| frame.span.start_duration() <= duration)
            .unwrap_or_default();

        self.current.frame_index = index;
        self.current.span = frame.span.shift_back();

        self.decoder.seek(io::SeekFrom::Start(frame.pos))?;

        while {
            self.next_frame()?;
            self.current.span.end_duration() < duration
        } {}

        self.current
            .frame
            .start_at(duration - self.current.span.start_duration());

        Ok(duration)
    }

    fn find_right(&mut self, duration: Duration) -> Result<Duration, io::Error> {
        let (mut index, mut pos) = (self.current.frame_index, self.current.frame.pos);

        for (idx, frame) in self.cache.enumerated(self.current.frame_index..) {
            if frame.span.contains(duration) {
                index = idx;
                pos = frame.pos;
                break
            }

            if frame.span.start_duration() <= duration {
                index = idx;
                pos = frame.pos;
            } else {
                break
            }
        }

        self.current.frame_index = index;

        self.decoder.seek(io::SeekFrom::Start(pos))?;

        while self.current.span.end_duration() < duration {
            self.next_frame()?
        }

        self.current
            .frame
            .start_at(duration - self.current.span.start_duration());

        Ok(duration)
    }
}
