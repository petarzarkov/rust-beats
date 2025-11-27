use rand::Rng;

/// Drum articulation types for realistic metal drumming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Articulation {
    GhostNote,  // Low velocity (20-40)
    Normal,     // Medium velocity (80-100)
    Accent,     // High velocity (110-127)
    Flam,       // Two hits 5ms apart
    Drag,       // Buzz roll into hit
}

impl Articulation {
    /// Get base velocity for this articulation
    pub fn base_velocity(&self) -> u8 {
        match self {
            Articulation::GhostNote => 30,
            Articulation::Normal => 90,
            Articulation::Accent => 115,
            Articulation::Flam => 100,
            Articulation::Drag => 95,
        }
    }

    /// Check if this articulation requires multiple hits
    pub fn is_multi_hit(&self) -> bool {
        matches!(self, Articulation::Flam | Articulation::Drag)
    }
}

/// Cymbal interaction patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CymbalPattern {
    Crash,
    CrashStack,  // Crash + China simultaneous
    Ride,
    China,
    Splash,
}

/// Hi-hat state for open/closed patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HiHatState {
    Closed,
    Open,
    HalfOpen,
}

/// Hi-hat pattern generator
pub struct HiHatPattern {
    pattern: Vec<HiHatState>,
    current_index: usize,
}

impl HiHatPattern {
    /// Create alternating closed/open pattern
    pub fn alternating(length: usize) -> Self {
        let mut pattern = Vec::with_capacity(length);
        for i in 0..length {
            pattern.push(if i % 2 == 0 {
                HiHatState::Closed
            } else {
                HiHatState::Open
            });
        }
        HiHatPattern {
            pattern,
            current_index: 0,
        }
    }

    /// Create mostly closed with occasional opens
    pub fn mostly_closed(length: usize, open_probability: f32) -> Self {
        let mut rng = rand::thread_rng();
        let mut pattern = Vec::with_capacity(length);
        for _ in 0..length {
            pattern.push(if rng.gen_bool(open_probability as f64) {
                HiHatState::Open
            } else {
                HiHatState::Closed
            });
        }
        HiHatPattern {
            pattern,
            current_index: 0,
        }
    }

    /// Get next hi-hat state
    pub fn next(&mut self) -> HiHatState {
        if self.pattern.is_empty() {
            return HiHatState::Closed;
        }
        let state = self.pattern[self.current_index];
        self.current_index = (self.current_index + 1) % self.pattern.len();
        state
    }

    /// Reset to beginning
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

/// Snare ghost note pattern generator
pub struct SnareGhostPattern {
    ghost_interval: usize, // Every Nth hit is a ghost note
}

impl SnareGhostPattern {
    /// Create a ghost pattern (every Nth hit is ghost)
    pub fn new(ghost_interval: usize) -> Self {
        SnareGhostPattern { ghost_interval }
    }

    /// Check if hit at this index should be a ghost note
    pub fn is_ghost(&self, index: usize) -> bool {
        if self.ghost_interval == 0 {
            return false;
        }
        index % self.ghost_interval != 0
    }

    /// Get articulation for this hit
    pub fn get_articulation(&self, index: usize, is_accent: bool) -> Articulation {
        if is_accent {
            Articulation::Accent
        } else if self.is_ghost(index) {
            Articulation::GhostNote
        } else {
            Articulation::Normal
        }
    }
}

/// Stamina model for realistic velocity decay over time
#[derive(Debug, Clone)]
pub struct StaminaModel {
    kick_hits: usize,
    snare_hits: usize,
    hihat_hits: usize,
    decay_rate: f32, // Velocity decay per 16 hits
    min_velocity: u8,
}

impl StaminaModel {
    pub fn new(decay_rate: f32, min_velocity: u8) -> Self {
        StaminaModel {
            kick_hits: 0,
            snare_hits: 0,
            hihat_hits: 0,
            decay_rate,
            min_velocity,
        }
    }

    /// Record a hit and return velocity with stamina decay
    pub fn hit_kick(&mut self, base_velocity: u8) -> u8 {
        self.kick_hits += 1;
        self.apply_decay(base_velocity, self.kick_hits)
    }

    pub fn hit_snare(&mut self, base_velocity: u8) -> u8 {
        self.snare_hits += 1;
        self.apply_decay(base_velocity, self.snare_hits)
    }

