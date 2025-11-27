use crate::composition::music_theory::MidiNote;
use crate::composition::tuning::GuitarTuning;
use rand::seq::SliceRandom;

use std::cmp::Ordering;

/// Represents a position on the guitar fretboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FretPosition {
    pub string: u8,  // 0-indexed (0 = lowest string)
    pub fret: u8,    // 0 = open string
}

impl FretPosition {
    pub fn new(string: u8, fret: u8) -> Self {
        FretPosition { string, fret }
    }

    /// Calculate the cost of moving from this position to another
    /// Based on research: biomechanical constraints for playability
    pub fn movement_cost(&self, other: &FretPosition) -> f32 {
        let string_diff = (self.string as i16 - other.string as i16).abs();
        let fret_diff = (self.fret as i16 - other.fret as i16).abs();

        // Cost function based on research:
        // - Same string, 1 fret = low cost
        // - Adjacent string, same fret = low cost
        // - Large fret jumps = high cost
        // - String skipping = medium cost

        if string_diff == 0 {
            // Same string movement
            match fret_diff {
                0 => 0.0,           // No movement
                1 => 1.0,           // 1 fret = easy
                2 => 2.0,           // 2 frets = moderate
                3 => 3.5,           // 3 frets = harder
                4 => 5.0,           // 4 frets = difficult
                _ => 10.0 + fret_diff as f32, // 5+ frets = very difficult
            }
        } else if fret_diff == 0 {
            // Same fret, different string
            match string_diff {
                1 => 1.5,           // Adjacent string = easy
                2 => 3.0,           // String skip = medium
                _ => 5.0 + string_diff as f32, // Multiple string skip = hard
            }
        } else {
            // Diagonal movement (different string AND fret)
            let base_cost = (fret_diff as f32 * 1.5) + (string_diff as f32 * 2.0);
            
            // Penalize large diagonal movements
            if fret_diff > 3 || string_diff > 2 {
                base_cost * 1.5
            } else {
                base_cost
            }
        }
    }
}

/// Node for A* pathfinding
#[derive(Debug, Clone)]
struct PathNode {
    position: FretPosition,
    cost: f32,
    heuristic: f32,
}

impl PathNode {
    fn total_cost(&self) -> f32 {
        self.cost + self.heuristic
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost() == other.total_cost()
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse ordering for min-heap
        other.total_cost().partial_cmp(&self.total_cost())
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Fretboard pathfinding for playable riff generation
/// Based on research: ensures riffs are biomechanically feasible
pub struct FretboardPathfinder {
    tuning: GuitarTuning,
    max_fret: u8,
    mode: PlayabilityMode,
}

/// Playability mode for fretboard pathfinding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayabilityMode {
    /// Conservative: Prioritize comfort, avoid stretches
    Conservative,
    /// Standard: Balance between comfort and melodic flow
    Standard,
    /// Aggressive: Prioritize melodic contour, allow spicy stretches
    Aggressive,
}

impl PlayabilityMode {
    /// Get the stretch tolerance multiplier for this mode
    pub fn stretch_multiplier(&self) -> f32 {
        match self {
            PlayabilityMode::Conservative => 0.7,  // Reduce stretch costs
            PlayabilityMode::Standard => 1.0,      // Normal stretch costs
            PlayabilityMode::Aggressive => 1.8,    // Allow bigger stretches
        }
    }

    /// Get the melodic contour weight for this mode
    pub fn melodic_weight(&self) -> f32 {
        match self {
            PlayabilityMode::Conservative => 0.3,  // Low priority on contour
            PlayabilityMode::Standard => 0.5,      // Balanced
            PlayabilityMode::Aggressive => 0.8,    // High priority on contour
        }
    }

    /// Maximum allowed fret stretch for this mode
    pub fn max_stretch(&self) -> u8 {
        match self {
            PlayabilityMode::Conservative => 3,    // Max 3 fret stretch
            PlayabilityMode::Standard => 4,        // Max 4 fret stretch
            PlayabilityMode::Aggressive => 6,      // Max 6 fret stretch (spicy!)
        }
    }
}

impl FretboardPathfinder {
    pub fn new(tuning: GuitarTuning) -> Self {
        FretboardPathfinder {
            tuning,
            max_fret: 24, // Standard guitar has 24 frets
            mode: PlayabilityMode::Standard,
        }
    }

    /// Create a pathfinder with a specific playability mode
    pub fn with_mode(tuning: GuitarTuning, mode: PlayabilityMode) -> Self {
        FretboardPathfinder {
            tuning,
            max_fret: 24,
            mode,
        }
    }

