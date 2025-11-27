use crate::composition::music_theory::{Key, MidiNote};
use rand::Rng;

/// A riff motif: a 2-4 note pattern defined by interval relationships
/// Intervals are relative to the root note (in semitones)
#[derive(Debug, Clone)]
pub struct RiffMotif {
    pub name: String,
    pub intervals: Vec<i8>, // Relative intervals from root (e.g., [0, 6] = root + tritone)
    pub rhythm_density: f32, // 0.0 = slow, 1.0 = fast (tremolo)
}

impl RiffMotif {
    pub fn new(name: &str, intervals: Vec<i8>, rhythm_density: f32) -> Self {
        RiffMotif {
            name: name.to_string(),
            intervals,
            rhythm_density,
        }
    }

    /// Apply this motif starting from a root note
    pub fn apply(&self, root: MidiNote) -> Vec<MidiNote> {
        self.intervals
            .iter()
            .map(|&interval| {
                let note = root as i16 + interval as i16;
                note.clamp(0, 127) as MidiNote
            })
            .collect()
    }

    /// Apply with micro-variation (transpose, reverse, etc.)
    pub fn apply_with_variation(&self, root: MidiNote, variation: MotifVariation) -> Vec<MidiNote> {
        let mut notes = self.apply(root);
        
        match variation {
            MotifVariation::None => notes,
            MotifVariation::Transpose(semitones) => {
                notes.iter()
                    .map(|&n| ((n as i16 + semitones as i16).clamp(0, 127)) as MidiNote)
                    .collect()
            }
            MotifVariation::Reverse => {
                notes.reverse();
                notes
            }
            MotifVariation::DoubleSpeed => {
                // Repeat each note (simulates faster rhythm)
                notes.iter().flat_map(|&n| vec![n, n]).collect()
            }
            MotifVariation::HalfSpeed => {
                // Take every other note
                notes.iter().step_by(2).copied().collect()
            }
        }
    }
}

/// Micro-variations that can be applied to motifs
#[derive(Debug, Clone, Copy)]
pub enum MotifVariation {
    None,
    Transpose(i8),  // Transpose by N semitones
    Reverse,        // Reverse the motif
    DoubleSpeed,    // Double rhythm density
    HalfSpeed,      // Half rhythm density
}

/// Library of metal-specific riff motifs
pub struct MotifLibrary {
    motifs: Vec<RiffMotif>,
}

impl MotifLibrary {
    /// Create a new motif library with standard metal motifs
    pub fn new() -> Self {
        let mut motifs = Vec::new();

        // Tritone slide: Root → b5
        motifs.push(RiffMotif::new(
            "Tritone Slide",
            vec![0, 6], // Root, tritone
            0.5,
        ));

        // Chromatic passing: Root → +1 → +2
        motifs.push(RiffMotif::new(
            "Chromatic Passing",
            vec![0, 1, 2],
            0.7,
        ));

        // Hammer-on cluster: Root → +2 → +3 → +5
        motifs.push(RiffMotif::new(
            "Hammer-On Cluster",
            vec![0, 2, 3, 5],
            0.8,
        ));

        // Tremolo burst: Rapid repetition
        motifs.push(RiffMotif::new(
            "Tremolo Burst",
            vec![0, 0, 0, 0], // Same note repeated
            1.0,
        ));

        // Dissonant stab: Root + b2 (minor second)
        motifs.push(RiffMotif::new(
            "Dissonant Stab b2",
            vec![0, 1], // Root, minor second
            0.3,
        ));

        // Dissonant stab: Root + b6 (minor sixth)
        motifs.push(RiffMotif::new(
            "Dissonant Stab b6",
            vec![0, 8], // Root, minor sixth
            0.3,
        ));

        // Chromatic descent: 4-note chromatic run down
        motifs.push(RiffMotif::new(
            "Chromatic Descent",
            vec![0, -1, -2, -3],
            0.6,
        ));

        // Power interval jump: Root → P5 → Octave
        motifs.push(RiffMotif::new(
            "Power Jump",
            vec![0, 7, 12],
            0.4,
        ));

        // Minor third hammer: Root → b3 → Root
        motifs.push(RiffMotif::new(
            "Minor Third Hammer",
            vec![0, 3, 0],
            0.7,
        ));

        // Whole-half diminished: Root → +2 → +3
        motifs.push(RiffMotif::new(
            "Whole-Half Diminished",
            vec![0, 2, 3],
            0.5,
        ));

        MotifLibrary { motifs }
    }

