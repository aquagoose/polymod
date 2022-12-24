use mixr::AudioFormat;

pub struct Sample {
    pub data: Vec<u8>,
    pub format: AudioFormat
}

impl Sample {
    pub fn new(data: &[u8], format: AudioFormat) -> Self {
        Self { data: data.to_vec(), format }
    }
}