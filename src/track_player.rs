use mixr::ChannelProperties;

use crate::{track::Track, PianoKey, Effect, sample::Sample, Note};

pub const SAMPLE_RATE: i32 = 48000;

struct TrackChannel {
    pub properties: ChannelProperties,
    pub enabled: bool,

    pub current_sample: Option<u8>,
    pub note_volume: u8,

    pub vol_memory: u8
}

pub struct TrackPlayer<'a> {
    track: &'a Track,
    system: mixr::system::AudioSystem,
    buffers: Vec<i32>,

    current_half_sample: u32,
    half_samples_per_tick: u32,
    current_tick: u8,
    current_speed: u8,

    current_order: usize,
    current_row: usize,

    next_row: usize,
    next_order: usize,

    should_jump: bool,

    channels: Vec<TrackChannel>,

    pub tuning: f64
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

        let mut channels = Vec::with_capacity(system.num_channels() as usize);
        for i in 0..system.num_channels() {
            let mut properties = ChannelProperties::default();

            let pan = track.pans[i as usize];
            properties.panning = pan as f64 / 64.0;
            // A pan value of >= 128 means the channel is disabled and will not be played.
            channels.push(TrackChannel { properties, enabled: pan >= 128, current_sample: None, note_volume: 0, vol_memory: 0 });
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
            current_row: 0,
            
            next_row: 0,
            next_order: 0,

            should_jump: false,

            channels,

            tuning: 1.0
        }
    }

    pub fn advance(&mut self) -> i16 {
        let pattern = &self.track.patterns[self.track.orders[self.current_order] as usize];

        if self.current_half_sample == 0 {
            for c in 0..pattern.channels {
                let mut channel = &mut self.channels[c as usize];

                //if !channel.enabled {
                //    continue;
               // }

                let note = pattern.notes.get(c as usize, self.current_row);
                
                if !note.initialized {
                    continue;
                }

                if self.current_tick == 0 {
                    if note.key == PianoKey::NoteCut || note.key == PianoKey::NoteOff || note.key == PianoKey::NoteFade {
                        channel.current_sample = None;
                        channel.note_volume = 0;
                        self.system.stop(c).unwrap();
                        continue;
                    }

                    let mut sample_id = note.sample;
                    if sample_id.is_none() {
                        sample_id = channel.current_sample;
                    }

                    if let Some(sample_id) = sample_id {
                        if note.key != PianoKey::None && sample_id < self.buffers.len() as u8 {
                            let sample = &self.track.samples[sample_id as usize];
                            let properties = &mut channel.properties;
                            let volume = note.volume.unwrap_or(64);
                            properties.volume = ((volume as u32 * sample.global_volume as u32 * 64 * self.track.global_volume as u32) >> 18) as f64 / 128.0 * (self.track.mix_volume as f64 / u8::MAX as f64);
                            properties.speed = calculate_speed(note.key, note.octave, sample.multiplier) * self.tuning;
                            properties.looping = sample.looping;
                            properties.loop_start = sample.loop_start;
                            properties.loop_end = sample.loop_end;

                            self.system.play_buffer(self.buffers[sample_id as usize], c, channel.properties).unwrap();
                            
                            channel.current_sample = note.sample;
                            channel.note_volume = volume;
                        }
                    }

                    if let (Some(volume), Some(sample)) = (note.volume, channel.current_sample) {
                        let sample = &self.track.samples[sample as usize];
                        channel.properties.volume = ((volume as u32 * sample.global_volume as u32 * 64 * self.track.global_volume as u32) >> 18) as f64 / 128.0 * (self.track.mix_volume as f64 / u8::MAX as f64);
                        self.system.set_channel_properties(c, channel.properties).unwrap();
                        channel.note_volume = volume;
                    }
                }

                match note.effect {
                    Effect::None => {},
                    Effect::SetSpeed => if self.current_tick == 0 { self.current_speed = note.effect_param },
                    Effect::PositionJump => {
                        self.next_row = 0;
                        self.next_order = note.effect_param as usize;
                        self.should_jump = true;
                    },
                    Effect::PatternBreak => {
                        self.next_order = self.current_order + 1;
                        self.next_row = note.effect_param as usize;
                        self.should_jump = true;
                    },
                    Effect::VolumeSlide => {
                        // If the note parameter is 0, we just fetch the last one stored in memory.
                        // If the last parameter is also 0 then nothing happens.
                        let vol_param = if note.effect_param == 0 { channel.vol_memory } else { note.effect_param };
                        channel.vol_memory = vol_param;

                        // Handle DFy and DxF, if 'F' is set then the volume slide only occurs on the first tick.
                        if (vol_param < 0xF0 && (vol_param & 0xF) != 0xF) && self.current_tick != 0 {
                            continue;
                        }
                        // Volume slide occurs on every tick except the first, **unless** it is D0F.
                        else if self.current_tick == 0 && vol_param != 15 {
                            continue;
                        }

                        if channel.current_sample.is_none() {
                            continue;
                        }

                        let sample_id = channel.current_sample.unwrap();

                        let mut volume = channel.note_volume as i32;

                        // D0y decreases volume by y units.
                        // Dx0 increases volume by x units.
                        if vol_param < 16 {
                            volume -= vol_param as i32;
                        } else {
                            volume += vol_param as i32 / 16;
                        }

                        // Volume cannot exceed 64.
                        channel.note_volume = volume.clamp(0, 64) as u8;

                        let sample = &self.track.samples[sample_id as usize];
                        channel.properties.volume = ((channel.note_volume as u32 * sample.global_volume as u32 * 64 * self.track.global_volume as u32) >> 18) as f64 / 128.0 * (self.track.mix_volume as f64 / u8::MAX as f64);
                        self.system.set_channel_properties(c, channel.properties).unwrap();
                    },
                    /*Effect::PortamentoDown => todo!(),
                    Effect::PortamentoUp => todo!(),
                    Effect::TonePortamento => todo!(),
                    Effect::Vibrato => todo!(),
                    Effect::Tremor => todo!(),
                    Effect::Arpeggio => todo!(),
                    Effect::VolumeSlideVibrato => todo!(),
                    Effect::VolumeSlideTonePortamento => todo!(),
                    Effect::SetChannelVolume => todo!(),
                    Effect::ChannelVolumeSlide => todo!(),
                    Effect::SampleOffset => todo!(),
                    Effect::PanningSlide => todo!(),
                    Effect::Retrigger => todo!(),
                    Effect::Tremolo => todo!(),
                    Effect::Special => todo!(),
                    Effect::Tempo => todo!(),
                    Effect::FineVibrato => todo!(),
                    Effect::SetGlobalVolume => todo!(),
                    Effect::GlobalVolumeSlide => todo!(),
                    Effect::SetPanning => todo!(),
                    Effect::Panbrello => todo!(),
                    Effect::MidiMacro => todo!(),*/
                    _ => {}
                }
            }
        }

        self.current_half_sample += 1;

        if self.current_half_sample >= self.half_samples_per_tick {
            self.current_tick += 1;
            self.current_half_sample = 0;

            if self.current_tick >= self.current_speed {
                self.current_tick = 0;
                self.current_row += 1;     
                
                if self.should_jump {
                    self.should_jump = false;
                    self.current_row = self.next_row;
                    self.current_order = self.next_order;
                }

                if self.current_row >= pattern.rows as usize {
                    self.current_row = 0;
                    self.current_order += 1;

                    if self.current_order >= self.track.orders.len() || self.track.orders[self.current_order] == 255 {
                        self.current_order = 0;
                    }
                }
            }
        }

        self.system.advance()
    }
}

pub fn calculate_half_samples_per_tick(tempo: u8) -> u32 {
    ((2.5 / tempo as f64) * 2.0 * SAMPLE_RATE as f64) as u32
}

pub fn calculate_speed(key: PianoKey, octave: u8, multiplier: f64) -> f64 {
    if key == PianoKey::NoteCut {
        return 0.0;
    }

    // 40 is middle C. Therefore, to work out which note corresponds to the given piano key + octace, we first
    // convert the key to int, subtract the value of middle C (as it is not 0 in the enum), and then add on our octave,
    // multiplied by 12, as that is how many keys are in one octave. We subtract it by 5 as our "middle c" octave is 5.
    let note = 40 + (key as i32 - PianoKey::C as i32) + ((octave as i32 - 5) * 12);

    let pow_note = f64::powf(2.0, (note as f64 - 49.0) / 12.0);

    pow_note * multiplier
}