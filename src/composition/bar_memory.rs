use crate::composition::music_theory::MidiNote;
use crate::composition::metal_song_generator::RhythmPattern;
use rand::Rng;

/// Types of mutations that can be applied to a bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationType {
    IntervalShift,
    RhythmRotation,
    SlideInsertion,
    TrillInsertion,
    OctaveJump,
}

/// Note modifier for slides and trills
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteModifier {
    None,
    SlideUp,
    SlideDown,
    Trill,
}

/// Stores the previous bar's shape for evolutionary mutations
pub struct BarMotifStore {
    pub previous_notes: Vec<MidiNote>,
    pub previous_intervals: Vec<i8>,
    pub previous_rhythm_cells: Vec<RhythmPattern>,
    pub mutation_history: Vec<MutationType>,
    pub mutation_depth: usize,
    pub max_mutations: usize,
}

impl BarMotifStore {
    /// Create a new bar motif store
    pub fn new() -> Self {
        BarMotifStore {
            previous_notes: Vec::new(),
            previous_intervals: Vec::new(),
            previous_rhythm_cells: Vec::new(),
            mutation_history: Vec::new(),
            mutation_depth: 0,
            max_mutations: 3,
        }
    }

    /// Store a new bar for future mutations
    pub fn store_bar(&mut self, notes: &[MidiNote], rhythms: &[RhythmPattern]) {
        self.previous_notes = notes.to_vec();
        self.previous_rhythm_cells = rhythms.to_vec();
        
        // Calculate intervals between consecutive notes
        self.previous_intervals.clear();
        for i in 1..notes.len() {
            let interval = (notes[i] as i16 - notes[i-1] as i16) as i8;
            self.previous_intervals.push(interval);
        }
    }

    /// Check if we should reset (too many mutations)
    pub fn should_reset(&self) -> bool {
        self.mutation_depth >= self.max_mutations
    }

    /// Reset mutation depth
    pub fn reset(&mut self) {
        self.mutation_depth = 0;
        self.mutation_history.clear();
    }

    /// Shift intervals by a constant amount
    /// Example: [0, 3, 5] with shift +1 → [0, 4, 6]
    pub fn shift_intervals(&self, shift: i8) -> Vec<i8> {
        self.previous_intervals.iter()
            .map(|&interval| interval.saturating_add(shift))
            .collect()
    }

    /// Apply shifted intervals to create new notes
    pub fn apply_shifted_intervals(&self, root: MidiNote, shift: i8) -> Vec<MidiNote> {
        if self.previous_notes.is_empty() {
            return vec![root];
        }

        let shifted_intervals = self.shift_intervals(shift);
        let mut notes = vec![root];
        
        for &interval in &shifted_intervals {
            let last_note = *notes.last().unwrap();
            let new_note = (last_note as i16 + interval as i16).clamp(0, 127) as MidiNote;
            notes.push(new_note);
        }
        
        notes
    }

    /// Rotate rhythm cells
    /// Example: [Q, E, E, S] rotated by 1 → [E, S, Q, E]
    pub fn rotate_rhythm(&self, positions: usize) -> Vec<RhythmPattern> {
        if self.previous_rhythm_cells.is_empty() {
            return Vec::new();
        }

        let len = self.previous_rhythm_cells.len();
        let positions = positions % len;
        
        let mut rotated = Vec::with_capacity(len);
        rotated.extend_from_slice(&self.previous_rhythm_cells[positions..]);
        rotated.extend_from_slice(&self.previous_rhythm_cells[..positions]);
        
        rotated
    }

