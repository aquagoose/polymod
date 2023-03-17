use crate::Effect;

pub fn get_effect(it_effect: u8, param: u8) -> Effect {
    match it_effect {
        0 => Effect::None,
        1 => Effect::SetSpeed(param),
        2 => Effect::PositionJump(param),
        3 => Effect::PatternBreak(param),
        4 => Effect::VolumeSlide(param),
        5 => Effect::PortamentoDown(param),
        6 => Effect::PortamentoUp(param),
        7 => Effect::TonePortamento(param),
        8 => Effect::Vibrato(param),
        9 => Effect::Tremor(param),
        10 => Effect::Arpeggio(param),
        11 => Effect::VolumeSlideVibrato(param),
        12 => Effect::VolumeSlideTonePortamento(param),
        13 => Effect::SetChannelVolume(param),
        14 => Effect::ChannelVolumeSlide(param),
        15 => Effect::SampleOffset(param),
        16 => Effect::PanningSlide(param),
        17 => Effect::Retrigger(param),
        18 => Effect::Tremolo(param),
        19 => Effect::Special(param),
        20 => Effect::Tempo(param),
        21 => Effect::FineVibrato(param),
        22 => Effect::SetGlobalVolume(param),
        23 => Effect::GlobalVolumeSlide(param),
        24 => Effect::SetPanning(param),
        25 => Effect::Panbrello(param),
        26 => Effect::MidiMacro(param),
        _ => Effect::None
    }
}