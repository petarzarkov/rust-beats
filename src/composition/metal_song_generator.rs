use crate::composition::{
    drum_humanizer::{DrumHumanizer, BlastBeatStyle, generate_blast_beat, blast_beat_velocity},
    fretboard::{FretboardPathfinder, calculate_playability_score},
    music_theory::{Key, ScaleType, MidiNote},
    tuning::GuitarTuning,
};
use rand::Rng;

/// Legacy genre enum for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    SwampMetal,
}


/// Metal song structure sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetalSection {
    Intro,
    Verse,
    Chorus,
    Breakdown,
    Solo,
    Outro,
}

/// Intensity level for song sections
/// Used to vary riff generation and mixing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SectionIntensity {
    Low,      // Intro, build-up
    Medium,   // Verse
    High,     // Chorus
    Extreme,  // Breakdown, climax
}

impl MetalSection {
    /// Get the intensity level for this section
    pub fn intensity(&self) -> SectionIntensity {
        match self {
            MetalSection::Intro => SectionIntensity::Low,
            MetalSection::Verse => SectionIntensity::Medium,
            MetalSection::Chorus => SectionIntensity::High,
            MetalSection::Breakdown => SectionIntensity::Extreme,
            MetalSection::Solo => SectionIntensity::High,
            MetalSection::Outro => SectionIntensity::Low,
        }
    }
}

impl RhythmPattern {
    /// Convert rhythm pattern to duration in beats
    pub fn to_beats(&self) -> f32 {
        match self {
            RhythmPattern::QuarterNote => 1.0,
            RhythmPattern::EighthNote => 0.5,
            RhythmPattern::SixteenthNote => 0.25,
            RhythmPattern::ThirtySecondNote => 0.125,
            RhythmPattern::Gallop => 0.5, // Gallop is a compound pattern
            RhythmPattern::Rest => 0.0,
        }
    }

    /// Get the number of notes in this pattern (for gallop)
    pub fn note_count(&self) -> usize {
        match self {
            RhythmPattern::Gallop => 3, // Eighth + two sixteenths
            _ => 1,
        }
    }

    /// Get durations for a gallop pattern [eighth, sixteenth, sixteenth]
    pub fn gallop_durations(&self, beat_duration: f32) -> Vec<f32> {
        match self {
            RhythmPattern::Gallop => vec![
                beat_duration / 2.0,  // Eighth note
                beat_duration / 4.0, // Sixteenth note
                beat_duration / 4.0, // Sixteenth note
            ],
            _ => vec![self.to_beats() * beat_duration],
        }
    }
}

/// Metal subgenre for style-specific generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetalSubgenre {
    HeavyMetal,     // Traditional heavy metal
    ThrashMetal,    // Fast, aggressive
    DeathMetal,     // Brutal, low-tuned
    DoomMetal,      // Slow, heavy
    ProgressiveMetal, // Complex, technical
}

impl MetalSubgenre {
    /// Get appropriate guitar tuning for subgenre
    pub fn default_tuning(&self) -> GuitarTuning {
        match self {
            MetalSubgenre::HeavyMetal => GuitarTuning::EStandard,
            MetalSubgenre::ThrashMetal => GuitarTuning::EStandard,
            MetalSubgenre::DeathMetal => GuitarTuning::DStandard,
            MetalSubgenre::DoomMetal => GuitarTuning::CStandard,
            MetalSubgenre::ProgressiveMetal => GuitarTuning::DropC,
        }
    }

    /// Get appropriate scale for subgenre
    pub fn default_scale(&self) -> ScaleType {
        match self {
            MetalSubgenre::HeavyMetal => ScaleType::MinorPentatonic,
            MetalSubgenre::ThrashMetal => ScaleType::Phrygian,
            MetalSubgenre::DeathMetal => ScaleType::Phrygian,
            MetalSubgenre::DoomMetal => ScaleType::Dorian,
            MetalSubgenre::ProgressiveMetal => ScaleType::HarmonicMinor,
        }
    }

