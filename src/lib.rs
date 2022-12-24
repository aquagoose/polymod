pub mod track;
pub mod sample;
pub mod track_player;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PianoKey {
    None,
    NoteCut,
    NoteOff,
    NoteFade,

    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B
}

#[derive(Debug)]
pub enum Effect {
    None
}

#[derive(Debug)]
pub struct Note {
    pub initialized: bool,

    pub key: PianoKey,
    pub octave: u8,

    pub sample: u8,
    pub volume: u8,
    pub effect: Effect,
    pub effect_param: u8
}

impl Default for Note {
    fn default() -> Self {
        Self { initialized: false, key: PianoKey::None, octave: 0, sample: 0, volume: 0, effect: Effect::None, effect_param: 0 }
    }
}

impl Note {
    pub fn new(key: PianoKey, octave: u8, sample: u8, volume: u8, effect: Effect, effect_param: u8) -> Self {
        Self {
            initialized: true,
            key,
            octave,
            sample,
            volume,
            effect,
            effect_param
        }
    }
}

pub struct Arr2D<T: Default> {
    vec: Vec<T>,
    columns: usize,
    rows: usize
}

impl<T: Default> Arr2D<T> {
    pub fn new(columns: usize, rows: usize) -> Self {
        let mut vec = Vec::with_capacity(columns * rows);
        for _ in 0..(columns * rows) {
            vec.push(T::default());
        }

        Self { vec, columns, rows }
    }

    pub fn set(&mut self, column: usize, row: usize, value: T) {
        self.vec[row * self.columns + column] = value;
    }

    pub fn get(&self, column: usize, row: usize) -> &T {
        &self.vec[row * self.columns + column]
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
}