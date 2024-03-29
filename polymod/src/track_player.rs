use mixr::{ChannelProperties, BufferDescription, DataType, AudioFormat};

use crate::{track::Track, PianoKey, Effect, sample::Sample, Note};

pub const SAMPLE_RATE: i32 = 48000;

struct TrackChannel {
    properties: ChannelProperties,
    enabled: bool,

    current_sample: Option<u8>,
    note_volume: u8,

    vol_memory: u8,
    pitch_memory: u8,

    offset_memory: u8,
    high_offset: usize
}

pub struct TrackPlayer<'a> {
    track: &'a Track,
    system: mixr::system::AudioSystem,
    buffers: Vec<i32>,

    current_half_sample: u32,
    half_samples_per_tick: u32,
    current_tick: u8,
    current_speed: u8,
    current_tempo: u8,

    current_order: usize,
    current_row: usize,

    next_row: usize,
    next_order: usize,

    should_jump: bool,

    channels: Vec<TrackChannel>,

    pitch_tuning: f64,
    tempo_tuning: f64,

    global_volume: u8,

    pub looping: bool
}

impl<'a> TrackPlayer<'a> {
    pub fn new(track: &'a Track) -> Self {
        let mut system = mixr::system::AudioSystem::new(SAMPLE_RATE,64);
        
        let mut buffers = Vec::with_capacity(track.samples.len());
        for i in 0..track.samples.len() {
            let sample = &track.samples[i];
            let buffer = system.create_buffer(BufferDescription { data_type: DataType::Pcm, format: sample.format }, Some(&sample.data));
            buffers.push(buffer);
        }

        let mut channels = Vec::with_capacity(system.num_channels() as usize);
        for i in 0..system.num_channels() {
            let mut properties = ChannelProperties::default();
            properties.interpolation = mixr::InterpolationType::Linear;

            let pan = track.pans[i as usize];
            properties.panning = pan as f64 / 64.0;
            // A pan value of >= 128 means the channel is disabled and will not be played.
            channels.push(TrackChannel {
                properties,
                enabled: pan < 128,
                current_sample: None,
                note_volume: 0,

                vol_memory: 0,
                pitch_memory: 0,

                offset_memory: 0,
                high_offset: 0
            });
        }

        let half_samples_per_tick = calculate_half_samples_per_tick(track.tempo);
        let speed = track.speed;
        let tempo = track.tempo;

        Self { 
            track, 
            system,
            buffers,

            current_half_sample: 0,
            half_samples_per_tick,
            current_tick: 0,
            current_speed: speed,
            current_tempo: tempo,

            current_order: 0,
            current_row: 0,
            
            next_row: 0,
            next_order: 0,

            should_jump: false,

            channels,

            looping: true,
            pitch_tuning: 1.0,
            tempo_tuning: 1.0,

            global_volume: track.global_volume
        }
    }