    /// Get tempo range for subgenre (min, max BPM)
    pub fn tempo_range(&self) -> (u16, u16) {
        match self {
            MetalSubgenre::HeavyMetal => (120, 160),
            MetalSubgenre::ThrashMetal => (160, 220),
            MetalSubgenre::DeathMetal => (140, 200),
            MetalSubgenre::DoomMetal => (60, 100),
            MetalSubgenre::ProgressiveMetal => (100, 180),
        }
    }
}

/// Type of chord to play
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordType {
    Single,     // Single note
    Power,      // Root + 5th + Octave (Power Chord)
    Minor,      // Root + b3 + 5 (Minor Triad)
    Diminished, // Root + b3 + b5 (Diminished Triad)
    Octave,     // Root + Octave
}

/// Rhythm patterns for metal riffs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmPattern {
    QuarterNote,    // Whole beat
    EighthNote,     // Half beat
    SixteenthNote,  // Quarter beat
    ThirtySecondNote, // Eighth beat (tremolo)
    Gallop,         // Eighth + two sixteenths (special pattern)
    Rest,           // Silence
}

/// A note event with pitch, rhythm, and articulation
#[derive(Debug, Clone)]
pub struct NoteEvent {
    pub pitch: MidiNote,
    pub rhythm: RhythmPattern,
    pub palm_muted: bool,
    pub chord_type: ChordType,
}

/// A complete metal riff with notes, chords, palm muting, and rhythms
#[derive(Debug, Clone)]
pub struct MetalRiff {
    pub notes: Vec<MidiNote>,
    pub chord_types: Vec<ChordType>,
    pub palm_muted: Vec<bool>,
    pub rhythms: Vec<RhythmPattern>, // New: rhythm patterns for each note
    pub playability_score: f32,
}

/// A complete metal song structure
#[derive(Debug, Clone)]
pub struct MetalSong {
    pub subgenre: MetalSubgenre,
    pub key: Key,
    pub tempo: u16,
    pub tuning: GuitarTuning,
    pub sections: Vec<(MetalSection, MetalRiff)>,
    pub drum_humanizer: DrumHumanizer,
}

/// Metal song generator - integrates all components
pub struct MetalSongGenerator {
    subgenre: MetalSubgenre,
    tuning: GuitarTuning,
    key: Key,
    tempo: u16,
}

impl MetalSongGenerator {
    /// Create a new metal song generator
    pub fn new(subgenre: MetalSubgenre) -> Self {
        let mut rng = rand::thread_rng();
        
        // Choose tuning and scale based on subgenre
        let tuning = subgenre.default_tuning();
        let scale_type = subgenre.default_scale();
        
        // Choose root note based on tuning
        let root = tuning.lowest_note();
        let key = Key {
            root,
            scale_type,
        };
        
        // Choose tempo within subgenre range
        let (min_tempo, max_tempo) = subgenre.tempo_range();
        let tempo = rng.gen_range(min_tempo..=max_tempo);
        
        MetalSongGenerator {
            subgenre,
            tuning,
            key,
            tempo,
        }
    }

    /// Generate a complete metal riff for a section
    /// Now varies based on section intensity and type
    pub fn generate_riff(&self, section: MetalSection) -> MetalRiff {
        let _intensity = section.intensity();
        match section {
            MetalSection::Intro => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                // Intro: Low intensity, maybe single guitar or build-up
                // Simpler, more sparse
                let notes = self.generate_intro_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Verse => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                // Verse: Palm-muted chugs, tight rhythm
                let notes = self.generate_verse_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Chorus => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                // Chorus: Open power chords, wider, more melodic
                let notes = self.generate_chorus_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Breakdown => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                // Breakdown: Half-time feel, synchronized rhythmic stabs
                let notes = self.generate_breakdown_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Solo => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                // Solo: More melodic, faster
                let notes = self.generate_solo_sequence(root, scale, 32);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Outro => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                // Outro: Fade out, simpler
                let notes = self.generate_outro_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
        }
    }

