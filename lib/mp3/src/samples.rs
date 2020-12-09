#[derive(Copy, Clone)]
enum Channel {
    Left = 0,
    Right = 1
}

impl std::ops::Not for Channel {
    type Output = Channel;

    fn not(self) -> Self::Output {
        match self {
            Channel::Left => Channel::Right,
            Channel::Right => Channel::Left
        }
    }
}

pub struct Samples {
    samples: [[f32; 1152]; 2],
    len: usize,
    current: usize,
    channel: Channel
}

impl Samples {
    pub fn new(samples: [[f32; 1152]; 2], len: usize) -> Samples {
        Samples {
            samples,
            len,
            current: 0,
            channel: Channel::Left
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn set_current(&mut self, pos: usize) {
        self.current = pos;
        self.channel = Channel::Left
    }
}

impl Iterator for Samples {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.len {
            return None
        }

        let channel = self.channel as usize;

        let sample = self.samples[channel][self.current];

        self.current += channel;
        self.channel = !self.channel;

        sample.into()
    }
}