    pub fn hit_hihat(&mut self, base_velocity: u8) -> u8 {
        self.hihat_hits += 1;
        self.apply_decay(base_velocity, self.hihat_hits)
    }

    /// Apply stamina decay based on hit count
    fn apply_decay(&self, base_velocity: u8, hit_count: usize) -> u8 {
        let decay_amount = (hit_count / 16) as f32 * self.decay_rate;
        let decayed = base_velocity as f32 - decay_amount;
        decayed.max(self.min_velocity as f32) as u8
    }

    /// Reset stamina (after a rest)
    pub fn reset(&mut self) {
        self.kick_hits = 0;
        self.snare_hits = 0;
        self.hihat_hits = 0;
    }

    /// Reset specific limb
    pub fn reset_kick(&mut self) {
        self.kick_hits = 0;
    }

    pub fn reset_snare(&mut self) {
        self.snare_hits = 0;
    }

    pub fn reset_hihat(&mut self) {
        self.hihat_hits = 0;
    }
}

/// Limb imbalance model (right hand stronger than left)
#[derive(Debug, Clone)]
pub struct LimbImbalanceModel {
    right_hand_bonus: i8,  // Velocity bonus for right hand
    left_hand_penalty: i8, // Velocity penalty for left hand
    kick_rebound_variation: i8, // Alternating kick velocity variation
}

impl LimbImbalanceModel {
    pub fn new() -> Self {
        LimbImbalanceModel {
            right_hand_bonus: 5,
            left_hand_penalty: -5,
            kick_rebound_variation: 3,
        }
    }

    /// Apply right hand velocity (stronger)
    pub fn right_hand(&self, base_velocity: u8) -> u8 {
        (base_velocity as i16 + self.right_hand_bonus as i16).clamp(0, 127) as u8
    }

    /// Apply left hand velocity (weaker)
    pub fn left_hand(&self, base_velocity: u8) -> u8 {
        (base_velocity as i16 + self.left_hand_penalty as i16).clamp(0, 127) as u8
    }

    /// Apply kick pedal rebound (alternates)
    pub fn kick_pedal(&self, base_velocity: u8, is_even_hit: bool) -> u8 {
        let variation = if is_even_hit {
            self.kick_rebound_variation
        } else {
            -self.kick_rebound_variation
        };
        (base_velocity as i16 + variation as i16).clamp(0, 127) as u8
    }
}

impl Default for LimbImbalanceModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Deterministic drum pattern generator with articulations
pub struct DrumArticulationGenerator {
    ghost_pattern: SnareGhostPattern,
    hihat_pattern: HiHatPattern,
    stamina: StaminaModel,
    limb_imbalance: LimbImbalanceModel,
    kick_hit_count: usize,
}

impl DrumArticulationGenerator {
    pub fn new() -> Self {
        DrumArticulationGenerator {
            ghost_pattern: SnareGhostPattern::new(4), // Every 4th hit is ghost
            hihat_pattern: HiHatPattern::alternating(8),
            stamina: StaminaModel::new(2.0, 70), // Decay 2 velocity per 16 hits, min 70
            limb_imbalance: LimbImbalanceModel::new(),
            kick_hit_count: 0,
        }
    }

    /// Generate snare hit with articulation
    pub fn snare_hit(&mut self, index: usize, is_accent: bool) -> (u8, Articulation) {
        let articulation = self.ghost_pattern.get_articulation(index, is_accent);
        let base_velocity = articulation.base_velocity();
        
        // Apply limb imbalance (assume right hand on snare)
        let velocity_with_limb = self.limb_imbalance.right_hand(base_velocity);
        
        // Apply stamina decay
        let final_velocity = self.stamina.hit_snare(velocity_with_limb);
        
        (final_velocity, articulation)
    }

    /// Generate kick hit with rebound variation
    pub fn kick_hit(&mut self, base_velocity: u8) -> u8 {
        let is_even = self.kick_hit_count % 2 == 0;
        self.kick_hit_count += 1;
        
        // Apply rebound variation
        let velocity_with_rebound = self.limb_imbalance.kick_pedal(base_velocity, is_even);
        
        // Apply stamina decay
        self.stamina.hit_kick(velocity_with_rebound)
    }