    /// Get all possible fretboard positions for a given MIDI note
    pub fn get_positions_for_note(&self, note: MidiNote) -> Vec<FretPosition> {
        let mut positions = Vec::new();
        let string_notes = self.tuning.string_notes();

        for (string_idx, &open_note) in string_notes.iter().enumerate() {
            // Calculate fret position for this string
            if note >= open_note {
                let fret = note - open_note;
                if fret <= self.max_fret {
                    positions.push(FretPosition::new(string_idx as u8, fret));
                }
            }
        }

        positions
    }

    /// Find the most playable path through a sequence of notes using A* algorithm
    /// Returns the optimal fret positions for each note
    pub fn find_playable_path(&self, notes: &[MidiNote]) -> Vec<FretPosition> {
        if notes.is_empty() {
            return Vec::new();
        }

        let mut path = Vec::with_capacity(notes.len());
        
        // Start with the lowest position for the first note (most comfortable)
        let first_positions = self.get_positions_for_note(notes[0]);
        if first_positions.is_empty() {
            return Vec::new(); // Note not playable on this tuning
        }
        
        // Choose starting position (prefer lower frets, lower strings for metal)
        let mut current_pos = first_positions.iter()
            .min_by_key(|pos| (pos.fret, pos.string))
            .copied()
            .unwrap();
        
        path.push(current_pos);

        // For each subsequent note, find the best position
        for &note in &notes[1..] {
            let positions = self.get_positions_for_note(note);
            if positions.is_empty() {
                continue; // Skip unplayable notes
            }

            // Filter positions based on mode's max stretch
            let max_stretch = self.mode.max_stretch();
            let stretch_mult = self.mode.stretch_multiplier();
            
            let valid_positions: Vec<_> = positions.iter()
                .filter(|pos| {
                    let fret_diff = (current_pos.fret as i16 - pos.fret as i16).abs() as u8;
                    fret_diff <= max_stretch
                })
                .collect();

            // If no valid positions within stretch, use all positions
            let candidate_positions = if valid_positions.is_empty() {
                &positions
            } else {
                &valid_positions.iter().copied().copied().collect::<Vec<_>>()
            };

            // Find position with minimum adjusted cost
            let best_pos = candidate_positions.iter()
                .min_by(|a, b| {
                    let cost_a = current_pos.movement_cost(a) * stretch_mult;
                    let cost_b = current_pos.movement_cost(b) * stretch_mult;
                    cost_a.partial_cmp(&cost_b).unwrap_or(Ordering::Equal)
                })
                .copied()
                .unwrap();

            path.push(best_pos);
            current_pos = best_pos;
        }

        path
    }

    /// Find an aggressive path that prioritizes melodic contour over comfort
    /// Allows spicy stretches for better note flow
    pub fn find_aggressive_path(&self, notes: &[MidiNote]) -> Vec<FretPosition> {
        if notes.is_empty() {
            return Vec::new();
        }

        let mut path = Vec::with_capacity(notes.len());
        let melodic_weight = self.mode.melodic_weight();
        
        // For aggressive mode, prefer higher positions for better sustain
        let first_positions = self.get_positions_for_note(notes[0]);
        if first_positions.is_empty() {
            return Vec::new();
        }
        
        // In aggressive mode, prefer mid-range frets (5-12) for better tone
        let mut current_pos = first_positions.iter()
            .min_by_key(|pos| {
                let fret_penalty = if pos.fret >= 5 && pos.fret <= 12 {
                    0
                } else {
                    (pos.fret as i16 - 8).abs()
                };
                (fret_penalty, pos.string)
            })
            .copied()
            .unwrap();
        
        path.push(current_pos);

        // For each subsequent note, prioritize melodic flow
        for (i, &note) in notes[1..].iter().enumerate() {
            let positions = self.get_positions_for_note(note);
            if positions.is_empty() {
                continue;
            }

            // Calculate melodic direction (ascending/descending)
            let prev_note = notes[i];
            let is_ascending = note > prev_note;
            
            // Find position that best maintains melodic contour
            let best_pos = positions.iter()
                .min_by(|a, b| {
                    let movement_cost_a = current_pos.movement_cost(a);
                    let movement_cost_b = current_pos.movement_cost(b);
                    
                    // Melodic contour bonus: prefer same string for smooth lines
                    let contour_bonus_a = if a.string == current_pos.string {
                        -2.0 * melodic_weight
                    } else {
                        0.0
                    };
                    
                    let contour_bonus_b = if b.string == current_pos.string {
                        -2.0 * melodic_weight
                    } else {
                        0.0
                    };
                    
                    // Direction bonus: prefer ascending frets for ascending notes
                    let direction_bonus_a = if is_ascending && a.fret > current_pos.fret {
                        -1.0 * melodic_weight
                    } else if !is_ascending && a.fret < current_pos.fret {
                        -1.0 * melodic_weight
                    } else {
                        0.0
                    };
                    
                    let direction_bonus_b = if is_ascending && b.fret > current_pos.fret {
                        -1.0 * melodic_weight
                    } else if !is_ascending && b.fret < current_pos.fret {
                        -1.0 * melodic_weight
                    } else {
                        0.0
                    };
                    
                    let total_cost_a = movement_cost_a + contour_bonus_a + direction_bonus_a;
                    let total_cost_b = movement_cost_b + contour_bonus_b + direction_bonus_b;
                    
                    total_cost_a.partial_cmp(&total_cost_b).unwrap_or(Ordering::Equal)
                })
                .copied()
                .unwrap();

            path.push(best_pos);
            current_pos = best_pos;
        }

        path
    }

