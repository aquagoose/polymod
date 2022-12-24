use mixr::AudioFormat;

pub struct Sample {
    pub data: Vec<u8>,
    pub format: AudioFormat,
    pub multiplier: f64
}

impl Sample {
    pub fn new(data: &[u8], format: AudioFormat) -> Self {
        Self { 
            data: data.to_vec(), 
            format, 
            multiplier: crate::track_player::calculate_speed(crate::PianoKey::C, 5, 1.0) 
        }
    }
}