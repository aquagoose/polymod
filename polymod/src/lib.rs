pub mod track;
pub mod sample;
pub mod track_player;
pub mod utils;

pub enum ModuleType {
    PMM,
    IT,
    XM,
    S3M,
    MOD
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    None,

    SetSpeed(u8), // Axx
    PositionJump(u8), // Bxx
    PatternBreak(u8), // Cxx
    VolumeSlide(u8), // Dxx
    PortamentoDown(u8), // Exx
    PortamentoUp(u8), // Fxx
    TonePortamento(u8), // Gxx
    Vibrato(u8), // Hxx
    Tremor(u8), // Ixx
    Arpeggio(u8), // Jxx
    VolumeSlideVibrato(u8), // Kxx
    VolumeSlideTonePortamento(u8), // Lxx
    SetChannelVolume(u8), // Mxx
    ChannelVolumeSlide(u8), // Nxx
    SampleOffset(u8), // Oxx
    PanningSlide(u8), // Pxx
    Retrigger(u8), // Qxx
    Tremolo(u8), // Rxx
    Special(u8), // Sxx (this one contains like 20 other commands inside it)
    Tempo(u8), // Txx
    FineVibrato(u8), // Uxx
    SetGlobalVolume(u8), // Vxx
    GlobalVolumeSlide(u8), // Wxx
    SetPanning(u8), // Xxx
    Panbrello(u8), // Yxx
    MidiMacro(u8) // Zxx
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub initialized: bool,

    pub key: PianoKey,
    pub octave: u8,

    pub sample: Option<u8>,
    pub volume: Option<u8>,
    pub effect: Effect
}

impl Default for Note {
    fn default() -> Self {
        Self { initialized: false, key: PianoKey::None, octave: 0, sample: None, volume: None, effect: Effect::None }
    }
}

impl Note {
    pub fn new(key: PianoKey, octave: u8, sample: Option<u8>, volume: Option<u8>, effect: Effect) -> Self {
        Self {
            initialized: true,
            key,
            octave,
            sample,
            volume,
            effect
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

#[inline(always)]
#[cfg(debug_assertions)]
pub fn log(text: String) {
    println!("{text}");
}

#[inline(always)]
#[cfg(not(debug_assertions))]
pub fn log(text: String) {}