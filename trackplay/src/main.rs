use std::time::Duration;

use polymod::{self, track::{Track}, track_player::{TrackPlayer}};
use sdl2::audio::{AudioSpecDesired, AudioCallback};
use clap::Parser;

#[derive(Parser)]
struct Args {
    path: String,

    #[arg(long, default_value_t = 1.0)]
    pitch_tuning: f64,

    #[arg(long, default_value_t = 1.0)]
    tempo_tuning: f64,

    #[arg(long, default_value_t = 0.0)]
    start: f64,

    #[arg(long, default_value_t = false)]
    no_interpolation: bool
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
    let pitch_tuning = args.pitch_tuning;
    let tempo_tuning = args.tempo_tuning;
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