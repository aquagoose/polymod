use std::time::Duration;

use polymod::{self, track::{Pattern, Track}, Note, sample::Sample, track_player::TrackPlayer};
use sdl2::audio::{AudioSpecDesired, AudioCallback};

#[test]
fn test_notes() {
    let mut pattern = Pattern::new(64, 64);
    pattern.set_note(0, 0, Note { initialized: true, key: polymod::PianoKey::C, octave: 5, sample: 0, volume: 64, effect: polymod::Effect::None, effect_param: 0 });
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

#[test]
fn test_track_player() {
    let pcm = mixr::loaders::PCM::load_wav("/home/ollie/Music/Samples/piano_middlec.wav").unwrap();
    let sample = Sample::new(&pcm.data, pcm.format, false, 64);
    
    let mut pattern = Pattern::new(64, 4);
    pattern.set_note(0, 0, Note::new(polymod::PianoKey::C, 5, 0, 64, polymod::Effect::None, 0));
    pattern.set_note(1, 1, Note::new(polymod::PianoKey::E, 5, 0, 64, polymod::Effect::None, 0));
    pattern.set_note(2, 2, Note::new(polymod::PianoKey::G, 5, 0, 64, polymod::Effect::None, 0));
    pattern.set_note(3, 3, Note::new(polymod::PianoKey::C, 6, 0, 64, polymod::Effect::None, 0));

    let track = Track { patterns: vec![pattern], orders: vec![0], samples: vec![sample], tempo: 125, speed: 6 };
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

    //std::thread::sleep(Duration::from_secs((((length as i32) / 4 / rate) - 1) as u64));
    loop {
        std::thread::sleep(Duration::from_secs(5));
    }
}