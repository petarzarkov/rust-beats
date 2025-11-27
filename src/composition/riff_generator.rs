use crate::composition::music_theory::{Key, MidiNote};
use rand::Rng;
use std::collections::HashMap;

/// Markov chain for pitch transitions in metal riffs
/// Based on research: models common interval progressions in metal music
#[derive(Debug, Clone)]
pub struct MarkovChain {
    /// Transition probabilities: current_note -> (next_note, probability)
    transitions: HashMap<u8, Vec<(u8, f32)>>,
    /// Current state (MIDI note)
    current_state: u8,
}

impl MarkovChain {
    /// Create a new Markov chain with a starting note
    pub fn new(starting_note: u8) -> Self {
        MarkovChain {
            transitions: HashMap::new(),
            current_state: starting_note,
        }
    }

    /// Add a transition probability from one note to another
    pub fn add_transition(&mut self, from: u8, to: u8, probability: f32) {
        self.transitions
            .entry(from)
            .or_insert_with(Vec::new)
            .push((to, probability));
    }

    /// Get the next note based on transition probabilities
    pub fn next_note(&mut self) -> u8 {
        let mut rng = rand::thread_rng();
        
        if let Some(transitions) = self.transitions.get(&self.current_state) {
            // Normalize probabilities
            let total: f32 = transitions.iter().map(|(_, p)| p).sum();
            let mut random = rng.gen::<f32>() * total;
            
            for (note, prob) in transitions {
                random -= prob;
                if random <= 0.0 {
                    self.current_state = *note;
                    return *note;
                }
            }
        }
        
        // If no transitions defined, stay on current note
        self.current_state
    }

    /// Reset to a specific note
    pub fn reset(&mut self, note: u8) {
        self.current_state = note;
    }
}

/// Metal-specific Markov chain presets
pub struct MetalMarkovPresets;

impl MetalMarkovPresets {
    /// Create a heavy metal / thrash metal transition matrix
    /// Emphasizes minor seconds (b2) and power intervals (P4, P5)
    pub fn heavy_metal(key: &Key) -> MarkovChain {
        let root = key.root;
        let scale = key.get_scale_notes();
        
        let mut chain = MarkovChain::new(root);
        
        // For each scale degree, define transitions
        for (i, &note) in scale.iter().enumerate() {
            // High probability to root (pedal point)
            chain.add_transition(note, root, 0.4);
            
            // Minor second movement (characteristic of metal)
            if i + 1 < scale.len() {
                chain.add_transition(note, scale[i + 1], 0.25);
            }
            
            // Perfect fourth (power chord)
            if i + 3 < scale.len() {
                chain.add_transition(note, scale[i + 3], 0.2);
            }
            
            // Perfect fifth (power chord)
            if i + 4 < scale.len() {
                chain.add_transition(note, scale[i + 4], 0.15);
            }
        }
        
        chain
    }

    /// Create a death metal transition matrix
    /// Emphasizes chromatic movement and dissonance
    pub fn death_metal(key: &Key) -> MarkovChain {
        let root = key.root;
        let scale = key.get_scale_notes();
        
        let mut chain = MarkovChain::new(root);
        
        for (i, &note) in scale.iter().enumerate() {
            // Very high probability to root (brutal chugging)
            chain.add_transition(note, root, 0.5);
            
            // Chromatic movement (half-step)
            if i + 1 < scale.len() {
                chain.add_transition(note, scale[i + 1], 0.3);
            }
            
            // Tritone (diabolus in musica)
            if i + 6 < scale.len() {
                chain.add_transition(note, scale[i + 6], 0.2);
            }
        }
        
        chain
    }

