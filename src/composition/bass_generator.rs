use crate::composition::{
    music_theory::{Key, MidiNote},
    tuning::GuitarTuning,
};
use rand::Rng;

/// Bass line generator for metal music
/// Follows the guitar riff but simplified and lower
pub struct MetalBassGenerator {
    tuning: GuitarTuning,
}

impl MetalBassGenerator {
    /// Create a new bass generator
    pub fn new(tuning: GuitarTuning) -> Self {
        MetalBassGenerator { tuning }
    }

    /// Generate a bass line from a guitar riff
    /// Bass typically follows the root notes and emphasizes the pedal point
    pub fn generate_from_guitar_riff(&self, guitar_notes: &[MidiNote], key: &Key) -> Vec<MidiNote> {
        let mut bass_line = Vec::with_capacity(guitar_notes.len());
        let root = key.root;
        
        for &note in guitar_notes {
            // Bass follows guitar but simplified
            let bass_note = self.simplify_to_bass(note, root);
            bass_line.push(bass_note);
        }

        bass_line
    }

    /// Simplify a guitar note to a bass note
    /// Bass emphasizes root notes and power chord roots
    fn simplify_to_bass(&self, guitar_note: MidiNote, root: MidiNote) -> MidiNote {
        let mut rng = rand::thread_rng();
        
        // Calculate the note within the octave
        let note_class = guitar_note % 12;
        let root_class = root % 12;
        
        // Bass register: one octave below guitar (E2=40 → E1=28)
        // If it's the root note, play it in bass register
        if note_class == root_class {
            // Transpose root down to bass register
            if root >= 12 {
                root - 12
            } else {
                root
            }
        } else if rng.gen_bool(0.7) {
            // 70% chance: Play root (pedal point emphasis)
            if root >= 12 {
                root - 12
            } else {
                root
            }
        } else {
            // 30% chance: Follow guitar but in bass register
            if guitar_note >= 12 {
                guitar_note - 12
            } else {
                guitar_note
            }
        }
    }

    /// Generate a bass line with unison doubling
    /// For extra heaviness, bass can play in unison with guitar
    pub fn generate_unison_bass(&self, guitar_notes: &[MidiNote]) -> Vec<MidiNote> {
        guitar_notes.iter()
            .map(|&note| {
                // Transpose down one octave for bass
                if note >= 12 {
                    note - 12
                } else {
                    note
                }
            })
            .collect()
    }

    /// Generate a walking bass line (for progressive/doom sections)
    pub fn generate_walking_bass(&self, key: &Key, length: usize) -> Vec<MidiNote> {
        let mut bass_line = Vec::with_capacity(length);
        let scale = key.get_scale_notes();
        let root = key.root;
        
        // Start on root
        let mut current_note = 28 + (root % 12); // Bass register
        bass_line.push(current_note);
        
        let mut rng = rand::thread_rng();
        
        for _ in 1..length {
            // Walk up or down the scale
            let direction = if rng.gen_bool(0.5) { 1 } else { -1 };
            
            // Find next scale note
            let current_class = current_note % 12;
            let next_class = scale.iter()
                .filter(|&&n| {
                    let n_class = n % 12;
                    if direction > 0 {
                        n_class > current_class
                    } else {
                        n_class < current_class
                    }
                })
                .next()
                .copied()
                .unwrap_or(root);
            
            current_note = 28 + (next_class % 12);
            bass_line.push(current_note);
        }
        
        bass_line
    }

    /// Generate a breakdown bass line (slow, heavy, root-focused)
    pub fn generate_breakdown_bass(&self, root: MidiNote, length: usize) -> Vec<MidiNote> {
        // Transpose root to bass register (one octave down)
        let bass_root = if root >= 12 { root - 12 } else { root };
        
        // Breakdown: mostly root notes with occasional fifth
        let fifth = bass_root + 7;
        let mut bass_line = Vec::with_capacity(length);
        let mut rng = rand::thread_rng();
        
        for i in 0..length {
            if i % 4 == 0 {
                // Every 4th note, might play fifth
                if rng.gen_bool(0.3) {
                    bass_line.push(fifth);
                } else {
                    bass_line.push(bass_root);
                }
            } else {
                // Otherwise, play root
                bass_line.push(bass_root);
            }
        }
        
        bass_line
    }