    /// Insert slides instead of re-rolling notes
    pub fn insert_slides(&self, probability: f32) -> Vec<NoteModifier> {
        let mut rng = rand::thread_rng();
        let mut modifiers = Vec::with_capacity(self.previous_notes.len());
        
        for i in 0..self.previous_notes.len() {
            if i > 0 && rng.gen_bool(probability as f64) {
                // Determine slide direction based on interval
                let interval = if i < self.previous_intervals.len() {
                    self.previous_intervals[i - 1]
                } else {
                    0
                };
                
                let modifier = if interval > 0 {
                    NoteModifier::SlideUp
                } else if interval < 0 {
                    NoteModifier::SlideDown
                } else {
                    NoteModifier::None
                };
                
                modifiers.push(modifier);
            } else {
                modifiers.push(NoteModifier::None);
            }
        }
        
        modifiers
    }

    /// Insert trills (rapid alternation between two notes)
    pub fn insert_trills(&self, probability: f32) -> Vec<NoteModifier> {
        let mut rng = rand::thread_rng();
        let mut modifiers = Vec::with_capacity(self.previous_notes.len());
        
        for _ in 0..self.previous_notes.len() {
            if rng.gen_bool(probability as f64) {
                modifiers.push(NoteModifier::Trill);
            } else {
                modifiers.push(NoteModifier::None);
            }
        }
        
        modifiers
    }

    /// Generate a mutated bar based on the previous bar
    pub fn mutate_bar(&mut self, root: MidiNote) -> (Vec<MidiNote>, Vec<RhythmPattern>, Vec<NoteModifier>) {
        let mut rng = rand::thread_rng();
        
        // Choose mutation type
        let mutation = match rng.gen_range(0..5) {
            0 => MutationType::IntervalShift,
            1 => MutationType::RhythmRotation,
            2 => MutationType::SlideInsertion,
            3 => MutationType::TrillInsertion,
            _ => MutationType::OctaveJump,
        };
        
        self.mutation_history.push(mutation);
        self.mutation_depth += 1;
        
        let notes = match mutation {
            MutationType::IntervalShift => {
                let shift = rng.gen_range(-2..=2);
                self.apply_shifted_intervals(root, shift)
            },
            MutationType::OctaveJump => {
                // Jump up or down an octave
                let shift = if rng.gen_bool(0.5) { 12 } else { -12 };
                self.previous_notes.iter()
                    .map(|&note| (note as i16 + shift).clamp(0, 127) as MidiNote)
                    .collect()
            },
            _ => self.previous_notes.clone(),
        };
        
        let rhythms = if mutation == MutationType::RhythmRotation {
            let positions = rng.gen_range(1..=3);
            self.rotate_rhythm(positions)
        } else {
            self.previous_rhythm_cells.clone()
        };
        
        let modifiers = match mutation {
            MutationType::SlideInsertion => self.insert_slides(0.3),
            MutationType::TrillInsertion => self.insert_trills(0.2),
            _ => vec![NoteModifier::None; notes.len()],
        };
        
        (notes, rhythms, modifiers)
    }
}

impl Default for BarMotifStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_intervals() {
        let mut store = BarMotifStore::new();
        store.previous_intervals = vec![2, 3, -1];
        
        let shifted = store.shift_intervals(1);
        assert_eq!(shifted, vec![3, 4, 0]);
    }

    #[test]
    fn test_rotate_rhythm() {
        let mut store = BarMotifStore::new();
        store.previous_rhythm_cells = vec![
            RhythmPattern::QuarterNote,
            RhythmPattern::EighthNote,
            RhythmPattern::SixteenthNote,
        ];
        
        let rotated = store.rotate_rhythm(1);
        assert_eq!(rotated[0], RhythmPattern::EighthNote);
        assert_eq!(rotated[1], RhythmPattern::SixteenthNote);
        assert_eq!(rotated[2], RhythmPattern::QuarterNote);
    }

    #[test]
    fn test_mutation_depth() {
        let mut store = BarMotifStore::new();
        assert!(!store.should_reset());
        
        store.mutation_depth = 3;
        assert!(store.should_reset());
        
        store.reset();
        assert_eq!(store.mutation_depth, 0);
    }
}
