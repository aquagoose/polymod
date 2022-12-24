use polymod::{self, track::{Track}};

#[test]
pub fn test_load_track() {
    let track = Track::from_it("/home/ollie/Music/Modules/Created/track 1.it").unwrap();
}