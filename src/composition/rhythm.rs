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
