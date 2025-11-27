/// Euclidean rhythm generation using Bjorklund's algorithm
/// Distributes k pulses as evenly as possible over n steps
/// This is the proper algorithm used in neutron accelerators and metal rhythm generation

pub fn euclidean_rhythm(steps: usize, pulses: usize) -> Vec<bool> {
    if pulses == 0 || steps == 0 || pulses > steps {
        return vec![false; steps];
    }
    if pulses == steps {
        return vec![true; steps];
    }

    // Bjorklund's algorithm: distribute pulses as evenly as possible
    // Start with pulses as [1] and rests as [0]
    let mut groups: Vec<Vec<bool>> = Vec::new();
    
    // Initialize with 'pulses' groups of [true] and 'steps - pulses' groups of [false]
    for _ in 0..pulses {
        groups.push(vec![true]);
    }
    for _ in 0..(steps - pulses) {
        groups.push(vec![false]);
    }

    // Bjorklund's pairing algorithm
    let mut count = pulses.min(steps - pulses);
    
    while count > 1 {
        let mut i = 0;
        
        // Pair groups: take from the end and append to the beginning
        for _ in 0..count {
            if i + count < groups.len() {
                let mut combined = groups[i].clone();
                combined.extend(&groups[i + count]);
                groups[i] = combined;
            }
            i += 1;
        }
        
        // Remove the paired groups from the end
        groups.truncate(groups.len() - count);
        
        // Calculate new count for next iteration
        let remainder = groups.len() - count;
        count = count.min(remainder);
    }

    // Flatten the result
    groups.into_iter().flatten().collect()
}

/// Rotate a rhythm pattern by n steps
pub fn rotate_rhythm(pattern: &[bool], rotation: usize) -> Vec<bool> {
    if pattern.is_empty() {
        return vec![];
    }
    let rot = rotation % pattern.len();
    let mut rotated = Vec::with_capacity(pattern.len());
    rotated.extend_from_slice(&pattern[rot..]);
    rotated.extend_from_slice(&pattern[..rot]);
    rotated
}

/// Convert rhythm bitmap to beat positions
/// subdivision: 0.25 for 16th notes, 0.5 for 8th notes, etc.
pub fn bitmap_to_positions(bitmap: &[bool], subdivision: f32) -> Vec<f32> {
    bitmap
        .iter()
        .enumerate()
        .filter_map(|(i, &hit)| {
            if hit {
                Some(i as f32 * subdivision)
            } else {
                None
            }
        })
        .collect()
}

/// Polymetric riff structure
#[derive(Debug, Clone)]
pub struct PolymetricRiff {
    pub phrase_length: usize,  // e.g., 5, 7, 9 sixteenth notes
    pub bar_length: usize,     // e.g., 16 (4/4 time)
    pub truncate: bool,        // Truncate at bar end vs. wrap
}

impl PolymetricRiff {
    pub fn new(phrase_length: usize, bar_length: usize, truncate: bool) -> Self {
        PolymetricRiff {
            phrase_length,
            bar_length,
            truncate,
        }
    }

    /// Calculate when the phrase resolves (LCM of phrase and bar length)
    pub fn resolution_point(&self) -> usize {
        lcm(self.phrase_length, self.bar_length)
    }

    /// Calculate how many bars until resolution
    pub fn bars_to_resolution(&self) -> usize {
        self.resolution_point() / self.bar_length
    }

    /// Fill bars with the phrase pattern
    pub fn fill_bars<T: Clone>(&self, phrase: &[T], num_bars: usize) -> Vec<T> {
        let total_steps = num_bars * self.bar_length;
        let mut result = Vec::with_capacity(total_steps);

        let mut phrase_index = 0;
        for step in 0..total_steps {
            if self.truncate && step % self.bar_length == 0 && step > 0 {
                // Reset phrase at bar boundary
                phrase_index = 0;
            }

            result.push(phrase[phrase_index % phrase.len()].clone());
            phrase_index += 1;
        }

        result
    }
}

/// Calculate least common multiple
fn lcm(a: usize, b: usize) -> usize {
    (a * b) / gcd(a, b)
}

