use polymod::{self, track::Pattern, Note};

#[test]
fn test_notes() {
    let mut pattern = Pattern::new(64, 64);
    pattern.set_note(0, 0, Note { key: polymod::PianoKey::C, octave: 5, volume: 64, effect: polymod::Effect::None, effect_param: 0 });
}

#[test]
fn test_track_player() {
    
}