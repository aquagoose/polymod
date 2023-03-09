use std::time::Duration;

use polymod::{self, track::{Track}, track_player::{TrackPlayer}};
use sdl2::audio::{AudioSpecDesired, AudioCallback};
use clap::Parser;

use crate::binary::BinaryWriter;

mod binary;

#[derive(Parser)]
struct Args {
    /// Path to the file.
    path: String,

    /// The pitch tuning, where 1.0 is no change.
    #[arg(long, default_value_t = 1.0)]
    pitch: f64,

    /// The tempo tuning, where 1.0 is no change.
    #[arg(long, default_value_t = 1.0)]
    tempo: f64,

    /// The start offset in seconds.
    #[arg(long, default_value_t = 0.0)]
    start: f64,

    /// Disable interpolation.
    #[arg(long, default_value_t = false)]
    no_interpolation: bool,

    /// If set, the output will be redirected to the given file.
    #[arg(long)]
    render: Option<String>
}

struct Audio<'a> {
    player: &'a mut TrackPlayer<'a>
}

impl<'a> AudioCallback for Audio<'a> {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = self.player.advance() as f32;
        }
    }
}

fn main() {
    let args = Args::parse();
    let path = args.path.as_str();
    let pitch_tuning = args.pitch;
    let tempo_tuning = args.tempo;
    let start = args.start;

    let track = Track::from_it(&std::fs::read(path).unwrap());
    if let Some(err) = track.as_ref().err() {
        if err.kind() == std::io::ErrorKind::NotFound {
            println!("The path \"{path}\" was not found.");
            std::process::exit(1);
        }
    }

    let track = track.unwrap();

    let mut player = TrackPlayer::new(&track);
    player.set_pitch_tuning(pitch_tuning);
    player.set_tempo_tuning(tempo_tuning);
    player.set_interpolation(if args.no_interpolation { mixr::InterpolationType::None } else { mixr::InterpolationType::Linear });

    player.seek_seconds(start);

    if let Some(render) = args.render {
        println!("Saving to {render}...");

        let mut writer = BinaryWriter::new();

        // "RIFF"
        writer.write_u32(0x46464952);

        // File size, we'll go back and add this in later.
        writer.write_u32(0x0);

        // "WAVE"
        writer.write_u32(0x45564157);

        // "fmt "
        writer.write_u32(0x20746D66);

        // Subchunk1 size, 16 for PCM in this case.
        writer.write_u32(16);

        // Format of 3, as floating point audio. 1 would be non floating point.
        writer.write_u16(3);

        // 2 as stereo.
        writer.write_u16(2);

        // Sampling rate.
        writer.write_u32(polymod::track_player::SAMPLE_RATE as u32);

        // Byte rate = sample rate * channels(2) * bits_per_sample(32) / 8
        writer.write_u32(polymod::track_player::SAMPLE_RATE as u32 * 8);

        // Block align = channels(2) * bits_per_sample(32) / 8
        writer.write_u16(8);

        // Bits per sample.
        writer.write_u16(32);

        // "data"
        writer.write_u32(0x61746164);

        // Multiply this by 2 as we're in stereo.
        let length_in_samples = (track.length_in_seconds * (1.0 / tempo_tuning) * polymod::track_player::SAMPLE_RATE as f64) as usize * 2;

        let mut output = Vec::with_capacity(length_in_samples * 4);

        println!("Rendering track...");

        for i in 0..length_in_samples {
            let samples = (player.advance() as f32).to_le_bytes();

            output.push(samples[0]);
            output.push(samples[1]);
            output.push(samples[2]);
            output.push(samples[3]);

            if i % 500000 == 0 {
                println!("{i} / {length_in_samples} ({:.2}%)", (i as f64 / length_in_samples as f64) * 100.0);
            }
        }

        println!("{length_in_samples} / {length_in_samples} (100.00%)");

        println!("Writing result...");

        // Subchunk2 size, in this case the PCM data length in bytes.
        writer.write_u32(output.len() as u32);
        writer.write_bytes(&output);

        // Get current position (which is the number of bytes total), then write this value (-8) to the
        // dummy value we entered earlier.
        let position = writer.position() - 8;
        writer.set_position(4);
        writer.write_u32(position as u32);

        println!("Saving...");
        std::fs::write(render.as_str(), writer.get_data()).unwrap();

        println!("Done!");

        return;
    }
    
    let sdl = sdl2::init().unwrap();
    let audio = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(polymod::track_player::SAMPLE_RATE),
        channels: Some(2),
        samples: Some(512)
    };

    let device = audio.open_playback(None, &desired_spec, |_| {
        Audio {
            player: &mut player
        }
    }).unwrap();

    device.resume();

    ctrlc::set_handler(move || { std::process::exit(0) }).unwrap();

    loop {
        std::thread::sleep(Duration::from_secs(5));
    }
}