/// Calculate greatest common divisor
fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Generate a breakdown pattern (metalcore/deathcore)
/// Returns (position, is_hit) pairs
pub fn generate_breakdown_pattern(
    bar_duration: f32,
    syncopation_level: f32, // 0.0 = simple, 1.0 = complex
) -> Vec<(f32, bool)> {
    let mut pattern = Vec::new();
    let sixteenth = bar_duration / 16.0;

    // Halftime feel: snare on beat 3 (position 8 in 16ths)
    // Kick pattern with syncopation

    if syncopation_level < 0.3 {
        // Simple breakdown: 1, 1-and, 3
        pattern.push((0.0, true));           // Beat 1
        pattern.push((sixteenth * 2.0, true)); // 1-and
        pattern.push((sixteenth * 8.0, true)); // Beat 3 (snare)
    } else if syncopation_level < 0.7 {
        // Medium: Add more syncopation
        pattern.push((0.0, true));
        pattern.push((sixteenth * 1.0, true));
        pattern.push((sixteenth * 2.0, true));
        pattern.push((sixteenth * 8.0, true)); // Snare
        pattern.push((sixteenth * 10.0, true));
    } else {
        // Complex: Burst patterns
        pattern.push((0.0, true));
        pattern.push((sixteenth * 0.5, true));
        pattern.push((sixteenth * 1.0, true));
        pattern.push((sixteenth * 1.5, true));
        // Rest
        pattern.push((sixteenth * 8.0, true)); // Snare
        pattern.push((sixteenth * 11.0, true));
        pattern.push((sixteenth * 11.5, true));
    }

    pattern
}

/// Odd subdivision pattern generator (quintuplets, septuplets, etc.)
#[derive(Debug, Clone)]
pub struct OddSubdivisionPattern {
    pub notes_per_beat: usize, // 5 for quintuplets, 7 for septuplets, etc.
    pub beats: usize,           // Number of beats to fill
}

impl OddSubdivisionPattern {
    pub fn quintuplet(beats: usize) -> Self {
        OddSubdivisionPattern {
            notes_per_beat: 5,
            beats,
        }
    }

    pub fn septuplet(beats: usize) -> Self {
        OddSubdivisionPattern {
            notes_per_beat: 7,
            beats,
        }
    }

    pub fn custom(notes_per_beat: usize, beats: usize) -> Self {
        OddSubdivisionPattern {
            notes_per_beat,
            beats,
        }
    }

    /// Generate a pattern with the specified subdivision
    /// Returns positions in beats (0.0 = start of bar)
    pub fn generate_positions(&self) -> Vec<f32> {
        let total_notes = self.notes_per_beat * self.beats;
        let subdivision = self.beats as f32 / total_notes as f32;
        
        (0..total_notes)
            .map(|i| i as f32 * subdivision)
            .collect()
    }

    /// Get the duration of each note in beats
    pub fn note_duration(&self) -> f32 {
        self.beats as f32 / (self.notes_per_beat * self.beats) as f32
    }
}

/// Displaced accent generator for off-grid rhythms
#[derive(Debug, Clone)]
pub struct DisplacedAccentGenerator {
    pub start_offset: f32,      // Offset from beat 1 (in beats)
    pub accent_interval: usize, // Accent every N notes
}

impl DisplacedAccentGenerator {
    /// Create a pattern that starts late (after beat 1)
    pub fn start_late(offset_beats: f32) -> Self {
        DisplacedAccentGenerator {
            start_offset: offset_beats,
            accent_interval: 4,
        }
    }

    /// Create a pattern with custom accent interval
    pub fn with_accents(start_offset: f32, accent_interval: usize) -> Self {
        DisplacedAccentGenerator {
            start_offset,
            accent_interval,
        }
    }

    /// Apply displacement to a pattern
    /// Returns (position, is_accent) pairs
    pub fn apply_to_pattern(&self, positions: &[f32]) -> Vec<(f32, bool)> {
        positions
            .iter()
            .enumerate()
            .map(|(i, &pos)| {
                let displaced_pos = pos + self.start_offset;
                let is_accent = i % self.accent_interval == 0;
                (displaced_pos, is_accent)
            })
            .collect()
    }

