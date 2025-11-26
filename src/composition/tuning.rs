use crate::composition::music_theory::MidiNote;

/// Guitar tunings for metal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuitarTuning {
    EStandard,      // E2 (MIDI 40) - Thrash, Heavy Metal, Power Metal
    DropD,          // D2 (MIDI 38) - Metalcore, Alt-Metal, Nu-Metal
    DStandard,      // D2 (MIDI 38) - Death Metal
    CStandard,      // C2 (MIDI 36) - Stoner Doom, Melodic Death
    DropC,          // C2 (MIDI 36) - Metalcore
    BStandard7,     // B1 (MIDI 35) - 7-string Deathcore, Death Metal
    DropA7,         // A1 (MIDI 33) - 7-string Deathcore, Djent
    FSharpStandard8,// F#1 (MIDI 30) - 8-string Djent, Prog Metal
    DropE8,         // E1 (MIDI 28) - 8-string Extreme Djent/Thall
}

impl GuitarTuning {
    /// Get the lowest note (MIDI number) for this tuning
    pub fn lowest_note(&self) -> MidiNote {
        match self {
            GuitarTuning::EStandard => 40,      // E2
            GuitarTuning::DropD => 38,          // D2
            GuitarTuning::DStandard => 38,      // D2
            GuitarTuning::CStandard => 36,      // C2
            GuitarTuning::DropC => 36,          // C2
            GuitarTuning::BStandard7 => 35,     // B1
            GuitarTuning::DropA7 => 33,         // A1
            GuitarTuning::FSharpStandard8 => 30,// F#1
            GuitarTuning::DropE8 => 28,         // E1
        }
    }

    /// Get the frequency in Hz of the lowest string
    pub fn frequency(&self) -> f32 {
        crate::composition::music_theory::midi_to_freq(self.lowest_note())
    }

    /// Get all string notes for this tuning (low to high)
    pub fn string_notes(&self) -> Vec<MidiNote> {
        match self {
            GuitarTuning::EStandard => vec![40, 45, 50, 55, 59, 64],      // E A D G B E
            GuitarTuning::DropD => vec![38, 45, 50, 55, 59, 64],          // D A D G B E
            GuitarTuning::DStandard => vec![38, 43, 48, 53, 57, 62],      // D G C F A D
            GuitarTuning::CStandard => vec![36, 41, 46, 51, 55, 60],      // C F Bb Eb G C
            GuitarTuning::DropC => vec![36, 43, 48, 53, 57, 62],          // C G C F A D
            GuitarTuning::BStandard7 => vec![35, 40, 45, 50, 55, 59, 64], // B E A D G B E
            GuitarTuning::DropA7 => vec![33, 40, 45, 50, 55, 59, 64],     // A E A D G B E
            GuitarTuning::FSharpStandard8 => vec![30, 35, 40, 45, 50, 55, 59, 64], // F# B E A D G B E
            GuitarTuning::DropE8 => vec![28, 35, 40, 45, 50, 55, 59, 64], // E B E A D G B E
        }
    }

    /// Check if bass should play in unison mode (tuning too low for octave down)
    pub fn bass_should_use_unison(&self) -> bool {
        // If lowest note is below B1 (MIDI 35), bass octave down would be too low
        self.lowest_note() < 35
    }

    /// Get the recommended bass offset in semitones
    pub fn bass_offset(&self) -> i8 {
        if self.bass_should_use_unison() {
            0  // Unison mode
        } else {
            -12  // Octave down
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuning_frequencies() {
        let drop_c = GuitarTuning::DropC;
        assert_eq!(drop_c.lowest_note(), 36);
        assert!((drop_c.frequency() - 65.41).abs() < 0.1);
    }

    #[test]
    fn test_bass_unison_mode() {
        assert!(!GuitarTuning::DropC.bass_should_use_unison());
        assert!(GuitarTuning::DropA7.bass_should_use_unison());
        assert!(GuitarTuning::DropE8.bass_should_use_unison());
    }

    #[test]
    fn test_string_notes() {
        let e_standard = GuitarTuning::EStandard;
        let strings = e_standard.string_notes();
        assert_eq!(strings.len(), 6);
        assert_eq!(strings[0], 40); // Low E
    }
}