    /// Create a progressive metal / djent transition matrix
    /// Emphasizes larger intervals and syncopation
    pub fn progressive_metal(key: &Key) -> MarkovChain {
        let root = key.root;
        let scale = key.get_scale_notes();
        
        let mut chain = MarkovChain::new(root);
        
        for (i, &note) in scale.iter().enumerate() {
            // Moderate probability to root
            chain.add_transition(note, root, 0.3);
            
            // Major/minor thirds
            if i + 2 < scale.len() {
                chain.add_transition(note, scale[i + 2], 0.25);
            }
            
            // Perfect fifths
            if i + 4 < scale.len() {
                chain.add_transition(note, scale[i + 4], 0.25);
            }
            
            // Octave jumps
            if i + 7 < scale.len() {
                chain.add_transition(note, scale[i + 7], 0.2);
            }
        }
        
        chain
    }
}

/// Chromatic mutation system for breaking free from scale-locking
/// Applies chromatic bends, tritone substitutions, and interval enforcement
#[derive(Debug, Clone)]
pub struct ChromaticMutator {
    pub bend_probability: f32,      // Probability of bending note ±1 semitone
    pub tritone_probability: f32,   // Probability of tritone substitution
    pub chromatic_run_probability: f32, // Probability of inserting chromatic run
    pub interval_enforcement: f32,  // Probability of forcing dissonant intervals
}

impl ChromaticMutator {
    pub fn new(mutation_intensity: f32) -> Self {
        ChromaticMutator {
            bend_probability: mutation_intensity * 0.15,
            tritone_probability: mutation_intensity * 0.10,
            chromatic_run_probability: mutation_intensity * 0.20,
            interval_enforcement: mutation_intensity * 0.20,
        }
    }

    /// Apply all mutations to a note sequence
    pub fn apply_mutations(&self, mut notes: Vec<MidiNote>) -> Vec<MidiNote> {
        // Apply bends
        notes = self.bend_notes(notes);
        
        // Apply tritone substitutions
        notes = self.tritone_substitute(notes);
        
        // Insert chromatic runs
        notes = self.insert_chromatic_runs(notes);
        
        // Enforce dissonant intervals
        notes = self.enforce_intervals(notes);
        
        notes
    }

    /// Bend notes ±1 semitone with probability
    fn bend_notes(&self, notes: Vec<MidiNote>) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        notes.iter()
            .map(|&note| {
                if rng.gen_bool(self.bend_probability as f64) {
                    let bend = if rng.gen_bool(0.5) { 1 } else { -1 };
                    ((note as i16 + bend).clamp(0, 127)) as MidiNote
                } else {
                    note
                }
            })
            .collect()
    }

    /// Replace notes with tritone (+6 semitones) with probability
    fn tritone_substitute(&self, notes: Vec<MidiNote>) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        notes.iter()
            .map(|&note| {
                if rng.gen_bool(self.tritone_probability as f64) {
                    ((note as i16 + 6).clamp(0, 127)) as MidiNote
                } else {
                    note
                }
            })
            .collect()
    }

    /// Insert 2-4 note chromatic passages every 4-8 notes
    fn insert_chromatic_runs(&self, notes: Vec<MidiNote>) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        let mut result = Vec::new();
        let mut i = 0;

        while i < notes.len() {
            result.push(notes[i]);
            
            // Check if we should insert a chromatic run
            if i > 0 && i % rng.gen_range(4..=8) == 0 && rng.gen_bool(self.chromatic_run_probability as f64) {
                let run_length = rng.gen_range(2..=4);
                let start_note = notes[i];
                let direction = if rng.gen_bool(0.5) { 1 } else { -1 };
                
                for j in 1..=run_length {
                    let chromatic_note = ((start_note as i16 + (j as i16 * direction)).clamp(0, 127)) as MidiNote;
                    result.push(chromatic_note);
                }
            }
            
            i += 1;
        }

        result
    }

    /// Enforce dissonant intervals (m2, m3, tritone) with probability
    fn enforce_intervals(&self, notes: Vec<MidiNote>) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        let mut result = Vec::new();

        for (i, &note) in notes.iter().enumerate() {
            result.push(note);
            
            if i > 0 && rng.gen_bool(self.interval_enforcement as f64) {
                let last_idx = result.len() - 1;
                let prev_idx = result.len() - 2;
                let prev_note = result[prev_idx];
                let interval = (note as i16 - prev_note as i16).abs();
                
                // If interval is too consonant (perfect 4th, 5th, octave), make it dissonant
                if interval == 5 || interval == 7 || interval == 12 {
                    let dissonant_intervals = vec![1, 3, 6]; // m2, m3, tritone
                    let chosen_interval = dissonant_intervals[rng.gen_range(0..dissonant_intervals.len())];
                    let new_note = ((prev_note as i16 + chosen_interval).clamp(0, 127)) as MidiNote;
                    result[last_idx] = new_note;
                }
            }
        }

        result
    }
}


