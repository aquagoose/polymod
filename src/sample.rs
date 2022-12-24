use mixr::AudioFormat;

pub struct Sample {
    pub data: Vec<u8>,
    pub format: AudioFormat,
    pub multiplier: f64
}

impl Sample {
    pub fn new(data: &[u8], format: AudioFormat) -> Self {
        let multiplier = format.sample_rate as f64 / (crate::track_player::calculate_speed(crate::PianoKey::C, 5, 1.0) * format.sample_rate as f64);

        let mut d_vec = data.to_vec();
        fix_sample(&mut d_vec);

        Self { 
            data: d_vec, 
            format, 
            multiplier
        }
    }
}

fn fix_sample(data: &mut Vec<u8>) {
    for i in 0..data.len() {
        data[i] = (data[i] as i32 - 128) as u8;
    }
}