    /// Build a MetalRiff from notes with appropriate palm muting, chords, and rhythms
    fn build_riff_from_notes(&self, notes: Vec<MidiNote>, section: MetalSection) -> MetalRiff {
        let root = self.key.root;
        
        // Generate rhythm patterns based on section and subgenre
        let rhythms = self.generate_rhythm_patterns(notes.len(), section);
        
        // Determine palm muting and chords based on section and intensity
        let mut palm_muted = Vec::new();
        let mut chord_types = Vec::new();
        
        for (i, &note) in notes.iter().enumerate() {
            let is_pedal = note == root || note == root + 12;
            let is_strong_beat = i % 4 == 0;
            let rhythm = rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            // If rest, skip this note (will be handled in renderer)
            if rhythm == RhythmPattern::Rest {
                palm_muted.push(true);
                chord_types.push(ChordType::Single);
                continue;
            }
            
            match section {
                MetalSection::Intro => {
                    // Intro: Mostly single notes, occasional power chords
                    if is_strong_beat && i % 8 == 0 {
                        palm_muted.push(false);
                        chord_types.push(ChordType::Power);
                    } else {
                        palm_muted.push(true);
                        chord_types.push(ChordType::Single);
                    }
                },
                MetalSection::Verse => {
                    // Verse: Palm-muted chugs
                    if is_pedal {
                        palm_muted.push(true);
                        chord_types.push(ChordType::Single);
                    } else {
                        palm_muted.push(true);
                        chord_types.push(ChordType::Power);
                    }
                },
                MetalSection::Chorus => {
                    // Chorus: Open power chords, wide stereo
                    if is_strong_beat {
                        palm_muted.push(false);
                        chord_types.push(ChordType::Power);
                    } else {
                        palm_muted.push(false);
                        chord_types.push(ChordType::Power);
                    }
                },
                MetalSection::Breakdown => {
                    // Breakdown: Heavy, synchronized stabs
                    if is_strong_beat || (i % 4 == 2) {
                        palm_muted.push(false);
                        chord_types.push(ChordType::Power);
                    } else {
                        palm_muted.push(true);
                        chord_types.push(ChordType::Single);
                    }
                },
                MetalSection::Solo => {
                    // Solo: Single notes, fast
                    palm_muted.push(false);
                    chord_types.push(ChordType::Single);
                },
                MetalSection::Outro => {
                    // Outro: Fade out
                    palm_muted.push(true);
                    chord_types.push(ChordType::Single);
                },
            }
        }

        // Validate playability
        let pathfinder = FretboardPathfinder::new(self.tuning);
        let fret_positions = pathfinder.find_playable_path(&notes);
        let playability_score = calculate_playability_score(&fret_positions);

        MetalRiff {
            notes,
            chord_types,
            palm_muted,
            rhythms,
            playability_score,
        }
    }

    /// Generate rhythm patterns for a riff based on section and subgenre
    fn generate_rhythm_patterns(&self, length: usize, section: MetalSection) -> Vec<RhythmPattern> {
        match section {
            MetalSection::Breakdown => self.generate_breakdown_rhythms(length),
            _ => match self.subgenre {
                MetalSubgenre::ThrashMetal => self.generate_thrash_rhythms(length, section),
                MetalSubgenre::DeathMetal => self.generate_death_rhythms(length, section),
                MetalSubgenre::DoomMetal => self.generate_doom_rhythms(length, section),
                MetalSubgenre::HeavyMetal => self.generate_heavy_rhythms(length, section),
                MetalSubgenre::ProgressiveMetal => self.generate_progressive_rhythms(length, section),
            },
        }
    }

    /// Generate intro sequence (low intensity, sparse)
    fn generate_intro_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        let mut notes = Vec::with_capacity(length);
        