/// Pedal point riff generator
/// Based on research: most metal riffs return to a static bass note (pedal)
#[derive(Debug, Clone)]
pub struct PedalPointGenerator {
    pub pedal_note: MidiNote,
    pub melodic_pool: Vec<MidiNote>,
    pub return_probability: f32, // Probability of returning to pedal
}

impl PedalPointGenerator {
    /// Create a new pedal point generator
    pub fn new(pedal_note: MidiNote, melodic_pool: Vec<MidiNote>) -> Self {
        PedalPointGenerator {
            pedal_note,
            melodic_pool,
            return_probability: 0.65, // Default: 65% chance to return to pedal
        }
    }

    /// Create from a key (uses root as pedal, scale notes as melodic pool)
    pub fn from_key(key: &Key) -> Self {
        let pedal_note = key.root;
        let melodic_pool = key.get_scale_notes();
        
        PedalPointGenerator::new(pedal_note, melodic_pool)
    }

    /// Generate a sequence of notes with pedal point technique
    pub fn generate_sequence(&self, length: usize) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        let mut sequence = Vec::with_capacity(length);
        let mut on_pedal = true;
        
        for _ in 0..length {
            if on_pedal {
                // Play pedal note
                sequence.push(self.pedal_note);
                // Decide whether to move to melodic note
                on_pedal = rng.gen::<f32>() > 0.7; // 30% chance to stay on pedal
            } else {
                // Play melodic note
                let idx = rng.gen_range(0..self.melodic_pool.len());
                sequence.push(self.melodic_pool[idx]);
                // Decide whether to return to pedal
                on_pedal = rng.gen::<f32>() < self.return_probability;
            }
        }
        
        sequence
    }

    /// Generate a riff pattern with specified pedal/melodic ratio
    /// Returns (note, is_palm_muted)
    pub fn generate_riff_pattern(&self, bars: usize, notes_per_bar: usize) -> Vec<(MidiNote, bool)> {
        let total_notes = bars * notes_per_bar;
        let sequence = self.generate_sequence(total_notes);
        
        // Tag pedal notes as palm muted, melodic notes as open
        sequence.iter()
            .map(|&note| {
                let is_palm_mute = note == self.pedal_note;
                (note, is_palm_mute)
            })
            .collect()
    }
}

/// Riff structure following IRVD framework
/// Introduction, Repetition, Variation, Destruction
#[derive(Debug, Clone)]
pub struct RiffStructure {
    pub intro: Vec<MidiNote>,
    pub main_riff: Vec<MidiNote>,
    pub variation: Vec<MidiNote>,
    pub breakdown: Vec<MidiNote>,
}

impl RiffStructure {
    /// Generate a complete riff structure using pedal point and Markov chains
    pub fn generate(key: &Key, style: MetalStyle) -> Self {
        let pedal_gen = PedalPointGenerator::from_key(key);
        
        // Introduction: Simple pedal pattern
        let intro = pedal_gen.generate_sequence(8);
        
        // Main riff: Longer sequence with variation
        let main_riff = pedal_gen.generate_sequence(16);
        
        // Variation: Use Markov chain for more melodic movement
        let mut markov = match style {
            MetalStyle::HeavyMetal => MetalMarkovPresets::heavy_metal(key),
            MetalStyle::DeathMetal => MetalMarkovPresets::death_metal(key),
            MetalStyle::Progressive => MetalMarkovPresets::progressive_metal(key),
        };
        
        let mut variation = Vec::with_capacity(16);
        for _ in 0..16 {
            variation.push(markov.next_note());
        }
        
        // Breakdown: All pedal notes (brutal chugging)
        let breakdown = vec![pedal_gen.pedal_note; 8];
        
        RiffStructure {
            intro,
            main_riff,
            variation,
            breakdown,
        }
    }
}

