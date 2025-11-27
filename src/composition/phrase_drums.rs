use crate::composition::music_theory::MidiNote;
use crate::composition::metal_song_generator::MetalRiff;
use rand::Rng;

/// Cymbal type selection based on intensity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CymbalType {
    HiHat,
    Ride,
    Crash,
    China,
}

/// A single drum hit with timing and velocity
#[derive(Debug, Clone)]
pub struct DrumHit {
    pub position: usize,  // Position in 16th notes
    pub velocity: u8,
    pub is_accent: bool,
}

/// Guitar context extracted from a riff
#[derive(Debug, Clone)]
pub struct GuitarContext {
    pub palm_mute_density: f32,      // 0.0 to 1.0
    pub riff_contour: Vec<i8>,       // Pitch deltas between notes
    pub interval_stress: Vec<bool>,  // True for dissonant intervals
    pub bar_accents: Vec<usize>,     // Strong beat positions
    pub note_count: usize,
}

impl GuitarContext {
    /// Extract context from a MetalRiff
    pub fn from_riff(riff: &MetalRiff) -> Self {
        let palm_mute_density = if riff.palm_muted.is_empty() {
            0.0
        } else {
            riff.palm_muted.iter().filter(|&&pm| pm).count() as f32 / riff.palm_muted.len() as f32
        };

        // Calculate pitch deltas (contour)
        let mut riff_contour = Vec::new();
        for i in 1..riff.notes.len() {
            let delta = (riff.notes[i] as i16 - riff.notes[i-1] as i16) as i8;
            riff_contour.push(delta);
        }

        // Identify stress points (large intervals, tritones)
        let mut interval_stress = vec![false]; // First note has no stress
        for &delta in &riff_contour {
            let abs_delta = delta.abs();
            // Stress on: tritone (6), large jumps (>7), minor 2nd (1)
            let is_stress = abs_delta == 6 || abs_delta > 7 || abs_delta == 1;
            interval_stress.push(is_stress);
        }

        // Identify bar accents (every 4th 16th note = downbeats)
        let mut bar_accents = Vec::new();
        for i in (0..riff.notes.len()).step_by(4) {
            bar_accents.push(i);
        }

        GuitarContext {
            palm_mute_density,
            riff_contour,
            interval_stress,
            bar_accents,
            note_count: riff.notes.len(),
        }
    }
}

/// Phrase-aware drum generator that reacts to guitar context
pub struct PhraseAwareDrumGenerator {
    pub sample_rate: u32,
    pub bpm: u16,
}

impl PhraseAwareDrumGenerator {
    pub fn new(sample_rate: u32, bpm: u16) -> Self {
        PhraseAwareDrumGenerator {
            sample_rate,
            bpm,
        }
    }

    /// Sync kick accents with guitar palm-mute chugs
    pub fn accent_with_chugs(&self, context: &GuitarContext) -> Vec<DrumHit> {
        let mut hits = Vec::new();
        let mut rng = rand::thread_rng();

        // High palm-mute density = more kick accents
        let accent_probability = context.palm_mute_density * 0.8;

        for (i, &is_accent_pos) in context.bar_accents.iter().enumerate() {
            if is_accent_pos < context.note_count {
                let should_accent = rng.gen_bool(accent_probability as f64);
                
                hits.push(DrumHit {
                    position: is_accent_pos,
                    velocity: if should_accent { 120 } else { 100 },
                    is_accent: should_accent,
                });
            }
        }

        hits
    }

    /// Generate fill before riff transition
    pub fn fill_before_transition(&self, next_bar_intensity: f32) -> Vec<DrumHit> {
        let mut hits = Vec::new();
        let mut rng = rand::thread_rng();

        // Fill density based on next bar intensity
        let fill_notes = (4.0 + next_bar_intensity * 8.0) as usize;

        for i in 0..fill_notes {
            let position = 12 + i; // Last 4 16th notes
            let velocity = 90 + rng.gen_range(0..20);
            
            hits.push(DrumHit {
                position,
                velocity,
                is_accent: i % 2 == 0,
            });
        }

        hits
    }

