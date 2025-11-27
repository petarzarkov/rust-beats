use rand::Rng;

/// Musical note represented as MIDI number (C4 = 60)
pub type MidiNote = u8;

/// Represents a musical key
#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub root: MidiNote,
    pub scale_type: ScaleType,
}

/// Types of musical scales
#[derive(Debug, Clone, Copy)]
pub enum ScaleType {
    Major,
    Minor,
    Dorian,
    Mixolydian,
    Phrygian,         // Dark, Spanish flavor - METAL CORE
    Lydian,           // Dreamy, jazzy
    MinorPentatonic,
    MajorPentatonic,
    Blues,
    HarmonicMinor,    // Dramatic, classical
    MelodicMinor,     // Jazz, sophisticated
    PhrygianDominant, // Exotic, Middle Eastern - METAL TECH
    WholeTone,        // Dreamy, surreal
    Diminished,       // Tense, jazzy
    Locrian,          // Very dark, unstable - METAL EXTREME
    DoubleHarmonicMajor, // Byzantine scale - METAL DJENT/PROG
}

/// Represents a chord
#[derive(Debug, Clone)]
pub struct Chord {
    pub root: MidiNote,
    pub chord_type: ChordType,
}

/// Types of chords
#[derive(Debug, Clone, Copy)]
pub enum ChordType {
    Major,
    Minor,
    Dominant7,
    Minor7,
    Major7,
    Diminished,
    Sus4,
    // Extended jazz chords for lofi
    Major9,
    Minor9,
    Dominant9,
    Major11,
    Minor11,
    Major13,
    Dominant13,
    HalfDiminished7, // m7b5
    MinorMajor7,
    // Additional chord varieties for more harmonic richness
    Sus2,
    Augmented,
    Add9,
    Sixth,          // Major 6
    Minor6,         // Minor 6
    Dominant7Sharp9,// Hendrix chord
    Dominant7Flat9, // Altered dominant
    Power5,         // Root + 5th
}

impl ScaleType {
    /// Get the intervals for this scale type (in semitones from root)
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            ScaleType::Major => vec![0, 2, 4, 5, 7, 9, 11],
            ScaleType::Minor => vec![0, 2, 3, 5, 7, 8, 10],
            ScaleType::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            ScaleType::Mixolydian => vec![0, 2, 4, 5, 7, 9, 10],
            ScaleType::Phrygian => vec![0, 1, 3, 5, 7, 8, 10],
            ScaleType::Lydian => vec![0, 2, 4, 6, 7, 9, 11],
            ScaleType::MinorPentatonic => vec![0, 3, 5, 7, 10],
            ScaleType::MajorPentatonic => vec![0, 2, 4, 7, 9],
            ScaleType::Blues => vec![0, 3, 5, 6, 7, 10],
            ScaleType::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleType::MelodicMinor => vec![0, 2, 3, 5, 7, 9, 11],
            ScaleType::PhrygianDominant => vec![0, 1, 4, 5, 7, 8, 10],
            ScaleType::WholeTone => vec![0, 2, 4, 6, 8, 10],
            ScaleType::Diminished => vec![0, 2, 3, 5, 6, 8, 9, 11],
            ScaleType::Locrian => vec![0, 1, 3, 5, 6, 8, 10],
            ScaleType::DoubleHarmonicMajor => vec![0, 1, 4, 5, 7, 8, 11], // 1, b2, 3, 4, 5, b6, 7
        }
    }
}

impl Key {
    /// Create a key from a specific scale type
    pub fn from_scale(scale_type: ScaleType) -> Self {
        let mut rng = rand::thread_rng();
        let roots = vec![36, 38, 40, 41, 43, 45, 46]; // C, D, E, F, G, A, Bb
        Key {
            root: roots[rng.gen_range(0..roots.len())],
            scale_type,
        }
    }

    /// Create a random key suitable for funk/jazz
    pub fn random_funky() -> Self {
        let mut rng = rand::thread_rng();

        // Funky keys often use flats and tend toward minor/dorian
        let roots = vec![
            36, // C
            38, // D
            40, // E
            41, // F
            43, // G
            45, // A
            46, // Bb
        ];

        // Weighted towards happier scales
        let scale_choice = rng.gen_range(0..100);
        let scale_type = if scale_choice < 30 {
            ScaleType::Major // 30% - Bright, happy!
        } else if scale_choice < 50 {
            ScaleType::MajorPentatonic // 20% - Simple, cheerful
        } else if scale_choice < 70 {
            ScaleType::Lydian // 20% - Dreamy, uplifting
        } else if scale_choice < 85 {
            ScaleType::Mixolydian // 15% - Funky, bright
        } else if scale_choice < 93 {
            ScaleType::Dorian // 8% - Jazzy (not dark)
        } else {
            ScaleType::Minor // 7% - Occasional moodiness
        };

        Key {
            root: roots[rng.gen_range(0..roots.len())],
            scale_type,
        }
    }

