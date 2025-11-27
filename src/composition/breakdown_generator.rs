use crate::composition::music_theory::MidiNote;
use rand::Rng;

/// Breakdown transformation types for aggressive metal breakdowns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakdownTransform {
    /// Syncopated silences (unexpected gaps)
    SyncopatedSilence,
    /// Metric modulation (tempo feel changes)
    MetricModulation,
    /// Dotted-eighth stabs (3/16 note attacks)
    DottedEighthStab,
    /// Random silence injection
    RandomSilence,
}

/// Generator for aggressive breakdown patterns
pub struct BreakdownGenerator {
    pub silence_probability: f32,     // Probability of inserting silence
    pub modulation_intensity: f32,    // How much to shift the feel
    pub stab_density: f32,            // Density of dotted-eighth stabs
}

impl BreakdownGenerator {
    /// Create a new breakdown generator with default settings
    pub fn new() -> Self {
        BreakdownGenerator {
            silence_probability: 0.3,
            modulation_intensity: 0.5,
            stab_density: 0.4,
        }
    }

    /// Create an aggressive breakdown generator
    pub fn aggressive() -> Self {
        BreakdownGenerator {
            silence_probability: 0.5,
            modulation_intensity: 0.8,
            stab_density: 0.6,
        }
    }

    /// Apply syncopated silences to a note pattern
    /// Returns (note, is_silent) pairs
    pub fn apply_syncopated_silences(&self, notes: &[MidiNote]) -> Vec<(MidiNote, bool)> {
        let mut rng = rand::thread_rng();
        let mut result = Vec::new();

        for (i, &note) in notes.iter().enumerate() {
            // Don't silence the first note (need strong downbeat)
            if i == 0 {
                result.push((note, false));
                continue;
            }

            // Higher probability of silence on off-beats
            let is_offbeat = i % 2 == 1;
            let silence_chance = if is_offbeat {
                self.silence_probability * 1.5
            } else {
                self.silence_probability * 0.5
            };

            let is_silent = rng.gen_bool(silence_chance as f64);
            result.push((note, is_silent));
        }

        result
    }

    /// Generate dotted-eighth stab pattern
    /// Returns positions in 16th notes where stabs occur
    pub fn generate_dotted_eighth_stabs(&self, bars: usize) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let total_sixteenths = bars * 16;
        let mut positions = Vec::new();

        // Dotted eighth = 3 sixteenth notes
        let mut pos = 0;
        while pos < total_sixteenths {
            if rng.gen_bool(self.stab_density as f64) {
                positions.push(pos);
                pos += 3; // Dotted eighth spacing
            } else {
                pos += 4; // Quarter note spacing
            }
        }

        positions
    }

    /// Apply metric modulation to create tempo feel changes
    /// Returns a multiplier for note durations
    pub fn metric_modulation_multiplier(&self, position: usize, total_length: usize) -> f32 {
        let progress = position as f32 / total_length as f32;
        
        // Create a wave that modulates the feel
        let modulation = (progress * std::f32::consts::PI * 2.0).sin() * self.modulation_intensity;
        
        // Return multiplier: 1.0 = normal, <1.0 = faster feel, >1.0 = slower feel
        1.0 + modulation * 0.3
    }

    /// Inject random silences into a pattern
    /// Returns indices where silences should occur
    pub fn random_silence_positions(&self, pattern_length: usize) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let mut silences = Vec::new();

        for i in 1..pattern_length { // Skip first position
            if rng.gen_bool((self.silence_probability * 0.7) as f64) {
                silences.push(i);
            }
        }

        silences
    }

    /// Generate a complete breakdown pattern with all transformations
    /// Returns (position, note, duration_multiplier, is_silent)
    pub fn generate_breakdown_pattern(
        &self,
        root_note: MidiNote,
        bars: usize,
    ) -> Vec<(usize, MidiNote, f32, bool)> {
        let mut rng = rand::thread_rng();
        let stab_positions = self.generate_dotted_eighth_stabs(bars);
        let total_sixteenths = bars * 16;
        
        let mut pattern = Vec::new();

        for &pos in &stab_positions {
            if pos >= total_sixteenths {
                break;
            }

            // Vary the note slightly (octave or fifth)
            let note_variation = rng.gen_range(0..3);
            let note = match note_variation {
                0 => root_note,                    // Root
                1 => root_note.saturating_add(7),  // Fifth
                _ => root_note.saturating_sub(12), // Octave down
            };

            // Apply metric modulation
            let duration_mult = self.metric_modulation_multiplier(pos, total_sixteenths);

            // Check if this should be silent (random silence)
            let is_silent = rng.gen_bool((self.silence_probability * 0.3) as f64);

            pattern.push((pos, note, duration_mult, is_silent));
        }

        pattern
    }
}

impl Default for BreakdownGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Breakdown pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakdownPattern {
    /// Standard breakdown (quarter note chugs)
    Standard,
    /// Syncopated (off-beat accents)
    Syncopated,
    /// Dotted rhythm (dotted eighth stabs)
    DottedRhythm,
    /// Chaotic (random silences and modulation)
    Chaotic,
}

impl BreakdownPattern {
    /// Get the appropriate generator settings for this pattern
    pub fn generator(&self) -> BreakdownGenerator {
        match self {
            BreakdownPattern::Standard => BreakdownGenerator {
                silence_probability: 0.1,
                modulation_intensity: 0.2,
                stab_density: 0.3,
            },
            BreakdownPattern::Syncopated => BreakdownGenerator {
                silence_probability: 0.4,
                modulation_intensity: 0.4,
                stab_density: 0.5,
            },
            BreakdownPattern::DottedRhythm => BreakdownGenerator {
                silence_probability: 0.2,
                modulation_intensity: 0.3,
                stab_density: 0.8,
            },
            BreakdownPattern::Chaotic => BreakdownGenerator::aggressive(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syncopated_silences() {
        let gen = BreakdownGenerator::new();
        let notes = vec![60, 62, 64, 65, 67];
        let result = gen.apply_syncopated_silences(&notes);
        
        assert_eq!(result.len(), notes.len());
        assert!(!result[0].1); // First note should not be silent
    }

    #[test]
    fn test_dotted_eighth_stabs() {
        let gen = BreakdownGenerator::new();
        let stabs = gen.generate_dotted_eighth_stabs(2);
        
        // Should have some stabs
        assert!(!stabs.is_empty());
        
        // All positions should be within 2 bars (32 sixteenths)
        for &pos in &stabs {
            assert!(pos < 32);
        }
    }

    #[test]
    fn test_metric_modulation() {
        let gen = BreakdownGenerator::new();
        let mult = gen.metric_modulation_multiplier(0, 100);
        
        // Should return a reasonable multiplier
        assert!(mult > 0.5 && mult < 1.5);
    }

    #[test]
    fn test_breakdown_pattern_generators() {
        let standard = BreakdownPattern::Standard.generator();
        let chaotic = BreakdownPattern::Chaotic.generator();
        
        // Chaotic should have higher probabilities
        assert!(chaotic.silence_probability > standard.silence_probability);
        assert!(chaotic.stab_density > standard.stab_density);
    }
}
