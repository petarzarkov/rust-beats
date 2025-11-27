use crate::composition::{
    drum_humanizer::{DrumHumanizer, BlastBeatStyle, generate_blast_beat, blast_beat_velocity},
    fretboard::{FretboardPathfinder, PlayabilityMode, calculate_playability_score},
    music_theory::{Key, ScaleType, MidiNote},
    tuning::GuitarTuning,
    rhythm::{euclidean_rhythm, rotate_rhythm, OddSubdivisionPattern, DisplacedAccentGenerator, PolymetricInterference},
    riff_generator::{MetalMarkovPresets, PedalPointGenerator, ChromaticMutator},
    riff_motifs::{RiffMotif, MotifLibrary, MotifRecombinator},
    drum_articulations::DrumArticulationGenerator,
    breakdown_generator::{BreakdownGenerator, BreakdownPattern},
    bar_memory::BarMotifStore,
    phrase_drums::{PhraseAwareDrumGenerator, GuitarContext},
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
        
        // Use motif-based generation for some riffs (40% chance)
        if section != MetalSection::Intro && rng.gen_bool(0.4) {
            return self.generate_motif_based_riff(section);
        }
        
        // Use polymetric riffs for progressive metal
        if matches!(self.subgenre, MetalSubgenre::ProgressiveMetal) && rng.gen_bool(0.3) {
            return self.generate_polymetric_riff(section);
        }
        
        // Standard generation
        match section {
            MetalSection::Intro => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_intro_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Verse => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_verse_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Chorus => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_chorus_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Breakdown => {
                // Already handled above
                self.generate_breakdown_riff()
            },
            MetalSection::Solo => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_solo_sequence(root, scale, 32);
                self.build_riff_from_notes(notes, section)
            },
            MetalSection::Outro => {
                let root = self.key.root;
                let scale = self.key.scale_type;
                let notes = self.generate_outro_sequence(root, scale, 16);
                self.build_riff_from_notes(notes, section)
            },
        }
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
        
        for (i, &note) in notes.iter().enumerate() {
            let is_pedal = note == root || note == root + 12;
            let is_strong_beat = i % 4 == 0;
            let rhythm = rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            // BREAKDOWN LOGIC: Force unison/palm mutes
            if section == MetalSection::Breakdown {
                if rhythm == RhythmPattern::Rest {
                    palm_muted.push(true);
                    chord_types.push(ChordType::Single);
                    continue;
                }
                
                // Heavy accents on strong beats
                if i % 2 == 0 {
                    palm_muted.push(false); // Open for accent
                    chord_types.push(ChordType::Power); // Power chord
                } else {
                    palm_muted.push(true); // Tight mute
                    chord_types.push(ChordType::Single);
                }
                continue;
            }

            // Normal Section Logic
            if rhythm == RhythmPattern::Rest {
                palm_muted.push(true);
                chord_types.push(ChordType::Single);
                continue;
            }
            
            // ENHANCED: More complex chord selection based on section and note position
            match section {
                MetalSection::Intro => {
                    if is_strong_beat && i % 8 == 0 {
                        palm_muted.push(false);
                        chord_types.push(if rng.gen_bool(0.6) {
                            ChordType::Octave
                        } else {
                            ChordType::Power
                        });
                    } else {
                        palm_muted.push(true);
                        chord_types.push(ChordType::Single);
                    }
                },
                MetalSection::Verse => {
                    if is_pedal {
                        palm_muted.push(true);
                        chord_types.push(ChordType::Single);
                    } else if is_strong_beat {
                        palm_muted.push(false);
                        chord_types.push(if rng.gen_bool(0.5) {
                            ChordType::Minor
                        } else {
                            ChordType::Power
                        });
                    } else {
                        palm_muted.push(true);
                        chord_types.push(ChordType::Power);
                    }
                },
                MetalSection::Chorus => {
                    palm_muted.push(false);
                    if is_strong_beat {
                        chord_types.push(if rng.gen_bool(0.4) {
                            ChordType::Minor
                        } else {
                            ChordType::Power
                        });
                    } else {
                        chord_types.push(ChordType::Power);
                    }
                },
                MetalSection::Breakdown => {
                    // This case handled above, but fallback:
                     palm_muted.push(true);
                     chord_types.push(ChordType::Power);
                },
                MetalSection::Solo => {
                    if is_strong_beat && i % 8 == 0 {
                        palm_muted.push(false);
                        chord_types.push(ChordType::Octave);
                    } else {
                        palm_muted.push(false);
                        chord_types.push(ChordType::Single);
                    }
                },
                MetalSection::Outro => {
                    palm_muted.push(true);
                    chord_types.push(if is_strong_beat {
                        ChordType::Octave
                    } else {
                        ChordType::Single
                    });
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
    fn generate_intro_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        self.generate_markov_sequence_with_pedal(root, scale, length, 0.60)
    }

    /// Generate verse sequence (palm-muted chugs, tight rhythm)
    fn generate_verse_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        self.generate_markov_sequence_with_pedal(root, scale, length, 0.50)
    }

    /// Generate chorus sequence (open power chords, melodic)
    fn generate_chorus_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        self.generate_markov_sequence_with_pedal(root, scale, length, 0.30)
    }

    /// Generate solo sequence (melodic, fast)
    fn generate_solo_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        self.generate_markov_sequence_with_pedal(root, scale, length, 0.20)
    }

    /// Generate outro sequence (fade out, simple)
    fn generate_outro_sequence(&self, root: MidiNote, scale: ScaleType, length: usize) -> Vec<MidiNote> {
        self.generate_markov_sequence_with_pedal(root, scale, length, 0.80)
    }

    /// Generate a complete metal song structure
    pub fn generate_song(&self) -> MetalSong {
        let mut sections = Vec::new();

        sections.push((MetalSection::Intro, self.generate_riff(MetalSection::Intro)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse)));
        sections.push((MetalSection::Breakdown, self.generate_riff(MetalSection::Breakdown)));
        sections.push((MetalSection::Solo, self.generate_riff(MetalSection::Solo)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Breakdown, self.generate_riff(MetalSection::Breakdown)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Outro, self.generate_riff(MetalSection::Outro)));

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

    /// Get interval weight for metal generation (DEPRECATED - use Markov chains instead)
    fn get_interval_weight(interval: u8, _scale: ScaleType, subgenre: MetalSubgenre) -> f32 {
        match interval {
            1 => 2.0,  // Minor second - very metal (b2)
            6 => 1.8,  // Tritone - devil's interval (b5)
            7 => 1.2,  // Perfect fifth
            10 => 1.1, // Minor seventh
            3 => if matches!(subgenre, MetalSubgenre::DeathMetal) { 1.0 } else { 0.3 },
            4 => 0.2,  // Major third
            2 => 0.5,  // Major second
            5 => 1.0,  // Perfect fourth
            8 => 0.8,  // Minor sixth
            9 => 0.4,  // Major sixth
            11 => 1.0, // Major seventh
            _ => 0.5,
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
                MetalSubgenre::ThrashMetal => MetalMarkovPresets::heavy_metal(&key),
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
            } else if rng.gen_bool(0.3) {
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

    /// Generate doom metal rhythms
    fn generate_doom_rhythms(&self, length: usize, _section: MetalSection) -> Vec<RhythmPattern> {
        let mut rng = rand::thread_rng();
        let mut rhythms = Vec::with_capacity(length);
        let mut i = 0;
        
        while i < length {
            if rng.gen_bool(0.5) {
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
            MetalSection::Verse => 0.35,
            MetalSection::Chorus => 0.15,
            MetalSection::Intro => 0.5,
            _ => 0.25,
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

    /// Generate drums for a section (Legacy support method)
    pub fn generate_drums(&self, section: MetalSection, humanizer: &DrumHumanizer) -> Vec<(u8, i32)> {
        let mut drum_hits = Vec::new();
        let subdivisions = match section {
            MetalSection::Breakdown => 8,
            _ => 16,
        };

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
            for i in 0..subdivisions {
                let is_accent = i % 4 == 0;
                let base_velocity = if is_accent { 110 } else { 95 };
                let (velocity, timing) = humanizer.humanize_hit(base_velocity, is_accent);
                drum_hits.push((velocity, timing));
            }
        }

        drum_hits
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
    fn generate_breakdown_riff(&self) -> MetalRiff {
        let root = self.key.root;
        
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
        
        MetalRiff {
            notes,
            palm_muted,
            chord_types,
            rhythms,
            playability_score: 0.8,
        }
    }
}