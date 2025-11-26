use rand::Rng;

/// Drum humanization for realistic metal drum programming
/// Based on research: avoids robotic "machine gun" effect

/// Velocity range for MIDI (0-127)
pub type Velocity = u8;

/// Timing offset in samples or ticks
pub type TimingOffset = i32;

/// Drum humanizer for realistic velocity and timing variations
#[derive(Debug, Clone)]
pub struct DrumHumanizer {
    /// Velocity randomization range (±)
    pub velocity_variance: u8,
    /// Timing randomization range in ticks (±)
    pub timing_variance: i32,
    /// Timing bias (positive = drag/late, negative = rush/early)
    pub timing_bias: i32,
    /// Accent probability (0.0-1.0) for emphasizing certain beats
    pub accent_probability: f32,
    /// Accent velocity boost
    pub accent_boost: u8,
}

impl DrumHumanizer {
    /// Create a new drum humanizer with default settings
    pub fn new() -> Self {
        DrumHumanizer {
            velocity_variance: 5,      // ±5 for round-robin simulation
            timing_variance: 10,        // ±10 ticks
            timing_bias: 0,             // No bias (on the grid)
            accent_probability: 0.15,   // 15% chance of accent
            accent_boost: 15,           // +15 velocity for accents
        }
    }

    /// Preset for blast beats (lower velocity, tighter timing)
    pub fn blast_beat() -> Self {
        DrumHumanizer {
            velocity_variance: 3,       // Tighter variance for fast playing
            timing_variance: 5,         // Very tight timing
            timing_bias: -5,            // Slightly rushed (urgency)
            accent_probability: 0.25,   // Accent first beat of measure
            accent_boost: 12,
        }
    }

    /// Preset for breakdowns (heavier, dragged timing)
    pub fn breakdown() -> Self {
        DrumHumanizer {
            velocity_variance: 8,       // More variation for impact
            timing_variance: 15,        // Looser timing
            timing_bias: 10,            // Dragged (weight/sludge)
            accent_probability: 0.3,    // More accents for impact
            accent_boost: 20,
        }
    }

    /// Preset for thrash metal (rushed, aggressive)
    pub fn thrash() -> Self {
        DrumHumanizer {
            velocity_variance: 6,
            timing_variance: 12,
            timing_bias: -8,            // Rushed (frantic energy)
            accent_probability: 0.2,
            accent_boost: 18,
        }
    }

    /// Humanize a velocity value
    /// Returns a velocity with randomization and optional accent
    pub fn humanize_velocity(&self, base_velocity: Velocity, is_accent: bool) -> Velocity {
        let mut rng = rand::thread_rng();
        
        // Apply random variance (±)
        let variance = rng.gen_range(-(self.velocity_variance as i16)..=(self.velocity_variance as i16));
        let mut velocity = (base_velocity as i16 + variance).clamp(1, 127) as u8;
        
        // Apply accent if flagged
        if is_accent {
            velocity = (velocity as u16 + self.accent_boost as u16).min(127) as u8;
        }
        
        velocity
    }

    /// Humanize timing offset
    /// Returns timing offset in ticks
    pub fn humanize_timing(&self) -> TimingOffset {
        let mut rng = rand::thread_rng();
        
        // Apply bias + random variance
        let variance = rng.gen_range(-self.timing_variance..=self.timing_variance);
        self.timing_bias + variance
    }

    /// Check if this hit should be accented (random based on probability)
    pub fn should_accent(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < self.accent_probability
    }

    /// Humanize a drum hit (velocity + timing)
    /// Returns (humanized_velocity, timing_offset)
    pub fn humanize_hit(&self, base_velocity: Velocity, force_accent: bool) -> (Velocity, TimingOffset) {
        let is_accent = force_accent || self.should_accent();
        let velocity = self.humanize_velocity(base_velocity, is_accent);
        let timing = self.humanize_timing();
        
        (velocity, timing)
    }
}

impl Default for DrumHumanizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Blast beat pattern generator
/// Based on research: different blast beat styles for metal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlastBeatStyle {
    Traditional,  // Kick and snare simultaneous
    Hammer,       // Kick and snare unison (same as traditional)
    Euro,         // Kick and snare alternate
    Gravity,      // Uses rimshot articulations
}

/// Generate a blast beat pattern
/// Returns (kick_hits, snare_hits) as boolean arrays for each subdivision
pub fn generate_blast_beat(style: BlastBeatStyle, subdivisions: usize) -> (Vec<bool>, Vec<bool>) {
    let mut kicks = vec![false; subdivisions];
    let mut snares = vec![false; subdivisions];
    
    match style {
        BlastBeatStyle::Traditional | BlastBeatStyle::Hammer => {
            // Kick and snare hit together on every subdivision
            for i in 0..subdivisions {
                kicks[i] = true;
                snares[i] = true;
            }
        }
        BlastBeatStyle::Euro => {
            // Alternating kick-snare pattern
            for i in 0..subdivisions {
                if i % 2 == 0 {
                    kicks[i] = true;
                } else {
                    snares[i] = true;
                }
            }
        }
        BlastBeatStyle::Gravity => {
            // Similar to traditional but with rimshot articulation
            // (articulation would be handled separately in MIDI)
            for i in 0..subdivisions {
                kicks[i] = true;
                snares[i] = true;
            }
        }
    }
    
    (kicks, snares)
}

