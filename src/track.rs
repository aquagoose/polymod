use mixr::AudioFormat;

use super::{Arr2D, Note, sample::Sample};
use std::io;

pub struct Pattern {
    pub notes: Arr2D<Note>,
    pub channels: u16,
    pub rows: u16
}

impl Pattern {
    pub fn new(channels: u16, rows: u16) -> Self {
        Self { notes: Arr2D::new(channels as usize, rows as usize), channels, rows }
    }

    pub fn set_note(&mut self, channel: u16, row: u16, note: Note) {
        self.notes.set(channel as usize, row as usize, note);
    }
}

pub struct Track {
    pub patterns: Vec<Pattern>,
    pub orders: Vec<u8>,
    pub samples: Vec<Sample>,

    pub tempo: u8,
    pub speed: u8
}

impl Track {
    pub fn from_it(path: &str) -> Result<(), io::Error> {
        let mut reader = mixr::binary_reader::BinaryReader::new(path)?;
        if reader.read_string(4) != String::from("IMPM") {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected \"IMPM\", not found."));
        }

        let title = reader.read_string(26);
        println!("Loading \"{}\"...", title);

        reader.read_bytes(2); // pattern highlight
        
        let num_orders = reader.read_u16();
        let num_instruments = reader.read_u16();
        let num_samples = reader.read_u16();
        let num_patterns = reader.read_u16();

        reader.read_bytes(4); // created with tracker, not needed here.

        let flags = reader.read_u16();
        if (flags & 4) == 4 {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Instruments are not currently supported."));
        }

        reader.read_bytes(2); // special, not needed.

        let global_volume = reader.read_u8();
        let mix_volume = reader.read_u8();
        let initial_speed = reader.read_u8();
        let initial_tempo = reader.read_u8();

        println!("gv: {global_volume}, mv: {mix_volume}, spd: {initial_speed}, tmp: {initial_tempo}");

        reader.read_bytes(12); // stuff we don't need.

        let pans = reader.read_bytes(64);
        let vols = reader.read_bytes(64);

        assert_eq!(reader.position, 0xC0);

        let orders = reader.read_bytes(num_orders as usize);

        reader.position = (0xC0 + num_orders + num_instruments * 4) as usize;
        
        let mut samples = Vec::with_capacity(num_samples as usize);

        for _ in 0..num_samples {
            let offset = reader.read_u32();
            let curr_pos = reader.position;

            reader.position = offset as usize;

            if reader.read_string(4) != String::from("IMPS") {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected \"IMPS\", not found."));
            }

            let dos_name = reader.read_string(12);
            reader.read_u8(); // seemingly unused byte.

            let s_global = reader.read_u8();
            let s_flags = reader.read_u8();

            let mut format = AudioFormat::default();
            format.bits_per_sample = if (s_flags & 2) == 2 { 16 } else { 8 };
            format.channels = if (s_flags & 4) == 4 { 2 } else { 1 };
            // todo, loops and stuff

            reader.read_u8(); // default volume, not needed for playback.

            let s_name = reader.read_string(26);
            println!("Loading {s_name} ({dos_name})...");

            let s_cvt = reader.read_u8(); // convert, unused *yet* but will be later.
            reader.read_u8(); // default pan, don't think it needs to be used.

            let s_length = reader.read_u32();
            let s_loop_start = reader.read_u32();
            let s_loop_end = reader.read_u32();
            format.sample_rate = reader.read_i32();

            reader.read_bytes(8); // ignoring sustain stuff for now

            let pointer = reader.read_u32();

            reader.position = pointer as usize;
            let s_data = reader.read_bytes(s_length as usize);

            samples.push(Sample::new(s_data, format));

            reader.position = curr_pos;
        }

        Ok(())
    }
}