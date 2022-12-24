use std::time::Duration;

use polymod::{self, track::{Track, Pattern}, track_player::{TrackPlayer}, Note, PianoKey};
use sdl2::audio::{AudioSpecDesired, AudioCallback};

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
    let path = std::env::args().nth(1).unwrap();
    let path = &path[..];

    let track = Track::from_it("/home/ollie/Music/Modules/Created/track 1.it").unwrap();

    let mut player = TrackPlayer::new(&track);
    
    let sdl = sdl2::init().unwrap();
    let audio = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(polymod::track_player::SAMPLE_RATE),
        channels: Some(2),
        samples: Some(8192)
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