    /// Check if a riff overlaps the barline
    pub fn overlaps_barline(&self, pattern_length: f32, bar_length: f32) -> bool {
        (self.start_offset + pattern_length) > bar_length
    }
}

/// Polymeter interference system for phasing guitar/drum patterns
#[derive(Debug, Clone)]
pub struct PolymetricInterference {
    pub guitar_meter: usize,  // e.g., 5 for 5/16
    pub kick_meter: usize,    // e.g., 4 for 4/4
    pub snare_meter: usize,   // e.g., 2 for half-time feel
}

impl PolymetricInterference {
    /// Create a standard prog-metal polymeter (guitars in 5, drums in 4)
    pub fn prog_metal() -> Self {
        PolymetricInterference {
            guitar_meter: 5,
            kick_meter: 4,
            snare_meter: 2, // Half-time feel
        }
    }

    /// Create a djent-style polymeter (guitars in 7, drums in 4)
    pub fn djent() -> Self {
        PolymetricInterference {
            guitar_meter: 7,
            kick_meter: 4,
            snare_meter: 4,
        }
    }

    /// Custom polymeter
    pub fn custom(guitar_meter: usize, kick_meter: usize, snare_meter: usize) -> Self {
        PolymetricInterference {
            guitar_meter,
            kick_meter,
            snare_meter,
        }
    }

    /// Calculate when all meters align (resolution point)
    pub fn resolution_point(&self) -> usize {
        let guitar_kick_lcm = lcm(self.guitar_meter, self.kick_meter);
        lcm(guitar_kick_lcm, self.snare_meter)
    }

    /// Calculate how many bars until resolution (assuming 4/4 time signature)
    pub fn bars_to_resolution(&self) -> usize {
        self.resolution_point() / 4
    }

    /// Generate guitar pattern positions (in 16th notes)
    pub fn guitar_pattern(&self, bars: usize) -> Vec<usize> {
        let total_sixteenths = bars * 16;
        let mut positions = Vec::new();
        let mut pos = 0;
        
        while pos < total_sixteenths {
            positions.push(pos);
            pos += self.guitar_meter;
        }
        
        positions
    }

    /// Generate kick pattern positions (in 16th notes)
    pub fn kick_pattern(&self, bars: usize) -> Vec<usize> {
        let total_sixteenths = bars * 16;
        let mut positions = Vec::new();
        let mut pos = 0;
        
        while pos < total_sixteenths {
            positions.push(pos);
            pos += self.kick_meter;
        }
        
        positions
    }

    /// Generate snare pattern positions (in 16th notes)
    /// Typically on beat 3 in half-time feel
    pub fn snare_pattern(&self, bars: usize) -> Vec<usize> {
        let total_sixteenths = bars * 16;
        let mut positions = Vec::new();
        
        // Half-time: snare on beat 3 (position 8 in 16ths)
        for bar in 0..bars {
            let snare_pos = bar * 16 + 8;
            if snare_pos < total_sixteenths {
                positions.push(snare_pos);
            }
        }
        
        positions
    }

    /// Check if patterns are aligned at a given position
    pub fn is_aligned(&self, position: usize) -> bool {
        position % self.guitar_meter == 0
            && position % self.kick_meter == 0
            && position % self.snare_meter == 0
    }

