use super::{Arr2D, Note, sample::Sample};

pub struct Pattern {
    notes: Arr2D<Note>
}

impl Pattern {
    pub fn new(channels: u16, rows: u16) -> Self {
        Self { notes: Arr2D::new(channels as usize, rows as usize) }
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