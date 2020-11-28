use {
    std::time::Duration,
    bandcamp_api::data::releases::Track
};

#[derive(Default, Debug)]
pub struct Queue {
    tracks: Vec<Track>,
    current: usize
}

impl Queue {
    pub fn clone_tracks(&mut self, tracks: &Vec<Track>) {
        self.tracks.clone_from(tracks);
        self.current = 0;
    }

    pub fn set_track(&mut self, index: usize) {
        self.current = index
    }

    pub fn current(&self) -> Option<&Track> {
        self.tracks.get(self.current)
    }

    pub fn index(&self) -> usize {
        self.current
    }

    pub fn next(&self) -> Option<&Track> {
        self.tracks.get(self.current + 1)
    }

    pub fn prev(&self) -> Option<&Track> {
        self.tracks.get(self.current.checked_sub(1)?)
    }

    pub fn prepare_next(&self, elapsed: Duration) -> Option<&Track> {
        if self.current()?
            .duration
            .checked_sub(elapsed)
            .unwrap_or_default()
            <= Duration::from_secs(5)
        {
            self.next()   
        } else {  
            None 
        }
            
    }

    pub fn advance(&mut self) {
        if self.next().is_some() {
            self.current += 1
        }
    }

    pub fn regress(&mut self) {
        if self.prev().is_some() {
            self.current -= 1;
        }
    }

    pub fn finished_current(&self, elapsed: Duration) -> bool {
        self.current()
            .map(|track| elapsed >= track.duration)
            .unwrap_or_default()
    }
}