    /// Trigger blast beat on harmonic tension
    pub fn blast_on_tension(&self, context: &GuitarContext) -> Vec<DrumHit> {
        let mut hits = Vec::new();

        // Count stress points
        let stress_count = context.interval_stress.iter().filter(|&&s| s).count();
        let stress_ratio = stress_count as f32 / context.interval_stress.len().max(1) as f32;

        // High stress = blast beat
        if stress_ratio > 0.3 {
            // Generate 16th note blast pattern
            for i in 0..16 {
                hits.push(DrumHit {
                    position: i,
                    velocity: 110 + (i % 2) as u8 * 10, // Alternating velocities
                    is_accent: i % 4 == 0,
                });
            }
        }

        hits
    }

    /// Choose cymbal based on intensity
    pub fn choose_cymbal(&self, intensity: f32) -> CymbalType {
        match intensity {
            i if i < 0.3 => CymbalType::HiHat,
            i if i < 0.6 => CymbalType::Ride,
            i if i < 0.85 => CymbalType::Crash,
            _ => CymbalType::China,
        }
    }

    /// Generate complete drum pattern reacting to guitar phrase
    pub fn generate_reactive_pattern(&self, context: &GuitarContext, intensity: f32) -> Vec<DrumHit> {
        let mut pattern = Vec::new();
        let mut rng = rand::thread_rng();

        // Base kick pattern synced with palm mutes
        let kick_hits = self.accent_with_chugs(context);
        pattern.extend(kick_hits);

        // Add blast beats on tension
        if rng.gen_bool(0.3) {
            let blast_hits = self.blast_on_tension(context);
            pattern.extend(blast_hits);
        }

        // Add fill if transitioning
        if rng.gen_bool(0.2) {
            let fill = self.fill_before_transition(intensity);
            pattern.extend(fill);
        }

        // Sort by position
        pattern.sort_by_key(|hit| hit.position);

        pattern
    }

    /// React to riff contour with snare placement
    pub fn snare_follows_contour(&self, context: &GuitarContext) -> Vec<DrumHit> {
        let mut hits = Vec::new();

        for (i, &delta) in context.riff_contour.iter().enumerate() {
            // Snare on ascending phrases
            if delta > 0 && i % 2 == 1 {
                hits.push(DrumHit {
                    position: i + 1,
                    velocity: 105,
                    is_accent: delta > 5,
                });
            }
        }

        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::metal_song_generator::{ChordType, RhythmPattern};

    #[test]
    fn test_guitar_context_extraction() {
        let riff = MetalRiff {
            notes: vec![60, 62, 64, 65],
            palm_muted: vec![true, true, false, false],
            chord_types: vec![ChordType::Power; 4],
            rhythms: vec![RhythmPattern::QuarterNote; 4],
            playability_score: 0.8,
        };

        let context = GuitarContext::from_riff(&riff);
        
        assert_eq!(context.palm_mute_density, 0.5);
        assert_eq!(context.riff_contour.len(), 3);
        assert_eq!(context.note_count, 4);
    }

    #[test]
    fn test_cymbal_selection() {
        let gen = PhraseAwareDrumGenerator::new(44100, 140);
        
        assert_eq!(gen.choose_cymbal(0.2), CymbalType::HiHat);
        assert_eq!(gen.choose_cymbal(0.5), CymbalType::Ride);
        assert_eq!(gen.choose_cymbal(0.8), CymbalType::Crash);
        assert_eq!(gen.choose_cymbal(0.95), CymbalType::China);
    }

    #[test]
    fn test_accent_with_chugs() {
        let gen = PhraseAwareDrumGenerator::new(44100, 140);
        let context = GuitarContext {
            palm_mute_density: 0.8,
            riff_contour: vec![],
            interval_stress: vec![],
            bar_accents: vec![0, 4, 8, 12],
            note_count: 16,
        };

        let hits = gen.accent_with_chugs(&context);
        assert!(!hits.is_empty());
    }
}
