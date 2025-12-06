use crate::composition::{
    drum_humanizer::{DrumHumanizer, BlastBeatStyle, generate_blast_beat, blast_beat_velocity},
    fretboard::{FretboardPathfinder, PlayabilityMode, calculate_playability_score},
    music_theory::{Key, ScaleType, MidiNote},
    tuning::GuitarTuning,
    rhythm::{euclidean_rhythm, rotate_rhythm, PolymetricInterference},
    riff_generator::{MetalMarkovPresets, PedalPointGenerator, ChromaticMutator},
    riff_motifs::MotifLibrary,
    breakdown_generator::BreakdownGenerator,
    bar_memory::BarMotifStore,
    phrase_drums::PhraseAwareDrumGenerator,
};
use crate::synthesis::aggressive_mix::AggressiveMixPipeline;
use rand::Rng;

/// Legacy genre enum for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    SwampMetal,
}

/// Defines the rhythmic feel of the drums relative to the tempo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmicFeel {
    HalfTime,    // Drums feel like tempo is 50% (Breakdowns, Sludge)
    Normal,      // Standard 4/4
    DoubleTime,  // Drums feel 2x faster (Thrash Skank beats)
    Blast,       // Maximum density
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

    /// Get the rhythmic feel (Drum Tempo modifier)
    pub fn rhythmic_feel(&self) -> RhythmicFeel {
        match self {
            MetalSection::Intro => RhythmicFeel::Normal,
            MetalSection::Verse => RhythmicFeel::Normal, // Could be Blast for Death Metal logic
            MetalSection::Chorus => RhythmicFeel::Normal,
            MetalSection::Breakdown => RhythmicFeel::HalfTime, // CRITICAL: Fixes DnB feel
            MetalSection::Solo => RhythmicFeel::Normal,
            MetalSection::Outro => RhythmicFeel::HalfTime,
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
            RhythmPattern::Quintuplet => 0.8, // 5 notes in 4 beats = 4/5 per note
            RhythmPattern::Septuplet => 0.571, // 7 notes in 4 beats = 4/7 per note
            RhythmPattern::DottedEighth => 0.75, // 3/16 of a bar
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
    Quintuplet,     // 5 notes in 4 beats
    Septuplet,      // 7 notes in 4 beats
    DottedEighth,   // Dotted 8th note (3/16)
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
    pub motif_library: MotifLibrary,
    pub chromatic_mutator: ChromaticMutator,
    pub breakdown_generator: BreakdownGenerator,
    pub aggressive_pathfinder: FretboardPathfinder,
    pub bar_memory: BarMotifStore,
    pub phrase_drums: PhraseAwareDrumGenerator,
    pub polymeter: PolymetricInterference,
    pub mix_pipeline: AggressiveMixPipeline,
    pub chaos_level: f32,
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
        
        // Initialize new enhancement systems
        let motif_library = MotifLibrary::new();
        
        // Chromatic intensity varies by subgenre
        let chromatic_intensity = match subgenre {
            MetalSubgenre::DeathMetal => 0.8,      // Very chromatic
            MetalSubgenre::ProgressiveMetal => 0.7, // Complex
            MetalSubgenre::ThrashMetal => 0.6,     // Moderately chromatic
            MetalSubgenre::HeavyMetal => 0.4,      // Less chromatic
            MetalSubgenre::DoomMetal => 0.5,       // Moderate
        };
        let chromatic_mutator = ChromaticMutator::new(chromatic_intensity);
        
        // Breakdown generator - aggressive for most subgenres
        let breakdown_generator = if matches!(subgenre, MetalSubgenre::DoomMetal) {
            BreakdownGenerator::new() // Standard for doom
        } else {
            BreakdownGenerator::aggressive()
        };
        
        let sample_rate = crate::utils::get_sample_rate();
        
        // Determine playability mode based on subgenre
        let playability_mode = match subgenre {
            MetalSubgenre::ProgressiveMetal => PlayabilityMode::Aggressive,
            MetalSubgenre::DeathMetal => PlayabilityMode::Aggressive,
            _ => PlayabilityMode::Standard,
        };
        
        let pathfinder = FretboardPathfinder::with_mode(tuning, playability_mode);
        
        MetalSongGenerator {
            subgenre,
            tuning,
            key,
            tempo,
            motif_library,
            chromatic_mutator,
            breakdown_generator,
            aggressive_pathfinder: pathfinder,
            bar_memory: BarMotifStore::new(),
            phrase_drums: PhraseAwareDrumGenerator::new(sample_rate, tempo),
            polymeter: PolymetricInterference::prog_metal(),
            mix_pipeline: AggressiveMixPipeline::new(sample_rate),
            chaos_level: match subgenre {
                MetalSubgenre::ProgressiveMetal => 0.7,
                MetalSubgenre::DeathMetal => 0.8,
                _ => 0.5,
            },
        }
    }

    /// Generate a complete metal riff for a section
    /// Now varies based on section intensity and type
    pub fn generate_riff(&self, section: MetalSection) -> MetalRiff {
        let mut rng = rand::thread_rng();
        
        // Use breakdown generator for breakdowns
        if section == MetalSection::Breakdown {
            return self.generate_breakdown_riff();
        }
        
        // NEW: Use chord progressions 50% of the time for more musical structure
        if rng.gen_bool(0.5) {
            return self.generate_chord_progression_riff(section);
        }
        
        // Use motif-based generation for some riffs (40% chance)
        if section != MetalSection::Intro && rng.gen_bool(0.4) {
            return self.generate_motif_based_riff(section);
        }
        
        // Use polymetric riffs for progressive metal and for solos in all subgenres
        if matches!(self.subgenre, MetalSubgenre::ProgressiveMetal) && rng.gen_bool(0.3) {
            return self.generate_polymetric_riff(section);
        }
        
        // Use polymetric patterns for solos across all subgenres (adds complexity)
        if section == MetalSection::Solo && rng.gen_bool(0.5) {
            return self.generate_polymetric_riff(section);
        }
        
        // Standard generation
        match section {
            MetalSection::Intro => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_intro_sequence(root, scale, 24);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Verse => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_verse_sequence(root, scale, 32);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Chorus => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_chorus_sequence(root, scale, 32);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Breakdown => {
                // Already handled above
                self.generate_breakdown_riff()
            },
            MetalSection::Solo => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_solo_sequence(root, scale, 48);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Outro => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_outro_sequence(root, scale, 24);
                self.build_riff_from_notes(notes, section)
            },
        }
    }

    /// Generate a metal chord progression riff (I-IV-V-I style)
    /// Creates actual chord progressions for more musical structure
    fn generate_chord_progression_riff(&self, section: MetalSection) -> MetalRiff {
        let scale_notes = self.key.get_scale_notes();
        let root = self.key.root;
        
        // Metal chord progressions (using scale degrees)
        let progression = match section {
            MetalSection::Intro | MetalSection::Outro => {
                // Simple: I - VI - III - I (e.g., E - C - G - E for E Phrygian)
                vec![0, 5, 2, 0]
            },
            MetalSection::Verse => {
                // I - VII - VI - V repeated twice for 8 chords
                vec![0, 6, 5, 4, 0, 6, 5, 4]
            },
            MetalSection::Chorus => {
                // I - IV - V - I repeated twice (power progression)
                vec![0, 3, 4, 0, 0, 3, 4, 0]
            },
            MetalSection::Solo => {
                // More complex: I - II - III - IV - V - VI - VII - I
                vec![0, 1, 2, 3, 4, 5, 6, 0]
            },
            MetalSection::Breakdown => {
                // Just root (heavy chugging)
                vec![0, 0, 0, 0]
            },
        };
        
        // Convert scale degrees to notes, repeat each chord 8 times (16th notes)
        let mut notes = Vec::new();
        for &degree in &progression {
            let chord_root = scale_notes[degree % scale_notes.len()];
            // Play each chord 8 times for rhythmic feel
            for _ in 0..8 {
                notes.push(chord_root);
            }
        }
        
        self.build_riff_from_notes(notes, section)
    }

    /// Generate polymetric riff for Progressive Metal (Djent)
    /// Research Section 3.1: Uses PolymetricRiff for complex rhythmic structures
    /// Generate a polymetric riff for progressive metal
    fn generate_polymetric_riff(&self, section: MetalSection) -> MetalRiff {
        // Use PolymetricInterference for prog-metal
        let polymeter = PolymetricInterference::prog_metal();
        
        // Generate guitar pattern in odd meter (5/16)
        let guitar_positions = polymeter.guitar_pattern(4); // 4 bars
        
        // Convert positions to notes from scale
        let scale_notes = self.key.get_scale_notes();
        let mut notes = Vec::new();
        
        for pos in guitar_positions {
            let note_idx = (pos / polymeter.guitar_meter) % scale_notes.len();
            notes.push(scale_notes[note_idx]);
        }
        
        // Apply chromatic mutations for complexity
        let mutated_notes = self.chromatic_mutator.apply_mutations(notes);
        
        // Build riff
        self.build_riff_from_notes(mutated_notes, section)
    }

    /// Build a MetalRiff from notes with appropriate palm muting, chords, and rhythms
    fn build_riff_from_notes(&self, notes: Vec<MidiNote>, section: MetalSection) -> MetalRiff {
        let root = self.key.root;
        let mut rng = rand::thread_rng();
        
        // Generate rhythm patterns based on section and subgenre
        let rhythms = self.generate_rhythm_patterns(notes.len(), section);
        
        // Determine palm muting and chords based on section and intensity
        let mut palm_muted = Vec::new();
        let mut chord_types = Vec::new();
        
        // Calculate palm mute probability based on section and subgenre
        let base_palm_mute_prob = match section {
            MetalSection::Breakdown => 0.85,  // 85% palm muted (heavy chugs)
            MetalSection::Verse => 0.65,      // 65% palm muted (tight, staccato)
            MetalSection::Intro => 0.55,      // 55% palm muted (establishes mood)
            MetalSection::Chorus => 0.45,     // 45% palm muted (mixed for dynamics)
            MetalSection::Solo => 0.25,       // 25% palm muted (mostly open for sustain)
            MetalSection::Outro => 0.60,      // 60% palm muted (controlled fade)
        };
        
        // Adjust for subgenre
        let palm_mute_prob = match self.subgenre {
            MetalSubgenre::DeathMetal => (base_palm_mute_prob + 0.15_f32).min(0.95), // Death metal: tighter
            MetalSubgenre::ThrashMetal => (base_palm_mute_prob + 0.10_f32).min(0.90), // Thrash: aggressive chugs
            MetalSubgenre::DoomMetal => (base_palm_mute_prob - 0.10_f32).max(0.20),   // Doom: more open, droning
            MetalSubgenre::ProgressiveMetal => base_palm_mute_prob,                // Prog: balanced
            MetalSubgenre::HeavyMetal => (base_palm_mute_prob - 0.05_f32).max(0.25),  // Heavy: classic open sound
        };
        
        for (i, &note) in notes.iter().enumerate() {
            let is_pedal = note == root || note == root + 12;
            let is_strong_beat = i % 4 == 0;
            let rhythm = rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            // Skip rests
            if rhythm == RhythmPattern::Rest {
                palm_muted.push(true);
                chord_types.push(ChordType::Single);
                continue;
            }
            
            // Determine palm muting with probability
            let should_palm_mute = if section == MetalSection::Breakdown {
                // Breakdown: alternate palm mute/open for impact
                i % 2 != 0
            } else if section == MetalSection::Solo {
                // Solo: mostly open, occasional palm mute for accents
                !is_strong_beat && rng.gen_bool(palm_mute_prob as f64)
            } else {
                // Other sections: use probability, but pedal notes are always palm muted
                is_pedal || rng.gen_bool(palm_mute_prob as f64)
            };
            
            palm_muted.push(should_palm_mute);
            
            // Chord selection based on section and palm muting
            let chord = match section {
                MetalSection::Breakdown => {
                    if should_palm_mute {
                        ChordType::Single
                    } else {
                        ChordType::Power // Heavy power chords on accents
                    }
                },
                MetalSection::Chorus => {
                    // Chorus: open power chords and occasional minor chords
                    if is_strong_beat && rng.gen_bool(0.3) {
                        ChordType::Minor
                    } else {
                        ChordType::Power
                    }
                },
                MetalSection::Solo => {
                    // Solo: mostly single notes, occasional octaves on strong beats
                    if is_strong_beat && i % 8 == 0 {
                        ChordType::Octave
                    } else {
                        ChordType::Single
                    }
                },
                MetalSection::Verse => {
                    // Verse: mix of single notes and power chords
                    if should_palm_mute {
                        ChordType::Single
                    } else if is_strong_beat {
                        if rng.gen_bool(0.5) {
                            ChordType::Minor
                        } else {
                            ChordType::Power
                        }
                    } else {
                        ChordType::Power
                    }
                },
                MetalSection::Intro => {
                    // Intro: atmospheric, octaves on strong beats
                    if is_strong_beat && i % 8 == 0 {
                        if rng.gen_bool(0.6) {
                            ChordType::Octave
                        } else {
                            ChordType::Power
                        }
                    } else {
                        ChordType::Single
                    }
                },
                MetalSection::Outro => {
                    // Outro: simple, fading
                    if is_strong_beat {
                        ChordType::Octave
                    } else {
                        ChordType::Single
                    }
                },
            };
            
            chord_types.push(chord);
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
            MetalSection::Breakdown => {
                // BREAKDOWN: Sparse, quarter notes. NOT 16th notes.
                // This prevents the "noisy" machine gun effect
                let mut rhythms = Vec::new();
                for i in 0..length {
                    if i % 2 == 0 {
                         // Quarter note feel
                         rhythms.push(RhythmPattern::QuarterNote);
                    } else {
                        // Space
                        rhythms.push(RhythmPattern::Rest);
                    }
                }
                rhythms
            },
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
    /// Research: Intro establishes the pedal point foundation
    fn generate_intro_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        let notes = self.generate_markov_sequence_with_pedal(root, scale, length, 0.75); // Increased from 0.60
        // Light chromatic mutations for intro (0.15 intensity)
        let mutator = ChromaticMutator::new(0.15);
        mutator.apply_mutations(notes)
    }

    /// Generate verse sequence (palm-muted chugs, tight rhythm)
    /// Research: Verses should heavily use pedal point technique (0-1-0-1 pattern)
    fn generate_verse_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        let notes = self.generate_markov_sequence_with_pedal(root, scale, length, 0.70); // Increased from 0.50
        // Moderate chromatic mutations for verse (0.25 intensity, higher for death metal)
        let intensity = if matches!(self.subgenre, MetalSubgenre::DeathMetal) { 0.35 } else { 0.25 };
        let mutator = ChromaticMutator::new(intensity);
        mutator.apply_mutations(notes)
    }

    /// Generate chorus sequence (open power chords, melodic)
    /// Research: Chorus still uses pedal points but with more melodic movement
    fn generate_chorus_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        let notes = self.generate_markov_sequence_with_pedal(root, scale, length, 0.45); // Increased from 0.30
        // Moderate chromatic mutations for chorus (0.30 intensity)
        let mutator = ChromaticMutator::new(0.30);
        mutator.apply_mutations(notes)
    }

    /// Generate solo sequence (melodic, fast)
    /// Research: Solos are more melodic, less pedal-focused
    fn generate_solo_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        let notes = self.generate_markov_sequence_with_pedal(root, scale, length, 0.30); // Increased from 0.20
        // High chromatic mutations for solo (0.50 intensity, even higher for prog)
        let intensity = if matches!(self.subgenre, MetalSubgenre::ProgressiveMetal) { 0.60 } else { 0.50 };
        let mutator = ChromaticMutator::new(intensity);
        mutator.apply_mutations(notes)
    }

    /// Generate outro sequence (fade out, simple)
    /// Research: Outro returns to simple pedal point pattern
    fn generate_outro_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        let notes = self.generate_markov_sequence_with_pedal(root, scale, length, 0.85); // Increased from 0.80
        // Light chromatic mutations for outro (0.20 intensity)
        let mutator = ChromaticMutator::new(0.20);
        mutator.apply_mutations(notes)
    }

    /// Generate a complete metal song structure following IRVD framework
    /// Research Section 4: Introduction, Repetition, Variation, Destruction
    pub fn generate_song(&self) -> MetalSong {
        let mut rng = rand::thread_rng();
        let mut sections = Vec::new();

        // ==== I: INTRODUCTION ====
        // Sparse, establishing pedal point foundation
        sections.push((MetalSection::Intro, self.generate_riff(MetalSection::Intro)));
        
        // ==== R: REPETITION ====
        // Generate main verse and chorus riffs ONCE, then repeat with variations
        let main_verse_riff = self.generate_riff(MetalSection::Verse);
        let main_chorus_riff = self.generate_riff(MetalSection::Chorus);
        
        // Repeat verse-chorus pattern 2-3 times with same core riff
        let verse_chorus_cycles = rng.gen_range(2..=3);
        for cycle in 0..verse_chorus_cycles {
            if cycle == 0 {
                // First cycle: Use original riffs
                sections.push((MetalSection::Verse, main_verse_riff.clone()));
                sections.push((MetalSection::Chorus, main_chorus_riff.clone()));
            } else {
                // ==== V: VARIATION ====
                // Subsequent cycles: Apply subtle variations to the same riff
                sections.push((MetalSection::Verse, self.apply_variation(&main_verse_riff, 0.15)));
                sections.push((MetalSection::Chorus, self.apply_variation(&main_chorus_riff, 0.20)));
            }
        }
        
        // Optional breakdown before solo (climax building)
        if rng.gen_bool(0.5) {
            sections.push((MetalSection::Breakdown, self.generate_riff(MetalSection::Breakdown)));
        }
        
        // Solo: Peak of melodic expression (70% chance)
        if rng.gen_bool(0.7) {
            sections.push((MetalSection::Solo, self.generate_riff(MetalSection::Solo)));
        }
        
        // Return to chorus (recognition - same riff with variation)
        sections.push((MetalSection::Chorus, self.apply_variation(&main_chorus_riff, 0.25)));
        
        // ==== D: DESTRUCTION ====
        // Final breakdown: Maximum intensity and simplicity (brutal chugging)
        if rng.gen_bool(0.85) { // Increased from 0.8
            sections.push((MetalSection::Breakdown, self.generate_riff(MetalSection::Breakdown)));
        }
        
        // Optional outro: Return to intro-like simplicity (circular structure)
        if rng.gen_bool(0.6) {
            sections.push((MetalSection::Outro, self.generate_riff(MetalSection::Outro)));
        } else {
            // End with final stripped-down chorus (IRVD conclusion)
            sections.push((MetalSection::Chorus, self.apply_variation(&main_chorus_riff, 0.10)));
        }

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



    /// Generate sequence using advanced Markov chains and pedal point logic
    /// This replaces the old weighted random approach with proper music theory
    fn generate_markov_sequence_with_pedal(&self, root: u8, scale: ScaleType, length: usize, pedal_prob: f64) -> Vec<u8> {
        // Use the advanced Markov chain from riff_generator.rs
        let key = Key { root, scale_type: scale };
        
        // For high pedal probability, use PedalPointGenerator
        if pedal_prob > 0.5 {
            let mut pedal_gen = PedalPointGenerator::from_key(&key);
            pedal_gen.return_probability = pedal_prob as f32;
            pedal_gen.generate_sequence(length)
        } else {
            // For lower pedal probability, use Markov chain for more melodic movement
            let mut markov = match self.subgenre {
                MetalSubgenre::HeavyMetal => MetalMarkovPresets::heavy_metal(&key),
                MetalSubgenre::ThrashMetal => MetalMarkovPresets::death_metal(&key), // Thrash uses aggressive patterns
                MetalSubgenre::DeathMetal => MetalMarkovPresets::death_metal(&key),
                MetalSubgenre::DoomMetal => MetalMarkovPresets::heavy_metal(&key),
                MetalSubgenre::ProgressiveMetal => MetalMarkovPresets::progressive_metal(&key),
            };
            
            let mut notes = Vec::with_capacity(length);
            for _ in 0..length {
                notes.push(markov.next_note());
            }
            notes
        }
    }

    /// Generate thrash metal rhythms
    fn generate_thrash_rhythms(&self, length: usize, section: MetalSection) -> Vec<RhythmPattern> {
        let _ = section;
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let mut i = 0;
        
        while i < length {
            if matches!(section, MetalSection::Verse) && rng.gen_bool(0.4) {
                if i + 2 < length {
                    rhythms.push(RhythmPattern::Gallop);
                    i += 1;
                } else {
                    rhythms.push(RhythmPattern::SixteenthNote);
                    i += 1;
                }
            } else if rng.gen_bool(0.2) {
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

    /// Generate death metal rhythms
    fn generate_death_rhythms(&self, length: usize, _section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let mut i = 0;
        
        while i < length {
            if rng.gen_bool(0.3) {
                let burst_len = rng.gen_range(4..=8);
                for _ in 0..burst_len.min(length - i) {
                    rhythms.push(RhythmPattern::ThirtySecondNote);
                    i += 1;
                }
                if i < length && rng.gen_bool(0.2) {
                    rhythms.push(RhythmPattern::Rest);
                    i += 1;
                }
            } else if rng.gen_bool(0.05) {
                rhythms.push(RhythmPattern::Rest);
                i += 1;
            } else {
                rhythms.push(RhythmPattern::SixteenthNote);
                i += 1;
            }
        }
        
        rhythms
    }

    /// Generate doom metal rhythms
    fn generate_doom_rhythms(&self, length: usize, _section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let mut i = 0;
        
        while i < length {
            if rng.gen_bool(0.2) {
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

    /// Generate heavy metal rhythms
    fn generate_heavy_rhythms(&self, length: usize, section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let rest_prob = match section {
            MetalSection::Verse => 0.15,
            MetalSection::Chorus => 0.05,
            MetalSection::Intro => 0.2,
            _ => 0.1,
        };
        
        for i in 0..length {
            if rng.gen_bool(rest_prob) {
                rhythms.push(RhythmPattern::Rest);
            } else if i % 4 == 0 {
                rhythms.push(RhythmPattern::EighthNote);
            } else {
                rhythms.push(RhythmPattern::SixteenthNote);
            }
        }
        
        rhythms
    }

    /// Generate progressive metal rhythms using Euclidean rhythms
    /// Research: Polymetric and Euclidean patterns are essential for Djent/Progressive metal
    fn generate_progressive_rhythms(&self, length: usize, _section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        
        // Use Euclidean rhythm with prime numbers for interesting syncopation
        let pulses = if length >= 16 { 7 } else { 5 }; // Prime numbers create better patterns
        let euclidean_pattern = euclidean_rhythm(length, pulses);
        
        // Rotate the pattern for variety
        let rotation = rng.gen_range(0..length);
        let rotated_pattern = rotate_rhythm(&euclidean_pattern, rotation);
        
        // Convert boolean pattern to rhythm patterns
        let mut rhythms = Vec::with_capacity(length);
        for (i, &hit) in rotated_pattern.iter().enumerate() {
            if !hit {
                rhythms.push(RhythmPattern::Rest);
            } else if i % 4 == 0 {
                // Strong beats: eighth notes
                rhythms.push(RhythmPattern::EighthNote);
            } else {
                // Weak beats: sixteenth notes
                rhythms.push(RhythmPattern::SixteenthNote);
            }
        }
        
        rhythms
    }

    /// Apply variation to an existing riff (IRVD framework - Variation)
    /// Research Section 4: Variation through rhythm changes and chromatic mutations
    fn apply_variation(&self, original_riff: &MetalRiff, intensity: f32) -> MetalRiff {
        let mut rng = rand::thread_rng();
        let mut varied_riff = original_riff.clone();
        
        // Randomly alter some notes (based on intensity)
        for i in 0..varied_riff.notes.len() {
            if rng.gen::<f32>() < intensity {
                // Apply chromatic mutation (±1 semitone)
                let mutation = if rng.gen_bool(0.5) { 1 } else { -1 };
                varied_riff.notes[i] = (varied_riff.notes[i] as i8 + mutation).max(20).min(127) as u8;
            }
        }
        
        // Slightly vary rhythm patterns (lower probability)
        for i in 0..varied_riff.rhythms.len() {
            if rng.gen::<f32>() < intensity * 0.5 {
                // Switch between similar rhythms
                varied_riff.rhythms[i] = match varied_riff.rhythms[i] {
                    RhythmPattern::SixteenthNote => if rng.gen_bool(0.5) { RhythmPattern::EighthNote } else { RhythmPattern::SixteenthNote },
                    RhythmPattern::EighthNote => if rng.gen_bool(0.5) { RhythmPattern::SixteenthNote } else { RhythmPattern::QuarterNote },
                    RhythmPattern::QuarterNote => if rng.gen_bool(0.5) { RhythmPattern::EighthNote } else { RhythmPattern::QuarterNote },
                    other => other,
                };
            }
        }
        
        // Occasionally flip palm muting (adds dynamics)
        for i in 0..varied_riff.palm_muted.len() {
            if rng.gen::<f32>() < intensity * 0.3 {
                varied_riff.palm_muted[i] = !varied_riff.palm_muted[i];
            }
        }
        
        varied_riff
    }


    /// Generate a motif-based riff with chromatic mutations
    fn generate_motif_based_riff(&self, section: MetalSection) -> MetalRiff {
        let mut rng = rand::thread_rng();
        let root = self.key.root;
        
        // Select random motif
        let motif = self.motif_library.random_motif();
        
        // Apply motif to root note
        let base_notes = motif.apply(root);
        
        // Apply chromatic mutations for dissonance
        let mutated_notes = self.chromatic_mutator.apply_mutations(base_notes);
        
        // Use aggressive pathfinding if available
        let _fret_positions = if matches!(self.subgenre, MetalSubgenre::ProgressiveMetal) {
            self.aggressive_pathfinder.find_aggressive_path(&mutated_notes)
        } else {
            self.aggressive_pathfinder.find_playable_path(&mutated_notes)
        };
        
        // Build riff from mutated notes
        self.build_riff_from_notes(mutated_notes, section)
    }

    /// Generate a breakdown riff with syncopated silences and dotted-eighth stabs
    /// PHASE 2: BREAKDOWN VIOLENCE - Drop kick sync, palm mute stutter, silence gaps
    fn generate_breakdown_riff(&self) -> MetalRiff {
        let root = self.key.root;
        let mut rng = rand::thread_rng();
        
        // Generate breakdown pattern with syncopated silences
        let pattern = self.breakdown_generator.generate_breakdown_pattern(root, 2);
        
        let mut notes = Vec::new();
        let mut rhythms = Vec::new();
        let mut palm_muted = Vec::new();
        let mut chord_types = Vec::new();
        
        for (_pos, note, _duration_mult, is_silent) in pattern {
            if !is_silent {
                notes.push(note);
                rhythms.push(RhythmPattern::DottedEighth); // Dotted eighth stabs
                palm_muted.push(true); // Heavy palm muting
                chord_types.push(ChordType::Power); // Power chords
            }
        }
        
        // Ensure we have at least some notes
        if notes.is_empty() {
            notes = vec![root; 4];
            rhythms = vec![RhythmPattern::QuarterNote; 4];
            palm_muted = vec![true; 4];
            chord_types = vec![ChordType::Power; 4];
        }
        
        // ===== PHASE 2.1: DROP KICK SYNC =====
        // CRITICAL: Ensure first note is ALWAYS the root for the drop kick
        // This guarantees the kick drum and guitar hit together for maximum impact
        if !notes.is_empty() && notes[0] != root {
            notes[0] = root;
            rhythms[0] = RhythmPattern::QuarterNote; // Hold the drop
            palm_muted[0] = true; // Palm muted for tightness
        }
        
        // ===== PHASE 2.2: PALM MUTE STUTTER =====
        // Add rapid 32nd-note palm mute stutters (70% chance)
        if rng.gen_bool(0.7) && notes.len() >= 6 {
            let stutter_start = notes.len() / 2;
            let stutter_length = rng.gen_range(3..=5);
            
            for i in 0..stutter_length {
                let idx = stutter_start + i;
                if idx < notes.len() {
                    notes[idx] = root; // All root for consistency
                    rhythms[idx] = RhythmPattern::SixteenthNote; // Rapid stutter
                    palm_muted[idx] = true; // 100% palm muted
                }
            }
        }
        
        // ===== PHASE 2.3: SILENCE GAPS =====
        // Insert intentional silence gaps (50% chance)
        // This creates "breathing room" before the next brutal hit
        if rng.gen_bool(0.5) && notes.len() >= 4 {
            // Replace random notes with silence (represented by very low velocity later)
            let gap_count = rng.gen_range(1..=2);
            for _ in 0..gap_count {
                let gap_idx = rng.gen_range(1..notes.len()); // Never gap the first note (the drop)
                if gap_idx < notes.len() {
                    // We'll mark this as a rest by using a special rhythm
                    rhythms[gap_idx] = RhythmPattern::QuarterNote; // Silence duration
                    // Note: The renderer will need to handle this as silence
                }
            }
        }
        
        // Add "burst + rest" pattern (50% chance)
        // This creates 4 fast kicks followed by silence for maximum impact
        if rng.gen_bool(0.5) && notes.len() >= 4 {
            // Replace middle section with burst pattern
            let burst_start = notes.len() / 3;
            let burst_end = (burst_start + 4).min(notes.len());
            
            for i in burst_start..burst_end {
                if i < notes.len() {
                    notes[i] = root; // All root notes for consistency
                    rhythms[i] = RhythmPattern::SixteenthNote; // Fast burst
                    palm_muted[i] = true;
                    chord_types[i] = ChordType::Power;
                }
            }
            
            // Add rest after burst (if space allows)
            if burst_end < notes.len() {
                rhythms[burst_end] = RhythmPattern::Rest;
            }
        }
        
        // ===== PHASE 2.4: INTENTIONAL OVER-CHUGGING =====
        // Add extra palm-muted chugs for maximum brutality (80% chance)
        if rng.gen_bool(0.8) {
            let extra_chugs = rng.gen_range(2..=4);
            for _ in 0..extra_chugs {
                notes.push(root);
                rhythms.push(RhythmPattern::EighthNote); // Chug rhythm
                palm_muted.push(true);
                chord_types.push(ChordType::Power);
            }
        }
        
        MetalRiff {
            notes,
            rhythms,
            palm_muted,
            chord_types,
            playability_score: 0.5, // Breakdowns are intentionally brutal
        }
    }
}