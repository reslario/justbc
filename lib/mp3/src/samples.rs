pub type Sample = i16;

pub struct Samples {
    samples: Vec<i16>,
    current: u16
}

impl Samples {
    pub fn new(samples: Vec<Sample>) -> Samples {
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
