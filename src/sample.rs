use mixr::AudioFormat;

pub struct Sample {
    pub data: Vec<u8>,
    pub format: AudioFormat,
    pub multiplier: f64,

    pub looping: bool,
    pub loop_start: i32,
    pub loop_end: i32,

    pub global_volume: u8
}

impl Sample {
    pub fn new(data: &[u8], format: AudioFormat, looping: bool, loop_start: i32, loop_end: i32, global_volume: u8) -> Self {
        let multiplier = format.sample_rate as f64 / (crate::track_player::calculate_speed(crate::PianoKey::C, 5, 1.0) * format.sample_rate as f64);

        let mut d_vec = data.to_vec();
        fix_sample(&mut d_vec, &format);

        Self { 
            data: d_vec, 
            format, 
            multiplier,
            looping,
            loop_start,
            loop_end,

            global_volume
        }
    }
}

fn fix_sample(data: &mut Vec<u8>, format: &AudioFormat) {
    if format.bits_per_sample == 8 {
        for i in 0..data.len() {
            data[i] = (data[i] as i32 - 128) as u8;
        }
    }

    if format.channels == 2 {
        let old_data = data.clone();
        data.clear();
        let alignment = (format.bits_per_sample / 8) as usize;
        let mut side = true;

        for i in 0..old_data.len() {
            if i % alignment == 0 {
                side = !side;
            }

            
        }
    }
}