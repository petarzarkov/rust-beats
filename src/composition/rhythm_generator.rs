
use crate::composition::metal_song_generator::MetalSubgenre;

/// Drum pattern with all drum types
pub struct DrumPattern {
    pub kick: Vec<bool>,
    pub snare: Vec<bool>,
    pub hihat: Vec<bool>,
    pub crash: Vec<bool>,
    pub ride: Vec<bool>,
    pub tom: Vec<bool>,
    pub china: Vec<bool>,
}

impl DrumPattern {
    pub fn new(steps: usize) -> Self {
        Self {
            kick: vec![false; steps],
            snare: vec![false; steps],
            hihat: vec![false; steps],
            crash: vec![false; steps],
            ride: vec![false; steps],
            tom: vec![false; steps],
            china: vec![false; steps],
        }
    }
}

/// Generates a Euclidean rhythm pattern
/// steps: total number of steps (e.g., 16 for a bar of 16th notes)
/// pulses: number of hits distributed as evenly as possible
pub fn generate_euclidean_pattern(steps: usize, pulses: usize) -> Vec<bool> {
    if pulses == 0 {
        return vec![false; steps];
    }
    if pulses >= steps {
        return vec![true; steps];
    }

    let mut pattern = vec![false; steps];
    let bucket_size = steps as f32 / pulses as f32;
    
    for i in 0..pulses {
        let index = (i as f32 * bucket_size).round() as usize;
        if index < steps {
            pattern[index] = true;
        }
    }
    
    pattern
}

/// Generate double bass drum pattern (kick on every 16th note or every 8th)
pub fn generate_double_bass_pattern(steps: usize, intensity: f32) -> Vec<bool> {
    let mut pattern = vec![false; steps];
    
    if intensity > 0.8 {
        // Extreme: Every 16th note
        for i in 0..steps {
            pattern[i] = true;
        }
    } else if intensity > 0.5 {
        // High: Every 8th note
        for i in 0..steps {
            if i % 2 == 0 {
                pattern[i] = true;
            }
        }
    } else {
        // Medium: Galloping pattern
        for i in 0..steps {
            if i % 4 == 0 || i % 4 == 3 {
                pattern[i] = true;
            }
        }
    }
    
    pattern
}

/// Generate tom fill pattern (typically before section changes)
pub fn generate_tom_fill_pattern(steps: usize) -> Vec<bool> {
    let mut pattern = vec![false; steps];
    
    // Tom fill in last 4 steps (last beat of the bar)
    if steps >= 4 {
        let start = steps - 4;
        for i in start..steps {
            pattern[i] = true;
        }
    }
    
    pattern
}

/// Generate crash cymbal accent pattern
pub fn generate_crash_pattern(steps: usize, section_start: bool) -> Vec<bool> {
    let mut pattern = vec![false; steps];
    
    // Crash on section start (beat 1)
    if section_start && steps > 0 {
        pattern[0] = true;
    }
    
    // Crash on strong beats (every 16 steps = every bar)
    for i in 0..steps {
        if i % 16 == 0 {
            pattern[i] = true;
        }
    }
    
    pattern
}

/// Generate ride cymbal pattern (for verses)
pub fn generate_ride_pattern(steps: usize) -> Vec<bool> {
    let mut pattern = vec![false; steps];
    
    // Ride on 8th notes
    for i in 0..steps {
        if i % 2 == 0 {
            pattern[i] = true;
        }
    }
    
    pattern
}

/// Generates a blast beat pattern based on subgenre
pub fn generate_blast_beat(subgenre: MetalSubgenre, steps: usize) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
    // Returns (Kick, Snare, HiHat/Cymbal) patterns
    let mut kick = vec![false; steps];
    let mut snare = vec![false; steps];
    let mut cymbal = vec![false; steps];
    
    match subgenre {
        MetalSubgenre::DeathMetal => {
            // Traditional Blast Beat: Kick and Snare alternating or unison at high speed
            // Here we do a unison blast (Kick+Snare on every beat) or alternating
            for i in 0..steps {
                if i % 2 == 0 { // 8th notes at high tempo effectively
                    kick[i] = true;
                    snare[i] = true; // Unison blast
                    cymbal[i] = true;
                }
            }
        },
        MetalSubgenre::ThrashMetal => {
            // Skank beat (Kick on 1, Snare on &) - not exactly a blast beat but fast
            for i in 0..steps {
                if i % 4 == 0 {
                    kick[i] = true;
                    // Sparse cymbals: only on beat 1
                    if i % 16 == 0 {
                        cymbal[i] = true;
                    }
                } else if i % 4 == 2 {
                    snare[i] = true;
                }
            }
        },
        _ => {
            // Default heavy beat - sparse cymbals
             for i in 0..steps {
                if i % 4 == 0 {
                    kick[i] = true;
                    // Cymbals only on beat 1 of each bar (every 16th step)
                    if i % 16 == 0 {
                        cymbal[i] = true;
                    }
                } else if i % 4 == 2 {
                    snare[i] = true;
                }
            }
        }
    }
    
    (kick, snare, cymbal)
}

/// Generates a breakdown pattern using Euclidean rhythms
pub fn generate_breakdown_pattern(steps: usize, heaviness: f32) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
    let pulses = if heaviness > 0.8 { 5 } else { 3 }; // Irregular pulses for djent feel
    let kick_pattern = generate_euclidean_pattern(steps, pulses);
    
    // Snare usually on beat 3 (half-time feel)
    let mut snare = vec![false; steps];
    if steps >= 9 {
        snare[8] = true; // Backbeat on 3
    }
    
    // Cymbal follows kick or quarter notes
    let mut cymbal = vec![false; steps];
    for i in 0..steps {
        if i % 4 == 0 {
            cymbal[i] = true;
        }
    }
    
    (kick_pattern, snare, cymbal)
}
