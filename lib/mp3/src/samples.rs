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
    len: u16,
    current: u16,
    channel: Channel
}

impl Samples {
    pub fn new(samples: [[f32; 1152]; 2], len: u16) -> Samples {
        Samples {
            samples,
            len,
            current: 0,
            channel: Channel::Left
        }
    }

    pub fn len(&self) -> u16 {
        self.len
    }
    
    pub fn set_current(&mut self, pos: u16) {
        self.current = pos;
        self.channel = Channel::Left
    }
}

impl Iterator for Samples {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self
            .samples[self.channel as usize]
            .get(self.current as usize)
            .cloned();

        self.current += self.channel as u16;
        self.channel = !self.channel;
       
        sample
    }
}