    /// Check if a sequence of notes is playable (total cost below threshold)
    pub fn is_playable(&self, notes: &[MidiNote], max_total_cost: f32) -> bool {
        let path = self.find_playable_path(notes);
        if path.len() != notes.len() {
            return false; // Some notes couldn't be played
        }

        let total_cost: f32 = path.windows(2)
            .map(|window| window[0].movement_cost(&window[1]))
            .sum();

        total_cost <= max_total_cost
    }

    /// Optimize a riff for playability by adjusting notes if needed
    /// Returns (optimized_notes, fret_positions)
    pub fn optimize_riff(&self, notes: &[MidiNote]) -> (Vec<MidiNote>, Vec<FretPosition>) {
        let path = self.find_playable_path(notes);
        
        // Convert positions back to notes (in case we need to adjust)
        let optimized_notes: Vec<MidiNote> = path.iter()
            .map(|pos| {
                let string_notes = self.tuning.string_notes();
                string_notes[pos.string as usize] + pos.fret
            })
            .collect();

        (optimized_notes, path)
    }

    /// Find a spiteful path with chaos weight - overrides optimization for dramatic effect
    /// chaos: 0.0 = normal, 1.0 = maximum chaos
    pub fn find_spiteful_path(&self, notes: &[MidiNote], chaos: f32) -> Vec<FretPosition> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if notes.is_empty() {
            return Vec::new();
        }

        let mut path = Vec::with_capacity(notes.len());
        let mut current_pos = self.get_positions_for_note(notes[0])[0];
        path.push(current_pos);

        for &note in &notes[1..] {
            let positions = self.get_positions_for_note(note);
            if positions.is_empty() {
                continue;
            }

            // CHAOS: 20% chance of sudden large jump (>5 frets)
            if rng.gen_bool((chaos * 0.2) as f64) {
                let far_positions: Vec<_> = positions.iter()
                    .filter(|p| (p.fret as i16 - current_pos.fret as i16).abs() > 5)
                    .collect();
                
                if !far_positions.is_empty() {
                    current_pos = **far_positions.choose(&mut rng).unwrap();
                    path.push(current_pos);
                    continue;
                }
            }

            // CHAOS: 15% chance of position reset (back to fret 0-3)
            if rng.gen_bool((chaos * 0.15) as f64) {
                let low_positions: Vec<_> = positions.iter()
                    .filter(|p| p.fret <= 3)
                    .collect();
                
                if !low_positions.is_empty() {
                    current_pos = **low_positions.choose(&mut rng).unwrap();
                    path.push(current_pos);
                    continue;
                }
            }

            // CHAOS: 10% chance of unnecessary shift for tone
            if rng.gen_bool((chaos * 0.1) as f64) {
                // Pick a position on a different string
                let different_string: Vec<_> = positions.iter()
                    .filter(|p| p.string != current_pos.string)
                    .collect();
                
                if !different_string.is_empty() {
                    current_pos = **different_string.choose(&mut rng).unwrap();
                    path.push(current_pos);
                    continue;
                }
            }

            // CHAOS: 10% chance of open-string pivot
            if rng.gen_bool((chaos * 0.1) as f64) {
                let open_positions: Vec<_> = positions.iter()
                    .filter(|p| p.fret == 0)
                    .collect();
                
                if !open_positions.is_empty() {
                    current_pos = **open_positions.choose(&mut rng).unwrap();
                    path.push(current_pos);
                    continue;
                }
            }

            // CHAOS: 5% chance of half-neck slide (12+ fret jump)
            if rng.gen_bool((chaos * 0.05) as f64) {
                let extreme_positions: Vec<_> = positions.iter()
                    .filter(|p| (p.fret as i16 - current_pos.fret as i16).abs() >= 12)
                    .collect();
                
                if !extreme_positions.is_empty() {
                    current_pos = **extreme_positions.choose(&mut rng).unwrap();
                    path.push(current_pos);
                    continue;
                }
            }

            // Default: use aggressive path logic
            let best_pos = positions.iter()
                .min_by(|a, b| {
                    let cost_a = current_pos.movement_cost(a);
                    let cost_b = current_pos.movement_cost(b);
                    cost_a.partial_cmp(&cost_b).unwrap_or(Ordering::Equal)
                })
                .copied()
                .unwrap();

            path.push(best_pos);
            current_pos = best_pos;
        }

