mod cache;
mod samples;
mod decode;

use {
    cache::*,
    decode::*,
    samples::*,
    std::{
        ops::Range,
        time::Duration,
        io::{self, Read, Seek}
    }
};

pub struct Frame {
    pub samples: Samples,
    pub channels: u16,
    pub sample_rate: u32,
    pub pos: u64
}

impl Frame {
    fn duration(&self) -> Duration {
        let samples = self.samples.len() / self.channels;

        let secs = samples as f64
            / self.sample_rate as f64;

        Duration::from_secs_f64(secs)
    }

    fn start_at(&mut self, duration: Duration) {
        self.samples.set_current(self.samples_in(duration))
    }

    fn samples_in(&self, duration: Duration) -> u16 {
        let samples = duration.as_secs_f64()
            / self.sample_rate as f64;
        samples.round() as _
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
        samples: Samples::new(vec![]),
        sample_rate: 44100,
        channels: 2,
        pos: 0
    }
}

fn empty_range() -> Range<Duration> {
    <_>::default()..<_>::default()
}

/// Decodes an Mp3 from a reader.
/// Supports seeking through the [SeekableSource](seek::SeekableSource) trait.
pub struct Mp3<R: Read> {
    decoder: Decoder<R>,
    cache: FrameCache,
    current: Current
}

impl <R: Read> Mp3<R> {
    pub fn new(reader: R) -> Mp3<R> {
        Mp3 {
            decoder: Decoder::new(reader),
            cache: <_>::default(),
            current: <_>::default()
        }
    }

    fn next_frame(&mut self) -> Result<(), io::Error> {
        self.current.frame = self.decoder.next_frame()?;

        self.current.range.start = self.current.range.end;
        self.current.range.end = self.current.range.start 
            + self.current.frame.duration();

        let cached = CachedFrame {
            range: self.current.range.clone(),
            pos: self.current.frame.pos
        };

        self.cache.set(self.current.frame_index, cached);
        self.current.frame_index += 1;

        Ok(())
    }
}

impl <R: Read> Iterator for Mp3<R> {
    type Item = i16;

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

impl <R: Read + Seek> seek::SeekableSource for Mp3<R> {
    type Error = io::Error;

    fn seek(&mut self, duration: Duration) -> Result<Duration, Self::Error> {
        if self.current.range.contains(&duration) {
            self.current.frame.start_at(duration - self.current.range.start);
            Ok(duration)
        } else if self.current.range.start > duration {
            self.find_left(duration)
        } else {
            self.find_right(duration)
        }
    }
}

impl <R: Read + Seek> Mp3<R> {
    fn find_left(&mut self, duration: Duration) -> Result<Duration, io::Error> {
        let (index, frame) = self.cache
            .enumerated(..self.current.frame_index)
            .rfind(|(_, frame)| frame.range.start <= duration)
            .unwrap_or_default();

        self.current.frame_index = index;
        self.current.range.end = frame.range.start;

        self.decoder.seek(io::SeekFrom::Start(frame.pos))?;

        while {
            self.next_frame()?;
            self.current.range.end < duration
        } {}

        self.current.frame.start_at(duration - self.current.range.start);

        Ok(duration)
    }

    fn find_right(&mut self, duration: Duration) -> Result<Duration, io::Error> {
        let (mut index, mut pos) = (self.current.frame_index, self.current.frame.pos);

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

        self.decoder.seek(io::SeekFrom::Start(pos))?;
        
        while self.current.range.end < duration {
            self.next_frame()?
        }

        self.current.frame.start_at(duration - self.current.range.start);

        Ok(duration)
    }
}