        let intervals = match scale {
            ScaleType::Phrygian => vec![0, 1, 3, 5, 7, 8, 10],
            ScaleType::MinorPentatonic => vec![0, 3, 5, 7, 10],
            ScaleType::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleType::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            _ => vec![0, 2, 4, 5, 7, 9, 11],
        };
        
        for i in 0..length {
            // Intro: More sparse, mostly root with occasional melodic notes
            if i % 4 == 0 || rng.gen_bool(0.2) {
                if rng.gen_bool(0.7) {
                    notes.push(root);
                } else {
                    let interval = intervals[rng.gen_range(0..intervals.len())];
                    notes.push(root + interval);
                }
            } else {
                // Rest or sustain
                notes.push(root);
            }
        }
        notes
    }

    /// Generate verse sequence (palm-muted chugs, tight rhythm)
    fn generate_verse_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        // Verse: More rhythmic, pedal point emphasis
        self.generate_markov_sequence(root, scale, length)
    }

    /// Generate chorus sequence (open power chords, melodic)
    fn generate_chorus_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        let mut notes = Vec::with_capacity(length);
        
        let intervals = match scale {
            ScaleType::Phrygian => vec![0, 1, 3, 5, 7, 8, 10],
            ScaleType::MinorPentatonic => vec![0, 3, 5, 7, 10],
            ScaleType::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleType::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            _ => vec![0, 2, 4, 5, 7, 9, 11],
        };
        
        // Chorus: More melodic movement, less pedal point
        for i in 0..length {
            if i % 4 == 0 {
                // Strong beats: root
                notes.push(root);
            } else {
                // More melodic movement
                if rng.gen_bool(0.4) {
                    let interval = intervals[rng.gen_range(0..intervals.len())];
                    notes.push(root + interval);
                } else {
                    notes.push(root);
                }
            }
        }
        notes
    }

    /// Generate breakdown sequence (half-time feel, synchronized stabs)
    fn generate_breakdown_sequence(&self, root: MidiNote, _scale: ScaleType, length: usize) -> Vec<MidiNote> {
        // Breakdown: Heavy, synchronized, mostly root
        let mut notes = Vec::with_capacity(length);
        for i in 0..length {
            // Breakdown: Strong beats get root, off-beats might be rests or chugs
            if i % 4 == 0 || i % 4 == 2 {
                notes.push(root);
            } else {
                notes.push(root); // Still root, but will be palm muted
            }
        }
        notes
    }

    /// Generate solo sequence (melodic, fast)
    fn generate_solo_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        // Solo: More melodic, faster movement
        self.generate_markov_sequence(root, scale, length)
    }

    /// Generate outro sequence (fade out, simple)
    fn generate_outro_sequence(&self, root: MidiNote, _scale: ScaleType, length: usize) -> Vec<MidiNote> {
        // Outro: Simple, mostly root, fading
        vec![root; length]
    }

    /// Generate a complete metal song structure
    pub fn generate_song(&self) -> MetalSong {
        let mut sections = Vec::new();

        // Extended metal song structure for 2+ minute songs
        sections.push((MetalSection::Intro, self.generate_riff(MetalSection::Intro)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse))); // Bridge/Verse 3
        sections.push((MetalSection::Breakdown, self.generate_riff(MetalSection::Breakdown)));
        sections.push((MetalSection::Solo, self.generate_riff(MetalSection::Solo)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Breakdown, self.generate_riff(MetalSection::Breakdown))); // Final breakdown
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus))); // Final chorus
        sections.push((MetalSection::Outro, self.generate_riff(MetalSection::Outro)));

        // Choose drum humanizer based on subgenre
        let drum_humanizer = match self.subgenre {
            MetalSubgenre::HeavyMetal => DrumHumanizer::new(),
            MetalSubgenre::ThrashMetal => DrumHumanizer::thrash(),
            MetalSubgenre::DeathMetal => DrumHumanizer::blast_beat(),
            MetalSubgenre::DoomMetal => DrumHumanizer::breakdown(),
            MetalSubgenre::ProgressiveMetal => DrumHumanizer::new(),
        };

        MetalSong {
            subgenre: self.subgenre,
            key: self.key,
            tempo: self.tempo,
            tuning: self.tuning,
            sections,
            drum_humanizer,
        }
    }

    /// Get interval weight for metal generation (prioritizes dissonant intervals)
    fn get_interval_weight(interval: u8, _scale: ScaleType, subgenre: MetalSubgenre) -> f32 {
        match interval {
            1 => 2.0,  // Minor second - very metal (b2)
            6 => 1.8,  // Tritone - devil's interval (b5)
            7 => 1.2,  // Perfect fifth - power chord foundation
            10 => 1.1, // Minor seventh
            3 => if matches!(subgenre, MetalSubgenre::DeathMetal) { 1.0 } else { 0.3 }, // Minor third
            4 => 0.2,  // Major third - too happy
            2 => 0.5,  // Major second - less interesting
            5 => 1.0,  // Perfect fourth - good for metal
            8 => 0.8,  // Minor sixth
            9 => 0.4,  // Major sixth
            11 => 1.0, // Major seventh
            _ => 0.5,
        }
    }

    /// Generate a sequence of notes using a Markov-like process with metal interval weighting
    fn generate_markov_sequence(&self, root: u8, scale: ScaleType, length: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut notes = Vec::with_capacity(length);
        
        // Start with root
        let mut current_note = root;
        notes.push(current_note);
        
        // Get scale intervals
        let intervals = match scale {
            ScaleType::Phrygian => vec![0, 1, 3, 5, 7, 8, 10], // 1, b2, b3, 4, 5, b6, b7
            ScaleType::MinorPentatonic => vec![0, 3, 5, 7, 10],
            ScaleType::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleType::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            _ => vec![0, 2, 4, 5, 7, 9, 11], // Major/Ionian fallback
        };
        
        // Calculate weights for each interval
        let mut weighted_intervals = Vec::new();
        for &interval in &intervals {
            let weight = Self::get_interval_weight(interval, scale, self.subgenre);
            weighted_intervals.push((interval, weight));
        }
        
        for _ in 1..length {
            // Enhanced pedal point logic: 70-80% chance to return to root
            let pedal_prob = if matches!(self.subgenre, MetalSubgenre::DoomMetal) { 0.8 } else { 0.75 };
            
            if rng.gen_bool(pedal_prob) {
                current_note = root;
            } else {
                // Weighted selection based on metal intervals
                // Prioritize minor second (b2) when leaving pedal
                let total_weight: f32 = weighted_intervals.iter().map(|(_, w)| w).sum();
                let mut rand_val = rng.gen::<f32>() * total_weight;
                
                for &(interval, weight) in &weighted_intervals {
                    if rand_val <= weight {
                        current_note = root + interval;
                        break;
                    }
                    rand_val -= weight;
                }
            }
            notes.push(current_note);
        }
        
        notes
    }

    /// Generate thrash metal rhythms (gallop patterns)
    fn generate_thrash_rhythms(&self, length: usize, section: MetalSection) -> Vec<RhythmPattern> {
        let _ = section; // Used in match below
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let mut i = 0;
        
        while i < length {
            if matches!(section, MetalSection::Verse) && rng.gen_bool(0.4) {
                // Gallop pattern: Eighth + two sixteenths
                if i + 2 < length {
                    rhythms.push(RhythmPattern::Gallop);
                    i += 1; // Gallop counts as one pattern but represents 3 notes
                } else {
                    rhythms.push(RhythmPattern::SixteenthNote);
                    i += 1;
                }
            } else if rng.gen_bool(0.3) {
                // Rest for chugging space
                rhythms.push(RhythmPattern::Rest);
                i += 1;
            } else if rng.gen_bool(0.5) {
                rhythms.push(RhythmPattern::EighthNote);
                i += 1;
            } else {
                rhythms.push(RhythmPattern::SixteenthNote);
                i += 1;
            }
        }
        
        rhythms
    }

    /// Generate death metal rhythms (tremolo picking bursts)
    fn generate_death_rhythms(&self, length: usize, _section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let mut i = 0;
        
        while i < length {
            if rng.gen_bool(0.3) {
                // Tremolo burst: 4-8 thirty-second notes
                let burst_len = rng.gen_range(4..=8);
                for _ in 0..burst_len.min(length - i) {
                    rhythms.push(RhythmPattern::ThirtySecondNote);
                    i += 1;
                }
                // Rest after burst
                if i < length && rng.gen_bool(0.7) {
                    rhythms.push(RhythmPattern::Rest);
                    i += 1;
                }
            } else if rng.gen_bool(0.4) {
                rhythms.push(RhythmPattern::Rest);
                i += 1;
            } else {
                rhythms.push(RhythmPattern::SixteenthNote);
                i += 1;
            }
        }
        
        rhythms
    }

    /// Generate doom metal rhythms (slow, heavy)
    fn generate_doom_rhythms(&self, length: usize, _section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let mut i = 0;
        
        while i < length {
            if rng.gen_bool(0.5) {
                // Rest for heaviness
                rhythms.push(RhythmPattern::Rest);
                i += 1;
            } else if rng.gen_bool(0.6) {
                rhythms.push(RhythmPattern::QuarterNote);
                i += 1;
            } else {
                rhythms.push(RhythmPattern::EighthNote);
                i += 1;
            }
        }
        
        rhythms
    }

    /// Generate heavy metal rhythms (mix of eighth and sixteenth)
    fn generate_heavy_rhythms(&self, length: usize, section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let rest_prob = match section {
            MetalSection::Verse => 0.35,
            MetalSection::Chorus => 0.15,
            MetalSection::Intro => 0.5,
            _ => 0.25,
        };
        
        for i in 0..length {
            if rng.gen_bool(rest_prob) {
                rhythms.push(RhythmPattern::Rest);
            } else if i % 4 == 0 {
                // Strong beats: eighth notes
                rhythms.push(RhythmPattern::EighthNote);
            } else {
                rhythms.push(RhythmPattern::SixteenthNote);
            }
        }
        
        rhythms
    }

    /// Generate progressive metal rhythms (polymetric patterns)
    fn generate_progressive_rhythms(&self, length: usize, section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        
        for i in 0..length {
            if rng.gen_bool(0.25) {
                rhythms.push(RhythmPattern::Rest);
            } else if i % 5 == 0 || i % 7 == 0 {
                // Polymetric accents
                rhythms.push(RhythmPattern::EighthNote);
            } else {
                rhythms.push(RhythmPattern::SixteenthNote);
            }
        }
        
        rhythms
    }

    /// Generate breakdown rhythms (half-time feel: quarter notes)
    fn generate_breakdown_rhythms(&self, length: usize) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        
        for i in 0..length {
            if i % 4 == 0 || i % 4 == 2 {
                // Strong beats: quarter note power chords
                rhythms.push(RhythmPattern::QuarterNote);
            } else if rng.gen_bool(0.6) {
                // 60% rests for impact
                rhythms.push(RhythmPattern::Rest);
            } else {
                rhythms.push(RhythmPattern::EighthNote);
            }
        }
        
        rhythms
    }

    /// Generate drums for a section
    pub fn generate_drums(&self, section: MetalSection, humanizer: &DrumHumanizer) -> Vec<(u8, i32)> {
        let mut drum_hits = Vec::new();
        let subdivisions = match section {
            MetalSection::Breakdown => 8,  // Slower, heavier
            _ => 16, // Standard 16th notes
        };

        // Generate blast beat for appropriate sections
        let use_blast = matches!(
            (self.subgenre, section),
            (MetalSubgenre::DeathMetal, MetalSection::Verse) |
            (MetalSubgenre::DeathMetal, MetalSection::Chorus)
        );

        if use_blast {
            let (kicks, snares) = generate_blast_beat(BlastBeatStyle::Traditional, subdivisions);
            
            for i in 0..subdivisions {
                if kicks[i] || snares[i] {
                    let base_velocity = blast_beat_velocity(100, i == 0);
                    let (velocity, timing) = humanizer.humanize_hit(base_velocity, i == 0);
                    drum_hits.push((velocity, timing));
                }
            }
        } else {
            // Standard pattern
            for i in 0..subdivisions {
                let is_accent = i % 4 == 0; // Accent on beats
                let base_velocity = if is_accent { 110 } else { 95 };
                let (velocity, timing) = humanizer.humanize_hit(base_velocity, is_accent);
                drum_hits.push((velocity, timing));
            }
        }

        drum_hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metal_subgenre_tuning() {
        assert_eq!(MetalSubgenre::HeavyMetal.default_tuning(), GuitarTuning::EStandard);
        assert_eq!(MetalSubgenre::DeathMetal.default_tuning(), GuitarTuning::DStandard);
    }

    #[test]
    fn test_metal_subgenre_tempo() {
        let (min, max) = MetalSubgenre::ThrashMetal.tempo_range();
        assert!(min >= 160);
        assert!(max <= 220);
    }

    #[test]
    fn test_song_generator_creation() {
        let generator = MetalSongGenerator::new(MetalSubgenre::HeavyMetal);
        assert_eq!(generator.subgenre, MetalSubgenre::HeavyMetal);
        assert!(generator.tempo >= 120 && generator.tempo <= 160);
    }

    #[test]
    fn test_generate_riff() {
        let generator = MetalSongGenerator::new(MetalSubgenre::DeathMetal);
        let riff = generator.generate_riff(MetalSection::Verse);
        
        assert_eq!(riff.notes.len(), 16);
        assert_eq!(riff.palm_muted.len(), 16);
        assert!(riff.playability_score >= 0.0 && riff.playability_score <= 1.0);
    }

    #[test]
    fn test_generate_song() {
        let generator = MetalSongGenerator::new(MetalSubgenre::ThrashMetal);
        let song = generator.generate_song();
        
        assert_eq!(song.subgenre, MetalSubgenre::ThrashMetal);
        assert!(song.sections.len() > 0);
        assert!(song.tempo >= 160 && song.tempo <= 220);
    }

    #[test]
    fn test_generate_drums() {
        let generator = MetalSongGenerator::new(MetalSubgenre::DeathMetal);
        let humanizer = DrumHumanizer::blast_beat();
        let drums = generator.generate_drums(MetalSection::Verse, &humanizer);
        
        assert!(drums.len() > 0);
        // Check velocity and timing are within valid ranges
        for (velocity, timing) in drums {
            assert!(velocity >= 1 && velocity <= 127);
            assert!(timing.abs() < 100); // Reasonable timing offset
        }
    }

    #[test]
    fn test_all_subgenres() {
        let subgenres = vec![
            MetalSubgenre::HeavyMetal,
            MetalSubgenre::ThrashMetal,
            MetalSubgenre::DeathMetal,
            MetalSubgenre::DoomMetal,
            MetalSubgenre::ProgressiveMetal,
        ];

        for subgenre in subgenres {
            let generator = MetalSongGenerator::new(subgenre);
            let song = generator.generate_song();
            assert_eq!(song.subgenre, subgenre);
        }
    }

    #[test]
    fn test_riff_playability() {
        let generator = MetalSongGenerator::new(MetalSubgenre::ProgressiveMetal);
        let riff = generator.generate_riff(MetalSection::Solo);
        
        // Solo should be longer
        assert_eq!(riff.notes.len(), 32);
        // Should have a playability score
        assert!(riff.playability_score > 0.0);
    }
}
