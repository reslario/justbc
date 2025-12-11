use std::{fmt, time::Duration};

#[allow(non_camel_case_types)]
type u24 = [u8; 3];

#[derive(Copy, Clone)]
/// Represents the time span an mp3 frame occupies.
/// Measurements are in microseconds.
pub struct FrameSpan {
    start: u32,
    // mp3 frames are very short,
    // so 24 bits are enough to represent one's length
    len: u24,
}

impl FrameSpan {
    pub fn new(start: u32, len: u32) -> FrameSpan {
        let [a, b, c, _] = len.to_le_bytes();

        FrameSpan {
            start,
            len: [a, b, c],
        }
    }

    pub fn empty() -> FrameSpan {
        FrameSpan::new(0, 0)
    }

    pub fn contains(&self, duration: Duration) -> bool {
        use std::convert::TryFrom;

        let micros = match u32::try_from(duration.as_micros()) {
            Ok(micros) => micros,
            Err(_) => return false,
        };

        micros >= self.start && micros < self.start + self.len()
    }

    pub fn len(&self) -> u32 {
        let [a, b, c] = self.len;
        u32::from_le_bytes([a, b, c, 0])
    }

    pub fn is_empty(&self) -> bool {
        self.start | self.len() == 0
    }

    pub fn end(&self) -> u32 {
        self.start + self.len()
    }

    pub fn start_duration(&self) -> Duration {
        Duration::from_micros(self.start.into())
    }

    pub fn end_duration(&self) -> Duration {
        Duration::from_micros(self.end().into())
    }

    pub fn shift_back(mut self) -> FrameSpan {
        self.start -= self.len();
        self
    }
}

impl fmt::Debug for FrameSpan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let range = self.start_duration()..self.end_duration();
        range.fmt(f)
    }
}