    /// Get a random motif
    pub fn random_motif(&self) -> &RiffMotif {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.motifs.len());
        &self.motifs[idx]
    }

    /// Get a motif by name
    pub fn get_motif(&self, name: &str) -> Option<&RiffMotif> {
        self.motifs.iter().find(|m| m.name == name)
    }

    /// Get all motifs with high rhythm density (for fast sections)
    pub fn fast_motifs(&self) -> Vec<&RiffMotif> {
        self.motifs.iter().filter(|m| m.rhythm_density > 0.6).collect()
    }

    /// Get all motifs with low rhythm density (for heavy sections)
    pub fn heavy_motifs(&self) -> Vec<&RiffMotif> {
        self.motifs.iter().filter(|m| m.rhythm_density < 0.5).collect()
    }
}

impl Default for MotifLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Recombines motifs with micro-variations to create complete riffs
pub struct MotifRecombinator {
    library: MotifLibrary,
    variation_probability: f32, // Probability of applying variation per motif
}

impl MotifRecombinator {
    pub fn new(variation_probability: f32) -> Self {
        MotifRecombinator {
            library: MotifLibrary::new(),
            variation_probability,
        }
    }

    /// Generate a riff by recombining motifs
    /// Returns a sequence of notes
    pub fn generate_riff(&self, key: &Key, num_motifs: usize, prefer_fast: bool) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        let mut notes = Vec::new();

        let motif_pool = if prefer_fast {
            self.library.fast_motifs()
        } else {
            self.library.heavy_motifs()
        };

        if motif_pool.is_empty() {
            // Fallback to all motifs
            return self.generate_riff_fallback(key, num_motifs);
        }

        for _ in 0..num_motifs {
            // Pick a random motif from the pool
            let motif_idx = rng.gen_range(0..motif_pool.len());
            let motif = motif_pool[motif_idx];

            // Decide if we apply variation
            let variation = if rng.gen_bool(self.variation_probability as f64) {
                match rng.gen_range(0..4) {
                    0 => MotifVariation::Transpose(rng.gen_range(-2..=2)),
                    1 => MotifVariation::Reverse,
                    2 => MotifVariation::DoubleSpeed,
                    _ => MotifVariation::HalfSpeed,
                }
            } else {
                MotifVariation::None
            };

            // Apply motif to root note
            let motif_notes = motif.apply_with_variation(key.root, variation);
            notes.extend(motif_notes);
        }

        notes
    }

    /// Fallback: use all motifs if pool is empty
    fn generate_riff_fallback(&self, key: &Key, num_motifs: usize) -> Vec<MidiNote> {
        let mut rng = rand::thread_rng();
        let mut notes = Vec::new();

        for _ in 0..num_motifs {
            let motif = self.library.random_motif();
            let variation = if rng.gen_bool(self.variation_probability as f64) {
                MotifVariation::Transpose(rng.gen_range(-2..=2))
            } else {
                MotifVariation::None
            };
            let motif_notes = motif.apply_with_variation(key.root, variation);
            notes.extend(motif_notes);
        }

        notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::music_theory::ScaleType;

    #[test]
    fn test_motif_creation() {
        let motif = RiffMotif::new("Test", vec![0, 6], 0.5);
        assert_eq!(motif.name, "Test");
        assert_eq!(motif.intervals, vec![0, 6]);
        assert_eq!(motif.rhythm_density, 0.5);
    }

    #[test]
    fn test_motif_apply() {
        let motif = RiffMotif::new("Tritone", vec![0, 6], 0.5);
        let notes = motif.apply(40); // E2
        assert_eq!(notes, vec![40, 46]); // E2, Bb2
    }

    #[test]
    fn test_motif_transpose() {
        let motif = RiffMotif::new("Test", vec![0, 2], 0.5);
        let notes = motif.apply_with_variation(40, MotifVariation::Transpose(1));
        assert_eq!(notes, vec![41, 43]); // Transposed up 1 semitone
    }

    #[test]
    fn test_motif_reverse() {
        let motif = RiffMotif::new("Test", vec![0, 2, 4], 0.5);
        let notes = motif.apply_with_variation(40, MotifVariation::Reverse);
        assert_eq!(notes, vec![44, 42, 40]); // Reversed
    }

    #[test]
    fn test_library_creation() {
        let library = MotifLibrary::new();
        assert!(!library.motifs.is_empty());
    }

    #[test]
    fn test_library_get_motif() {
        let library = MotifLibrary::new();
        let motif = library.get_motif("Tritone Slide");
        assert!(motif.is_some());
        assert_eq!(motif.unwrap().intervals, vec![0, 6]);
    }

    #[test]
    fn test_library_fast_motifs() {
        let library = MotifLibrary::new();
        let fast = library.fast_motifs();
        assert!(!fast.is_empty());
        for motif in fast {
            assert!(motif.rhythm_density > 0.6);
        }
    }

    #[test]
    fn test_recombinator() {
        let recombinator = MotifRecombinator::new(0.5);
        let key = Key {
            root: 40,
            scale_type: ScaleType::Phrygian,
        };
        let riff = recombinator.generate_riff(&key, 4, false);
        assert!(!riff.is_empty());
    }
}
