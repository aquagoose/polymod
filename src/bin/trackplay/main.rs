use std::time::Duration;

use polymod::{self, track::{Track}, track_player::{TrackPlayer}};
use sdl2::audio::{AudioSpecDesired, AudioCallback};
use clap::Parser;

#[derive(Parser)]
struct Args {
    path: String,

    #[arg(short, long, default_value_t = 1.0)]
    tuning: f64
}

struct Audio<'a> {
    player: &'a mut TrackPlayer<'a>
}

impl<'a> AudioCallback for Audio<'a> {
    type Channel = i16;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = self.player.advance();
        }
    }
}

fn main() {
    let args = Args::parse();
    let path = args.path.as_str();
    let tuning = args.tuning;

    let track = Track::from_it(path);
    if let Some(err) = track.as_ref().err() {
        if err.kind() == std::io::ErrorKind::NotFound {
            println!("The path \"{path}\" was not found.");
            std::process::exit(1);
        }
    }

    let track = track.unwrap();

    let mut player = TrackPlayer::new(&track);
    player.tuning = tuning;
    
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