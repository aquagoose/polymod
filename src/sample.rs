use mixr::AudioFormat;

pub struct Sample {
    pub data: Vec<u8>,
    pub format: AudioFormat,
    pub multiplier: f64
}

impl Sample {
    pub fn new(data: &[u8], format: AudioFormat) -> Self {
        let multiplier = format.sample_rate as f64 / (crate::track_player::calculate_speed(crate::PianoKey::C, 5, 1.0) * format.sample_rate as f64);

        Self { 
            data: data.to_vec(), 
            format, 
            multiplier
        }
    }
}