    /// Determine if bass should use unison doubling based on tuning
    pub fn should_use_unison(&self) -> bool {
        self.tuning.bass_should_use_unison()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::music_theory::ScaleType;

    #[test]
    fn test_bass_generator_creation() {
        let generator = MetalBassGenerator::new(GuitarTuning::DStandard);
        // Just verify it was created successfully
        assert_eq!(generator.tuning, GuitarTuning::DStandard);
    }

    #[test]
    fn test_generate_from_guitar_riff() {
        let generator = MetalBassGenerator::new(GuitarTuning::EStandard);
        let key = Key {
            root: 40, // E
            scale_type: ScaleType::Phrygian,
        };
        
        let guitar_notes = vec![40, 41, 43, 40]; // E, F, G, E
        let bass_line = generator.generate_from_guitar_riff(&guitar_notes, &key);
        
        assert_eq!(bass_line.len(), 4);
        
        // Bass notes should be in bass register (around 28-40)
        for &note in &bass_line {
            assert!(note >= 24 && note <= 48);
        }
    }

    #[test]
    fn test_generate_unison_bass() {
        let generator = MetalBassGenerator::new(GuitarTuning::DStandard);
        let guitar_notes = vec![40, 42, 43];
        let bass_line = generator.generate_unison_bass(&guitar_notes);
        
        assert_eq!(bass_line.len(), 3);
        assert_eq!(bass_line[0], 28); // One octave down
        assert_eq!(bass_line[1], 30);
        assert_eq!(bass_line[2], 31);
    }

    #[test]
    fn test_generate_walking_bass() {
        let generator = MetalBassGenerator::new(GuitarTuning::EStandard);
        let key = Key {
            root: 40,
            scale_type: ScaleType::Dorian,
        };
        
        let bass_line = generator.generate_walking_bass(&key, 8);
        
        assert_eq!(bass_line.len(), 8);
        
        // Should be in bass register
        for &note in &bass_line {
            assert!(note >= 24 && note <= 48);
        }
    }

    #[test]
    fn test_generate_breakdown_bass() {
        let generator = MetalBassGenerator::new(GuitarTuning::CStandard);
        let bass_line = generator.generate_breakdown_bass(36, 16); // C (MIDI 36 → bass C = 24)
        
        assert_eq!(bass_line.len(), 16);
        
        // Should mostly be root notes (bass C = 24)
        let bass_root = 24; // 36 - 12
        let root_count = bass_line.iter().filter(|&&n| n == bass_root).count();
        assert!(root_count >= 12); // At least 75% root notes
    }

    #[test]
    fn test_simplify_to_bass() {
        let generator = MetalBassGenerator::new(GuitarTuning::EStandard);
        let root = 40; // E
        
        // Test root note simplification
        let bass_note = generator.simplify_to_bass(40, root);
        assert_eq!(bass_note % 12, root % 12);
        
        // Bass note should be in bass register
        assert!(bass_note >= 24 && bass_note <= 48);
    }

    #[test]
    fn test_bass_follows_guitar() {
        let generator = MetalBassGenerator::new(GuitarTuning::EStandard);
        let key = Key {
            root: 40,
            scale_type: ScaleType::Phrygian,
        };
        
        let guitar_notes = vec![40, 40, 40, 40]; // All root notes
        let bass_line = generator.generate_from_guitar_riff(&guitar_notes, &key);
        
        // When guitar plays all root notes, bass should too
        for &note in &bass_line {
            assert_eq!(note % 12, 40 % 12); // Same note class as root
        }
    }
}
