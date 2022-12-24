use mixr::ChannelProperties;

use crate::{track::Track, PianoKey};

const SAMPLE_RATE: i32 = 48000;

pub struct TrackPlayer<'a> {
    track: &'a Track,
    system: mixr::system::AudioSystem,
    buffers: Vec<i32>,

    current_half_sample: u32,
    half_samples_per_tick: u32,
    current_tick: u8,
    current_speed: u8,

    current_order: usize,
    current_row: usize
}

impl<'a> TrackPlayer<'a> {
    pub fn new(track: &'a Track) -> Self {
        let mut system = mixr::system::AudioSystem::new(Some(mixr::AudioFormat { 
            channels: 2, 
            sample_rate: SAMPLE_RATE, 
            bits_per_sample: 16 }),
        64);
        
        let mut buffers = Vec::with_capacity(track.samples.len());
        for i in 0..track.samples.len() {
            let buffer = system.create_buffer();
            let sample = &track.samples[i];
            system.update_buffer(buffer, &sample.data, sample.format).unwrap();
            buffers.push(buffer);
        }

        Self { 
            track, 
            system,
            buffers,

            current_half_sample: 0,
            half_samples_per_tick: calculate_half_samples_per_tick(track.tempo),
            current_tick: 0,
            current_speed: track.speed,

            current_order: 0,
            current_row: 0
        }
    }

    pub fn advance(&mut self) -> i16 {
        let pattern = &self.track.patterns[self.track.orders[self.current_order] as usize];

        if self.current_tick == 0 {
            for c in 0..pattern.channels {
                let note = pattern.notes.get(c as usize, self.current_row);
                
                if !note.initialized {
                    continue;
                }

                if note.key != PianoKey::None {
                    self.system.play_buffer(self.buffers[note.sample as usize], c, ChannelProperties { 
                        volume: note.volume as f64 / 64.0, 
                        speed: calculate_speed(note.key, note.octave, self.track.samples[note.sample as usize].multiplier), 
                        panning: 0.5, 
                        looping: false, 
                        interpolation_type: mixr::InterpolationType::Linear }).unwrap();
                }
            }
        }

        self.current_half_sample += 1;

        if self.current_half_sample >= self.half_samples_per_tick {
            self.current_tick += 1;

            if self.current_tick >= self.current_speed {
                self.current_tick = 0;
                self.current_row += 1;

                if self.current_row >= pattern.rows as usize {
                    self.current_row = 0;
                    self.current_order += 1;

                    if self.current_order >= self.track.orders.len() {
                        self.current_order = 0;
                    }
                }
            }
        }

        self.system.advance()
    }
}

pub fn calculate_half_samples_per_tick(tempo: u8) -> u32 {
    ((2.5 / tempo as f32) * 2.0 * SAMPLE_RATE as f32) as u32
}

pub fn calculate_speed(key: PianoKey, octave: u8, multiplier: f64) -> f64 {
    if key == PianoKey::NoteCut {
        return 0.0;
    }

    let note = 40 + (key as i32 - 3) + octave as i32 * 12;
    let pow_note = f64::powf(2.0, (note as f64 - 49.0) / 12.0);

    pow_note * multiplier
}