    /// Get all notes in this key within an octave
    pub fn get_scale_notes(&self) -> Vec<MidiNote> {
        self.scale_type
            .intervals()
            .iter()
            .map(|&interval| self.root + interval)
            .collect()
    }

    /// Get scale notes across multiple octaves
    pub fn get_scale_notes_range(&self, octaves: u8) -> Vec<MidiNote> {
        let mut notes = Vec::new();
        let intervals = self.scale_type.intervals();

        for octave in 0..octaves {
            for &interval in &intervals {
                let note = self.root + (octave * 12) + interval;
                if note < 128 {
                    notes.push(note);
                }
            }
        }
        notes
    }

    /// Calculate interval in semitones between two notes
    pub fn calculate_interval(note_a: MidiNote, note_b: MidiNote) -> u8 {
        (note_a as i16 - note_b as i16).abs() as u8 % 12
    }

    /// Check if an interval is dissonant (minor second or tritone)
    pub fn is_dissonant(note_a: MidiNote, note_b: MidiNote) -> bool {
        let interval = Self::calculate_interval(note_a, note_b);
        interval == 1 || interval == 6  // Minor second (b2) or tritone (b5)
    }

    /// Get dissonance weight for metal riff generation (higher = more dissonant = more metal)
    pub fn get_dissonance_weight(interval: u8) -> f32 {
        match interval {
            1 => 2.0,  // Minor second - very metal
            6 => 1.8,  // Tritone - the devil's interval
            2 => 0.5,  // Major second - less interesting
            3 => 0.3,  // Minor third - consonant
            4 => 0.2,  // Major third - too happy
            5 => 1.0,  // Perfect fourth - good for metal
            7 => 1.2,  // Perfect fifth - power chord foundation
            _ => 0.5,  // Other intervals
        }
    }
}

impl Chord {
    /// Get the notes that make up this chord
    pub fn get_notes(&self) -> Vec<MidiNote> {
        let intervals = match self.chord_type {
            ChordType::Major => vec![0, 4, 7],
            ChordType::Minor => vec![0, 3, 7],
            ChordType::Dominant7 => vec![0, 4, 7, 10],
            ChordType::Minor7 => vec![0, 3, 7, 10],
            ChordType::Major7 => vec![0, 4, 7, 11],
            ChordType::Diminished => vec![0, 3, 6],
            ChordType::Sus4 => vec![0, 5, 7],
            // Extended jazz chords
            ChordType::Major9 => vec![0, 4, 7, 11, 14],
            ChordType::Minor9 => vec![0, 3, 7, 10, 14],
            ChordType::Dominant9 => vec![0, 4, 7, 10, 14],
            ChordType::Major11 => vec![0, 4, 7, 11, 14, 17],
            ChordType::Minor11 => vec![0, 3, 7, 10, 14, 17],
            ChordType::Major13 => vec![0, 4, 7, 11, 14, 21],
            ChordType::Dominant13 => vec![0, 4, 7, 10, 14, 21],
            ChordType::HalfDiminished7 => vec![0, 3, 6, 10], // m7b5
            ChordType::MinorMajor7 => vec![0, 3, 7, 11],
            // Additional chord varieties
            ChordType::Sus2 => vec![0, 2, 7],
            ChordType::Augmented => vec![0, 4, 8],
            ChordType::Add9 => vec![0, 4, 7, 14],
            ChordType::Sixth => vec![0, 4, 7, 9],
            ChordType::Minor6 => vec![0, 3, 7, 9],
            ChordType::Dominant7Sharp9 => vec![0, 4, 7, 10, 15],
            ChordType::Dominant7Flat9 => vec![0, 4, 7, 10, 13],
            ChordType::Power5 => vec![0, 7, 12],
        };

        intervals
            .iter()
            .map(|&interval| self.root + interval)
            .filter(|&note| note < 128)
            .collect()
    }
}

/// Generate a chord progression suitable for funk/jazz
pub fn generate_chord_progression(key: &Key, length: usize) -> Vec<Chord> {
    generate_chord_progression_with_types(key, length, None)
}

