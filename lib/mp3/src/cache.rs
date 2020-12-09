use std::{
    ops::Range,
    time::Duration
};

#[derive(Clone)]
pub struct CachedFrame {
    pub range: Range<Duration>,
    pub pos: u64
}

impl CachedFrame {
    fn placeholder(&self) -> bool {
        self.range.is_empty()
    }
}

impl Default for CachedFrame {
    fn default() -> Self {
        CachedFrame {
            range: super::empty_range(),
            pos: 0
        }
    }
}

#[derive(Default)]
pub struct FrameCache {
    frames: Vec<CachedFrame>
}

impl FrameCache {
    pub fn set(&mut self, index: usize, frame: CachedFrame) {
        if index <= self.frames.len() {
            self.frames.resize(index + 1, <_>::default())
        }

        self.frames[index] = frame
    }

    pub fn enumerated<R>(&self, range: R) -> impl DoubleEndedIterator<Item = (usize, CachedFrame)> + '_
    where R: std::slice::SliceIndex<[CachedFrame], Output = [CachedFrame]> {
        self.frames[range]
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, frame)| !frame.placeholder())
    }

    pub fn latest(&self) -> Option<&CachedFrame> {
        self.frames
            .iter()
            .rev()
            .find(|frame| !frame.placeholder())
    }
}
