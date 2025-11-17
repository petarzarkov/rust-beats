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
    MinorPentatonic,
    MajorPentatonic,
    Blues,
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
}

impl ScaleType {
    /// Get the intervals for this scale type (in semitones from root)
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            ScaleType::Major => vec![0, 2, 4, 5, 7, 9, 11],
            ScaleType::Minor => vec![0, 2, 3, 5, 7, 8, 10],
            ScaleType::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            ScaleType::Mixolydian => vec![0, 2, 4, 5, 7, 9, 10],
            ScaleType::MinorPentatonic => vec![0, 3, 5, 7, 10],
            ScaleType::MajorPentatonic => vec![0, 2, 4, 7, 9],
            ScaleType::Blues => vec![0, 3, 5, 6, 7, 10],
        }
    }
}

impl Key {
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
        
        let scales = vec![
            ScaleType::Major,           // Bright, happy
            ScaleType::Minor,            // Dark, moody
            ScaleType::Dorian,           // Jazzy, sophisticated
            ScaleType::Mixolydian,       // Funky, dominant sound
            ScaleType::MinorPentatonic,  // Blues, rock
            ScaleType::MajorPentatonic,  // Country, pop
            ScaleType::Blues,            // Classic blues sound
        ];
        
        Key {
            root: roots[rng.gen_range(0..roots.len())],
            scale_type: scales[rng.gen_range(0..scales.len())],
        }
    }
    
    /// Get all notes in this key within an octave
    pub fn get_scale_notes(&self) -> Vec<MidiNote> {
        self.scale_type.intervals()
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
        };
        
        intervals.iter()
            .map(|&interval| self.root + interval)
            .filter(|&note| note < 128)
            .collect()
    }
}

/// Generate a chord progression suitable for funk/jazz
pub fn generate_chord_progression(key: &Key, length: usize) -> Vec<Chord> {
    let mut rng = rand::thread_rng();
    let scale_notes = key.get_scale_notes();
    
    // Common funk/jazz progressions patterns
    let patterns = vec![
        // ii-V-I (jazz standard)
        vec![(1, ChordType::Minor7), (4, ChordType::Dominant7), (0, ChordType::Major7)],
        // i-IV-V (funk standard)
        vec![(0, ChordType::Minor7), (3, ChordType::Dominant7), (4, ChordType::Dominant7)],
        // I-vi-ii-V
        vec![(0, ChordType::Major7), (5, ChordType::Minor7), (1, ChordType::Minor7), (4, ChordType::Dominant7)],
        // Funky i-iv
        vec![(0, ChordType::Minor7), (3, ChordType::Minor7)],
        // I-IV with 7ths
        vec![(0, ChordType::Major7), (3, ChordType::Major7)],
        // Sus4 funk pattern (creates tension/release)
        vec![(0, ChordType::Sus4), (3, ChordType::Major), (0, ChordType::Minor7)],
        // Diminished passing chords (jazz)
        vec![(0, ChordType::Major7), (0, ChordType::Diminished), (1, ChordType::Minor7), (4, ChordType::Dominant7)],
        // I-IV-V with variety
        vec![(0, ChordType::Major), (3, ChordType::Sus4), (4, ChordType::Dominant7)],
    ];
    
    let pattern = &patterns[rng.gen_range(0..patterns.len())];
    let mut progression = Vec::new();
    
    // Repeat pattern to fill the length
    for i in 0..length {
        let (scale_degree, chord_type) = pattern[i % pattern.len()];
        let root = scale_notes[scale_degree % scale_notes.len()];
        progression.push(Chord {
            root,
            chord_type,
        });
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