/// Generate a chord progression with preferred chord types
pub fn generate_chord_progression_with_types(
    key: &Key,
    length: usize,
    preferred_types: Option<&[ChordType]>,
) -> Vec<Chord> {
    let mut rng = rand::thread_rng();
    let scale_notes = key.get_scale_notes();

    // Helper to select chord type, preferring preferred types if available
    let select_chord_type =
        |rng: &mut rand::rngs::ThreadRng, _degree: usize, default: ChordType| -> ChordType {
            if let Some(preferred) = preferred_types {
                if !preferred.is_empty() {
                    // 70% chance to use preferred type, 30% chance for default
                    if rng.gen_range(0..100) < 70 {
                        preferred[rng.gen_range(0..preferred.len())]
                    } else {
                        default
                    }
                } else {
                    default
                }
            } else {
                default
            }
        };

    // Weighted towards happier, uplifting progressions, now with extended chords
    let choice = rng.gen_range(0..100);
    let pattern = if choice < 20 {
        // I-V-vi-IV (most popular happy progression!) - use extended chords
        vec![
            (0, select_chord_type(&mut rng, 0, ChordType::Major11)),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant13)),
            (5, select_chord_type(&mut rng, 5, ChordType::Minor11)),
            (3, select_chord_type(&mut rng, 3, ChordType::Major9)),
        ]
    } else if choice < 35 {
        // I-IV-V (classic happy progression)
        vec![
            (0, select_chord_type(&mut rng, 0, ChordType::Major7)),
            (3, select_chord_type(&mut rng, 3, ChordType::Major9)),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant7)),
        ]
    } else if choice < 50 {
        // I-vi-IV-V (upbeat pop progression)
        vec![
            (0, select_chord_type(&mut rng, 0, ChordType::Major9)),
            (5, select_chord_type(&mut rng, 5, ChordType::Minor11)),
            (3, select_chord_type(&mut rng, 3, ChordType::Major7)),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant9)),
        ]
    } else if choice < 60 {
        // I-V-IV (simple, bright)
        vec![
            (0, select_chord_type(&mut rng, 0, ChordType::Major7)),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant13)),
            (3, select_chord_type(&mut rng, 3, ChordType::Major7)),
        ]
    } else if choice < 70 {
        // ii-V-I (uplifting jazz resolution) - use half-diminished
        vec![
            (
                1,
                select_chord_type(&mut rng, 1, ChordType::HalfDiminished7),
            ),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant13)),
            (0, select_chord_type(&mut rng, 0, ChordType::Major9)),
        ]
    } else if choice < 78 {
        // sus4 to Major (dreamy, hopeful)
        vec![
            (0, ChordType::Sus4),
            (0, select_chord_type(&mut rng, 0, ChordType::Major13)),
            (3, select_chord_type(&mut rng, 3, ChordType::Major9)),
        ]
    } else if choice < 85 {
        // I-III-IV-V (bright and jazzy)
        vec![
            (0, select_chord_type(&mut rng, 0, ChordType::Major7)),
            (2, select_chord_type(&mut rng, 2, ChordType::MinorMajor7)),
            (3, select_chord_type(&mut rng, 3, ChordType::Major7)),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant7)),
        ]
    } else if choice < 92 {
        // I-vi-ii-V (smooth jazz standard)
        vec![
            (0, select_chord_type(&mut rng, 0, ChordType::Major9)),
            (5, select_chord_type(&mut rng, 5, ChordType::Minor11)),
            (1, select_chord_type(&mut rng, 1, ChordType::Minor7)),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant9)),
        ]
    } else {
        // vi-IV-I-V (slightly melancholic but resolves happy)
        vec![
            (5, select_chord_type(&mut rng, 5, ChordType::Minor11)),
            (3, select_chord_type(&mut rng, 3, ChordType::Major9)),
            (0, select_chord_type(&mut rng, 0, ChordType::Major13)),
            (4, select_chord_type(&mut rng, 4, ChordType::Dominant9)),
        ]
    };
    let mut progression = Vec::new();

    // Repeat pattern to fill the length
    for i in 0..length {
        let (scale_degree, chord_type) = pattern[i % pattern.len()];
        let root = scale_notes[scale_degree % scale_notes.len()];
        progression.push(Chord { root, chord_type });
    }

    progression
}

/// Convert MIDI note to frequency in Hz
pub fn midi_to_freq(midi_note: MidiNote) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

/// Tempo in BPM
#[derive(Debug, Clone, Copy)]
pub struct Tempo {
    pub bpm: f32,
}

impl Tempo {
    /// Generate a random tempo suitable for funk/jazz (90-130 BPM)
    #[allow(dead_code)]
    pub fn random_funky() -> Self {
        Self::random_funky_range(90.0, 130.0)
    }

    /// Generate a random tempo within a specific range
    pub fn random_funky_range(min_bpm: f32, max_bpm: f32) -> Self {
        let mut rng = rand::thread_rng();
        Tempo {
            bpm: rng.gen_range(min_bpm..max_bpm),
        }
    }

    /// Get the duration of one beat in seconds
    pub fn beat_duration(&self) -> f32 {
        60.0 / self.bpm
    }

    /// Get the duration of one bar (4 beats) in seconds
    pub fn bar_duration(&self) -> f32 {
        self.beat_duration() * 4.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = Key::random_funky();
        let notes = key.get_scale_notes();
        assert!(!notes.is_empty());
        println!("Key root: {}, notes: {:?}", key.root, notes);
    }

    #[test]
    fn test_chord_progression() {
        let key = Key::random_funky();
        let progression = generate_chord_progression(&key, 4);
        assert_eq!(progression.len(), 4);
        for chord in progression {
            let notes = chord.get_notes();
            println!("Chord notes: {:?}", notes);
        }
    }

    #[test]
    fn test_midi_to_freq() {
        assert!((midi_to_freq(69) - 440.0).abs() < 0.01);
        assert!((midi_to_freq(60) - 261.63).abs() < 0.01);
    }
}