/// Velocity profile for blast beats
/// Based on research: blast beats have lower velocity due to smaller range of motion
pub fn blast_beat_velocity(base_velocity: Velocity, is_first_beat: bool) -> Velocity {
    let blast_velocity = if base_velocity > 110 {
        // Reduce overly high velocities for blast beats
        90 + (base_velocity - 110) / 3
    } else {
        base_velocity
    };
    
    // Accent the first beat of the measure
    if is_first_beat {
        (blast_velocity as u16 + 15).min(127) as u8
    } else {
        blast_velocity.clamp(85, 110) // Typical blast beat range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanizer_creation() {
        let humanizer = DrumHumanizer::new();
        assert_eq!(humanizer.velocity_variance, 5);
        assert_eq!(humanizer.timing_bias, 0);
    }

    #[test]
    fn test_blast_beat_preset() {
        let humanizer = DrumHumanizer::blast_beat();
        assert_eq!(humanizer.velocity_variance, 3);
        assert!(humanizer.timing_bias < 0); // Should be rushed
    }

    #[test]
    fn test_breakdown_preset() {
        let humanizer = DrumHumanizer::breakdown();
        assert!(humanizer.timing_bias > 0); // Should be dragged
        assert!(humanizer.velocity_variance > 5);
    }

    #[test]
    fn test_humanize_velocity() {
        let humanizer = DrumHumanizer::new();
        let base_velocity = 100;
        
        // Test without accent
        let velocity = humanizer.humanize_velocity(base_velocity, false);
        assert!(velocity >= 95 && velocity <= 105); // Within ±5 range
        
        // Test with accent
        let accented = humanizer.humanize_velocity(base_velocity, true);
        assert!(accented >= velocity); // Should be higher or equal
    }

    #[test]
    fn test_humanize_velocity_clamp() {
        let humanizer = DrumHumanizer::new();
        
        // Test lower bound
        let low = humanizer.humanize_velocity(5, false);
        assert!(low >= 1 && low <= 127);
        
        // Test upper bound
        let high = humanizer.humanize_velocity(125, true);
        assert!(high <= 127);
    }

    #[test]
    fn test_humanize_timing() {
        let humanizer = DrumHumanizer::new();
        let timing = humanizer.humanize_timing();
        
        // Should be within bias ± variance
        assert!(timing >= -10 && timing <= 10);
    }

    #[test]
    fn test_humanize_timing_with_bias() {
        let humanizer = DrumHumanizer::thrash();
        let timing = humanizer.humanize_timing();
        
        // Thrash should be rushed (negative bias)
        assert!(timing < 5); // Should tend negative
    }

    #[test]
    fn test_humanize_hit() {
        let humanizer = DrumHumanizer::new();
        let (velocity, timing) = humanizer.humanize_hit(100, false);
        
        assert!(velocity >= 1 && velocity <= 127);
        assert!(timing >= -10 && timing <= 10);
    }

    #[test]
    fn test_humanize_hit_forced_accent() {
        let humanizer = DrumHumanizer::new();
        let (velocity, _) = humanizer.humanize_hit(100, true);
        
        // Forced accent should boost velocity
        assert!(velocity >= 100);
    }

    #[test]
    fn test_blast_beat_traditional() {
        let (kicks, snares) = generate_blast_beat(BlastBeatStyle::Traditional, 8);
        
        assert_eq!(kicks.len(), 8);
        assert_eq!(snares.len(), 8);
        
        // All hits should be true (simultaneous)
        assert!(kicks.iter().all(|&x| x));
        assert!(snares.iter().all(|&x| x));
    }

    #[test]
    fn test_blast_beat_euro() {
        let (kicks, snares) = generate_blast_beat(BlastBeatStyle::Euro, 8);
        
        // Should alternate
        for i in 0..8 {
            if i % 2 == 0 {
                assert!(kicks[i]);
                assert!(!snares[i]);
            } else {
                assert!(!kicks[i]);
                assert!(snares[i]);
            }
        }
    }

    #[test]
    fn test_blast_beat_velocity() {
        // Normal velocity
        let vel1 = blast_beat_velocity(100, false);
        assert!(vel1 >= 85 && vel1 <= 110);
        
        // First beat (accented)
        let vel2 = blast_beat_velocity(100, true);
        assert!(vel2 > vel1);
        
        // High velocity should be reduced
        let vel3 = blast_beat_velocity(120, false);
        assert!(vel3 < 120);
    }

    #[test]
    fn test_velocity_randomization() {
        let humanizer = DrumHumanizer::new();
        let base = 100;
        
        // Generate multiple velocities to check variance
        let velocities: Vec<u8> = (0..20)
            .map(|_| humanizer.humanize_velocity(base, false))
            .collect();
        
        // Should have some variation (not all the same)
        let unique_count = velocities.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(unique_count > 1);
    }

    #[test]
    fn test_timing_randomization() {
        let humanizer = DrumHumanizer::new();
        
        // Generate multiple timings to check variance
        let timings: Vec<i32> = (0..20)
            .map(|_| humanizer.humanize_timing())
            .collect();
        
        // Should have some variation
        let unique_count = timings.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(unique_count > 1);
    }
}