    /// Generate hi-hat hit with open/closed state
    pub fn hihat_hit(&mut self, base_velocity: u8) -> (u8, HiHatState) {
        let state = self.hihat_pattern.next();
        
        // Apply limb imbalance (assume left hand on hi-hat)
        let velocity_with_limb = self.limb_imbalance.left_hand(base_velocity);
        
        // Apply stamina decay
        let final_velocity = self.stamina.hit_hihat(velocity_with_limb);
        
        (final_velocity, state)
    }

    /// Reset stamina after a rest (>1 beat silence)
    pub fn rest(&mut self) {
        self.stamina.reset();
    }

    /// Check if we should add a flam (25% probability on beat 4)
    pub fn should_flam(&self, beat_position: usize) -> bool {
        let mut rng = rand::thread_rng();
        beat_position % 4 == 3 && rng.gen_bool(0.25)
    }

    /// Generate kick roll (4x 32nd notes before snare)
    pub fn kick_roll(&mut self, base_velocity: u8) -> Vec<u8> {
        (0..4).map(|_| self.kick_hit(base_velocity - 10)).collect()
    }
}

impl Default for DrumArticulationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_articulation_velocity() {
        assert_eq!(Articulation::GhostNote.base_velocity(), 30);
        assert_eq!(Articulation::Normal.base_velocity(), 90);
        assert_eq!(Articulation::Accent.base_velocity(), 115);
    }

    #[test]
    fn test_hihat_alternating() {
        let mut pattern = HiHatPattern::alternating(4);
        assert_eq!(pattern.next(), HiHatState::Closed);
        assert_eq!(pattern.next(), HiHatState::Open);
        assert_eq!(pattern.next(), HiHatState::Closed);
        assert_eq!(pattern.next(), HiHatState::Open);
    }

    #[test]
    fn test_ghost_pattern() {
        let pattern = SnareGhostPattern::new(4);
        assert!(!pattern.is_ghost(0)); // First hit is not ghost
        assert!(pattern.is_ghost(1));  // Second hit is ghost
        assert!(pattern.is_ghost(2));
        assert!(pattern.is_ghost(3));
        assert!(!pattern.is_ghost(4)); // Fifth hit is not ghost
    }

    #[test]
    fn test_stamina_decay() {
        let mut stamina = StaminaModel::new(2.0, 70);
        
        // First 16 hits should be at base velocity
        for _ in 0..16 {
            assert_eq!(stamina.hit_snare(100), 100);
        }
        
        // After 16 hits, should decay by 2
        assert_eq!(stamina.hit_snare(100), 98);
    }

    #[test]
    fn test_stamina_reset() {
        let mut stamina = StaminaModel::new(2.0, 70);
        
        // Generate 20 hits
        for _ in 0..20 {
            stamina.hit_snare(100);
        }
        
        // Reset
        stamina.reset();
        
        // Should be back to base velocity
        assert_eq!(stamina.hit_snare(100), 100);
    }

    #[test]
    fn test_limb_imbalance() {
        let limb = LimbImbalanceModel::new();
        
        assert_eq!(limb.right_hand(100), 105); // +5 bonus
        assert_eq!(limb.left_hand(100), 95);   // -5 penalty
    }

    #[test]
    fn test_kick_rebound() {
        let limb = LimbImbalanceModel::new();
        
        assert_eq!(limb.kick_pedal(100, true), 103);  // Even hit: +3
        assert_eq!(limb.kick_pedal(100, false), 97);  // Odd hit: -3
    }

    #[test]
    fn test_drum_articulation_generator() {
        let mut gen = DrumArticulationGenerator::new();
        
        // Generate snare hits
        let (vel1, art1) = gen.snare_hit(0, false);
        assert!(vel1 > 0);
        assert_eq!(art1, Articulation::Normal);
        
        let (vel2, art2) = gen.snare_hit(1, false);
        assert!(vel2 > 0);
        assert_eq!(art2, Articulation::GhostNote);
    }

    #[test]
    fn test_kick_roll() {
        let mut gen = DrumArticulationGenerator::new();
        let roll = gen.kick_roll(100);
        assert_eq!(roll.len(), 4);
        assert!(roll.iter().all(|&v| v > 0));
    }
}