    pub fn advance(&mut self) -> f64 {
        let pattern = &self.track.patterns[self.track.orders[self.current_order] as usize];

        if self.current_half_sample == 0 {
            for c in 0..pattern.channels {
                let mut channel = &mut self.channels[c as usize];

                if !channel.enabled {
                    continue;
                }

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
                            let volume = note.volume.unwrap_or(sample.default_volume);
                            properties.volume = ((volume as u32 * sample.global_volume as u32 * 64 * self.global_volume as u32) >> 18) as f64 / 128.0 * (self.track.mix_volume as f64 / u8::MAX as f64);
                            properties.speed = calculate_speed(note.key, note.octave, sample.multiplier) * self.pitch_tuning;
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
                        channel.properties.volume = ((volume as u32 * sample.global_volume as u32 * 64 * self.global_volume as u32) >> 18) as f64 / 128.0 * (self.track.mix_volume as f64 / u8::MAX as f64);
                        self.system.set_channel_properties(c, channel.properties).unwrap();
                        channel.note_volume = volume;
                    }
                }

                match note.effect {
                    Effect::None => {},
                    Effect::SetSpeed(speed) => if self.current_tick == 0 { self.current_speed = speed },
                    Effect::PositionJump(pos) => {
                        self.next_row = 0;
                        self.next_order = pos as usize;
                        self.should_jump = true;
                    },
                    Effect::PatternBreak(pos) => {
                        self.next_order = self.current_order + 1;
                        self.next_row = pos as usize;
                        self.should_jump = true;
                    },
                    Effect::VolumeSlide(value) => {
                        // If the note parameter is 0, we just fetch the last one stored in memory.
                        // If the last parameter is also 0 then nothing happens.
                        let mut vol_param = if value == 0 { channel.vol_memory } else { value };
                        channel.vol_memory = vol_param;

                        // Handle DFy and DxF, if 'F' is set then the volume slide only occurs on the first tick.
                        // However, if value is D0F, then ignore, as this is not a fine volume slide.
                        // Volume slide occurs on every tick except the first, **unless** it is D0F.
                        if channel.current_sample.is_none() || (self.current_tick == 0 && ((vol_param & 0xF0) != 0xF0 && (vol_param & 0xF) != 0xF)) ||
                            (((vol_param & 0xF0) == 0xF0 || ((vol_param & 0xF) == 0xF && (vol_param & 0xF0) != 0)) && self.current_tick != 0) {
                            continue;
                        }

                        let sample_id = channel.current_sample.unwrap();

                        let mut volume = channel.note_volume as i32;

                        // If the volume parameter is DFx then we need to remove the F so that the volume slide
                        // works as usual, otherwise it would think it's a value of 240 + x
                        if (vol_param & 0xF0) == 0xF0 {
                            vol_param = vol_param & 0x0F
                        }

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
                        channel.properties.volume = ((channel.note_volume as u32 * sample.global_volume as u32 * 64 * self.global_volume as u32) >> 18) as f64 / 128.0 * (self.track.mix_volume as f64 / u8::MAX as f64);
                        self.system.set_channel_properties(c, channel.properties).unwrap();
                    },
                    Effect::PortamentoDown(value) => {
                        let mut pitch_param = if value == 0 { channel.pitch_memory } else { value };
                        channel.pitch_memory = pitch_param;

                        if ((pitch_param & 0xF0) >= 0xE0 && self.current_tick != 0) || self.current_tick == 0 && (pitch_param & 0xF0) < 0xE0 {
                            continue;
                        }

                        let multiplier = if (pitch_param & 0xF0) == 0xE0 { 1.0 / 4.0 } else { 1.0 };

                        if (pitch_param & 0xF0) == 0xF0 {
                            pitch_param &= 0xF;
                        } else if (pitch_param & 0xF0) == 0xE0 {
                            pitch_param &= 0xF;
                        }

                        channel.properties.speed *= f64::powf(2.0, -4.0 * (pitch_param as f64 * multiplier) / 768.0);
                        self.system.set_channel_properties(c, channel.properties).unwrap();
                    },
                    Effect::PortamentoUp(value) => {
                        let mut pitch_param = if value == 0 { channel.pitch_memory } else { value };
                        channel.pitch_memory = pitch_param;

                        if ((pitch_param & 0xF0) >= 0xE0 && self.current_tick != 0) || self.current_tick == 0 && (pitch_param & 0xF0) < 0xE0 {
                            continue;
                        }

                        let multiplier = if (pitch_param & 0xF0) == 0xE0 { 1.0 / 4.0 } else { 1.0 };

                        if (pitch_param & 0xF0) == 0xF0 {
                            pitch_param &= 0xF;
                        } else if (pitch_param & 0xF0) == 0xE0 {
                            pitch_param &= 0xF;
                        }

                        channel.properties.speed *= f64::powf(2.0, 4.0 * (pitch_param as f64 * multiplier) / 768.0);
                        self.system.set_channel_properties(c, channel.properties).unwrap();
                    },
                    /*Effect::TonePortamento => todo!(),
                    Effect::Vibrato => todo!(),
                    Effect::Tremor => todo!(),
                    Effect::Arpeggio => todo!(),
                    Effect::VolumeSlideVibrato => todo!(),
                    Effect::VolumeSlideTonePortamento => todo!(),
                    Effect::SetChannelVolume => todo!(),
                    Effect::ChannelVolumeSlide => todo!(),*/
                    Effect::SampleOffset(offset) => {
                        if self.current_tick == 0 {
                            let offset = if offset == 0 { channel.offset_memory } else { offset };
                            channel.offset_memory = offset;

                            if note.key != PianoKey::None {
                                let _ = self.system.seek_to_sample(c, offset as usize * 256 + channel.high_offset);
                            }
                        }
                    },
                    /*Effect::PanningSlide => todo!(),
                    Effect::Retrigger => todo!(),
                    Effect::Tremolo => todo!(),*/
                    Effect::Special(cmd) => {
                        if cmd >= 0x80 && cmd <= 0x8F {
                            channel.properties.panning = (cmd & 0xF) as f64 / 15.0;
                            self.system.set_channel_properties(c, channel.properties).unwrap();
                        }

                        if cmd >= 0xA0 && cmd <= 0xAF {
                            channel.high_offset = (cmd & 0xF) as usize * 65536;
                        }
                    },
                    Effect::Tempo(tempo) => {
                        // TODO: Tempo slides
                        if tempo > 0x20 && self.current_tick == 0 {
                            self.set_tempo(tempo);
                        }
                    },
                    //Effect::FineVibrato => todo!(),
                    Effect::SetGlobalVolume(vol) => {
                        // TODO: This has weird behaviour right now. When global volume is adjusted - all sample volumes must be
                        // adjusted too. Currently, this only affects new samples that are played.
                        self.global_volume = vol;
                    },
                    //Effect::GlobalVolumeSlide => todo!(),
                    Effect::SetPanning(pan) => {
                        channel.properties.panning = pan as f64 / 255.0;
                        self.system.set_channel_properties(c, channel.properties).unwrap();
                    },
                    /*Effect::Panbrello => todo!(),
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
                        if self.looping {
                            self.current_order = 0;
                        }
                        else {
                            return 0.0;
                        }
                    }
                }

                //println!("Ord {}/{} Row {}/{} Spd {}, HSPT {} (Tmp {}, SR {})", self.current_order + 1, self.track.orders.len(), self.current_row, pattern.rows, self.current_speed, self.half_samples_per_tick, self.current_tempo, SAMPLE_RATE);
            }
        }

        self.system.advance()
    }

    pub fn set_interpolation(&mut self, interp_type: mixr::InterpolationType) {
        for channel in self.channels.iter_mut() {
            channel.properties.interpolation = interp_type;
        }
    }

    pub fn set_pitch_tuning(&mut self, tuning: f64) {
        self.pitch_tuning = tuning;
    }

    pub fn set_tempo_tuning(&mut self, tuning: f64) {
        self.tempo_tuning = tuning;
        self.set_tempo(self.current_tempo);
    }

    pub fn seek_seconds(&mut self, seconds: f64) -> f64 {
        for i in 0..self.track.seek_table.len() {
            let table = &self.track.seek_table[i];
            if table.start > seconds {
                let table = &self.track.seek_table[i - 1];
                for j in 0..table.rows.len() {
                    let row = &table.rows[j];
                    if row.start > seconds {
                        self.current_tick = 0;
                        self.current_half_sample = 0;
                        self.current_order = i - if i == 0 { 0 } else { 1 };
                        
                        self.current_row = j - if j == 0 { 0 } else { 1 };

                        self.current_speed = row.speed;
                        self.set_tempo(row.tempo);

                        return row.start;
                    }
                }
            }
        }

        0.0
    }

    fn set_tempo(&mut self, tempo: u8) {
        self.current_tempo = tempo;
        self.half_samples_per_tick = (calculate_half_samples_per_tick(tempo) as f64 * (1.0 / self.tempo_tuning)) as u32;
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