/// Metal subgenre styles for riff generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetalStyle {
    HeavyMetal,
    DeathMetal,
    Progressive,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::music_theory::ScaleType;

    #[test]
    fn test_markov_chain_creation() {
        let mut chain = MarkovChain::new(60);
        chain.add_transition(60, 62, 0.5);
        chain.add_transition(60, 64, 0.5);
        
        assert_eq!(chain.current_state, 60);
    }

    #[test]
    fn test_markov_chain_next_note() {
        let mut chain = MarkovChain::new(60);
        chain.add_transition(60, 62, 1.0); // 100% probability
        
        let next = chain.next_note();
        assert_eq!(next, 62);
        assert_eq!(chain.current_state, 62);
    }

    #[test]
    fn test_pedal_point_generator() {
        let pedal = 40; // E2
        let melodic_pool = vec![40, 41, 43, 45, 47];
        let gen = PedalPointGenerator::new(pedal, melodic_pool);
        
        let sequence = gen.generate_sequence(16);
        assert_eq!(sequence.len(), 16);
        
        // Should contain the pedal note
        assert!(sequence.contains(&pedal));
    }

    #[test]
    fn test_pedal_point_from_key() {
        let key = Key {
            root: 40,
            scale_type: ScaleType::Phrygian,
        };
        let gen = PedalPointGenerator::from_key(&key);
        
        assert_eq!(gen.pedal_note, 40);
        assert!(!gen.melodic_pool.is_empty());
    }

    #[test]
    fn test_riff_pattern_generation() {
        let key = Key {
            root: 40,
            scale_type: ScaleType::Phrygian,
        };
        let gen = PedalPointGenerator::from_key(&key);
        
        let pattern = gen.generate_riff_pattern(2, 8); // 2 bars, 8 notes each
        assert_eq!(pattern.len(), 16);
        
        // Check that pedal notes are marked as palm muted
        for (note, is_palm_mute) in pattern {
            if note == gen.pedal_note {
                assert!(is_palm_mute);
            }
        }
    }

    #[test]
    fn test_metal_markov_presets() {
        let key = Key {
            root: 40,
            scale_type: ScaleType::Phrygian,
        };
        
        let heavy = MetalMarkovPresets::heavy_metal(&key);
        assert_eq!(heavy.current_state, 40);
        
        let death = MetalMarkovPresets::death_metal(&key);
        assert_eq!(death.current_state, 40);
        
        let prog = MetalMarkovPresets::progressive_metal(&key);
        assert_eq!(prog.current_state, 40);
    }

    #[test]
    fn test_riff_structure_generation() {
        let key = Key {
            root: 40,
            scale_type: ScaleType::Phrygian,
        };
        let structure = RiffStructure::generate(&key, MetalStyle::HeavyMetal);
        
        assert_eq!(structure.intro.len(), 8);
        assert_eq!(structure.main_riff.len(), 16);
        assert_eq!(structure.variation.len(), 16);
        assert_eq!(structure.breakdown.len(), 8);
        
        // Breakdown should be all pedal notes
        assert!(structure.breakdown.iter().all(|&n| n == 40));
    }

    #[test]
    fn test_metal_styles() {
        let key = Key {
            root: 40,
            scale_type: ScaleType::Phrygian,
        };
        
        let heavy_structure = RiffStructure::generate(&key, MetalStyle::HeavyMetal);
        let death_structure = RiffStructure::generate(&key, MetalStyle::DeathMetal);
        let prog_structure = RiffStructure::generate(&key, MetalStyle::Progressive);
        
        // All should generate valid structures
        assert!(!heavy_structure.intro.is_empty());
        assert!(!death_structure.intro.is_empty());
        assert!(!prog_structure.intro.is_empty());
    }
}
