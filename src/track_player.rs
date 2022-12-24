use crate::track::Track;

pub struct TrackPlayer<'a> {
    track: &'a Track,
    system: mixr::system::AudioSystem,

    current_half_sample: u32
}

impl<'a> TrackPlayer<'a> {
    pub fn new(track: &'a Track) -> Self {
        let system = mixr::system::AudioSystem::new(None, 64);
        
        Self { track, system, current_half_sample: 0 }
    }

    pub fn advance(&mut self) -> i16 {
        

        self.current_half_sample += 1;
        self.system.advance()
    }
}