    /// CHAOS: Partial reset - reset guitar meter but keep drums going
    pub fn partial_reset(&mut self, bar_num: usize) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Every 4-8 bars, 30% chance of partial reset
        if bar_num % rng.gen_range(4..=8) == 0 && rng.gen_bool(0.3) {
            // Reset guitar meter to a different odd number
            self.guitar_meter = match rng.gen_range(0..3) {
                0 => 5,
                1 => 7,
                _ => 9,
            };
            true
        } else {
            false
        }
    }

    /// CHAOS: Generate mismatched overlap (guitar bleeds into next bar)
    pub fn mismatched_overlap(&self, bars: usize) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut pos = 0;
        let total_sixteenths = bars * 16;
        
        // Don't truncate at bar boundaries - let it bleed
        while pos < total_sixteenths + self.guitar_meter {
            positions.push(pos);
            pos += self.guitar_meter;
        }
        
        positions
    }

    /// CHAOS: Sudden meter drop (7 → 3, 5 → 2)
    pub fn sudden_drop(&mut self) -> usize {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let old_meter = self.guitar_meter;
        
        // Drop to a smaller meter
        self.guitar_meter = match old_meter {
            7 | 9 => 3,
            5 => 2,
            _ => 3,
        };
        
        old_meter
    }

    /// CHAOS: Generate a bar that completely ignores meter
    pub fn chaos_bar(&self) -> Vec<usize> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut positions = Vec::new();
        
        // Random number of hits (3-11)
        let num_hits = rng.gen_range(3..=11);
        
        // Random positions within 16 sixteenths
        for _ in 0..num_hits {
            positions.push(rng.gen_range(0..16));
        }
        
        positions.sort();
        positions.dedup();
        positions
    }

    /// Generate chaotic guitar pattern with controlled sabotage
    pub fn chaotic_guitar_pattern(&mut self, bars: usize) -> Vec<usize> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut all_positions = Vec::new();
        
        for bar in 0..bars {
            let bar_offset = bar * 16;
            
            // 5% chance of full chaos bar
            if rng.gen_bool(0.05) {
                let chaos = self.chaos_bar();
                all_positions.extend(chaos.iter().map(|&p| p + bar_offset));
                continue;
            }
            
            // 10% chance of sudden meter drop
            if rng.gen_bool(0.1) {
                self.sudden_drop();
            }
            
            // 20% chance of mismatched overlap
            if rng.gen_bool(0.2) {
                let overlap = self.mismatched_overlap(1);
                all_positions.extend(overlap.iter().map(|&p| p + bar_offset));
            } else {
                // Normal pattern for this bar
                let mut pos = 0;
                while pos < 16 {
                    all_positions.push(bar_offset + pos);
                    pos += self.guitar_meter;
                }
            }
            
            // Check for partial reset
            self.partial_reset(bar);
        }
        
        all_positions.sort();
        all_positions.dedup();
        all_positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_rhythm() {
        let rhythm = euclidean_rhythm(16, 5);
        assert_eq!(rhythm.len(), 16);
        assert_eq!(rhythm.iter().filter(|&&x| x).count(), 5);
    }

    #[test]
    fn test_euclidean_rhythm_edge_cases() {
        let empty: Vec<bool> = vec![];
        assert_eq!(euclidean_rhythm(0, 0), empty);
        assert_eq!(euclidean_rhythm(8, 0), vec![false; 8]);
        assert_eq!(euclidean_rhythm(8, 10), vec![false; 8]); // More pulses than steps
    }

    #[test]
    fn test_rotate_rhythm() {
        let pattern = vec![true, false, false, true];
        let rotated = rotate_rhythm(&pattern, 1);
        let expected: Vec<bool> = vec![false, false, true, true];
        assert_eq!(rotated, expected);
    }

    #[test]
    fn test_bitmap_to_positions() {
        let bitmap = vec![true, false, true, false];
        let positions = bitmap_to_positions(&bitmap, 0.25);
        assert_eq!(positions, vec![0.0, 0.5]);
    }

    #[test]
    fn test_polymetric_resolution() {
        let poly = PolymetricRiff::new(5, 16, false);
        assert_eq!(poly.resolution_point(), 80);
        assert_eq!(poly.bars_to_resolution(), 5);
    }

    #[test]
    fn test_lcm_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(lcm(5, 16), 80);
        assert_eq!(lcm(7, 16), 112);
    }

    #[test]
    fn test_breakdown_pattern() {
        let pattern = generate_breakdown_pattern(4.0, 0.5);
        assert!(!pattern.is_empty());
        // Verify snare hit exists around beat 3
        let has_snare_area = pattern.iter().any(|(pos, _)| *pos > 1.5 && *pos < 2.5);
        assert!(has_snare_area);
    }
}
