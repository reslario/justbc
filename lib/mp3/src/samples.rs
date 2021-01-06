use std::ops;

const MAX_FRAME_SAMPLES: u16 = minimp3_sys::MINIMP3_MAX_SAMPLES_PER_FRAME as _;

pub type Sample = i16;

#[derive(Default)]
pub struct SampleBuf {
    samples: Vec<Sample>
}

impl SampleBuf {
    pub fn set_len(&mut self, len: u16) {
        self.samples.resize(len as _, 0)
    }

    pub fn set_max_len(&mut self) {
        self.set_len(MAX_FRAME_SAMPLES)
    }
}

impl ops::Deref for SampleBuf {
    type Target = [Sample];

    fn deref(&self) -> &Self::Target {
        &self.samples
    }
}

impl ops::DerefMut for SampleBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.samples
    }
}

pub struct Samples {
    samples: SampleBuf,
    current: u16
}

impl Default for Samples {
    fn default() -> Self {
        Samples::new(<_>::default())
    }
}

impl Samples {
    pub fn new(samples: SampleBuf) -> Samples {
        Samples {
            samples,
            current: 0
        }
    }

    pub fn len(&self) -> u16 {
        self.samples.len() as _
    }
    
    pub fn set_current(&mut self, pos: u16) {
        self.current = pos
    }

    pub fn into_buf(self) -> SampleBuf {
        self.samples
    }
}

impl Iterator for Samples {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self
            .samples
            .get(self.current as usize)
            .cloned();

        self.current += 1;
       
        sample
    }
}