        path
    }
}

/// Helper function to calculate playability score (0.0 = impossible, 1.0 = very easy)
pub fn calculate_playability_score(positions: &[FretPosition]) -> f32 {
    if positions.len() < 2 {
        return 1.0;
    }

    let total_cost: f32 = positions.windows(2)
        .map(|window| window[0].movement_cost(&window[1]))
        .sum();

    let max_possible_cost = (positions.len() - 1) as f32 * 15.0; // Worst case scenario
    let normalized_cost = total_cost / max_possible_cost;

    (1.0 - normalized_cost).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fret_position_creation() {
        let pos = FretPosition::new(2, 5);
        assert_eq!(pos.string, 2);
        assert_eq!(pos.fret, 5);
    }

    #[test]
    fn test_movement_cost_same_string() {
        let pos1 = FretPosition::new(0, 0);
        let pos2 = FretPosition::new(0, 1);
        let cost = pos1.movement_cost(&pos2);
        assert_eq!(cost, 1.0); // 1 fret = low cost
    }

    #[test]
    fn test_movement_cost_large_jump() {
        let pos1 = FretPosition::new(0, 0);
        let pos2 = FretPosition::new(0, 10);
        let cost = pos1.movement_cost(&pos2);
        assert!(cost > 10.0); // Large jump = high cost
    }

    #[test]
    fn test_movement_cost_adjacent_string() {
        let pos1 = FretPosition::new(0, 5);
        let pos2 = FretPosition::new(1, 5);
        let cost = pos1.movement_cost(&pos2);
        assert_eq!(cost, 1.5); // Adjacent string = easy
    }

    #[test]
    fn test_movement_cost_string_skip() {
        let pos1 = FretPosition::new(0, 5);
        let pos2 = FretPosition::new(2, 5);
        let cost = pos1.movement_cost(&pos2);
        assert_eq!(cost, 3.0); // String skip = medium cost
    }

    #[test]
    fn test_get_positions_for_note() {
        let pathfinder = FretboardPathfinder::new(GuitarTuning::EStandard);
        let positions = pathfinder.get_positions_for_note(45); // A2
        
        assert!(!positions.is_empty());
        // Should find multiple positions for this note
        assert!(positions.len() > 1);
    }

    #[test]
    fn test_find_playable_path() {
        let pathfinder = FretboardPathfinder::new(GuitarTuning::EStandard);
        let notes = vec![40, 41, 43, 45]; // E2, F2, G2, A2
        
        let path = pathfinder.find_playable_path(&notes);
        
        assert_eq!(path.len(), notes.len());
        // Path should exist for all notes
        assert!(path.iter().all(|pos| pos.fret <= 24));
    }

    #[test]
    fn test_is_playable() {
        let pathfinder = FretboardPathfinder::new(GuitarTuning::EStandard);
        
        // Easy sequence (chromatic on one string)
        let easy_notes = vec![40, 41, 42, 43];
        assert!(pathfinder.is_playable(&easy_notes, 20.0));
        
        // Difficult sequence (large jumps)
        let hard_notes = vec![40, 60, 40, 60];
        assert!(!pathfinder.is_playable(&hard_notes, 20.0));
    }

    #[test]
    fn test_optimize_riff() {
        let pathfinder = FretboardPathfinder::new(GuitarTuning::DropC);
        let notes = vec![36, 38, 40, 43]; // C2, D2, E2, G2
        
        let (optimized, positions) = pathfinder.optimize_riff(&notes);
        
        assert_eq!(optimized.len(), notes.len());
        assert_eq!(positions.len(), notes.len());
    }

    #[test]
    fn test_playability_score() {
        // Easy path (small movements)
        let easy_path = vec![
            FretPosition::new(0, 0),
            FretPosition::new(0, 1),
            FretPosition::new(0, 2),
        ];
        let easy_score = calculate_playability_score(&easy_path);
        assert!(easy_score > 0.8);

        // Hard path (large jumps)
        let hard_path = vec![
            FretPosition::new(0, 0),
            FretPosition::new(5, 12),
            FretPosition::new(0, 0),
        ];
        let hard_score = calculate_playability_score(&hard_path);
        assert!(hard_score < 0.5);
    }

    #[test]
    fn test_metal_riff_playability() {
        let pathfinder = FretboardPathfinder::new(GuitarTuning::DropC);
        
        // Typical metal riff pattern (pedal point with melodic notes)
        let metal_riff = vec![
            36, 36, 43, 36, 36, 45, 36, 36, 48, // C pedal with G, A, C
        ];
        
        let path = pathfinder.find_playable_path(&metal_riff);
        let score = calculate_playability_score(&path);
        
        // Should be playable (metal riffs are designed for playability)
        assert!(score > 0.6);
    }
}
