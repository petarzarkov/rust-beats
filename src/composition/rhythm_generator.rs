
use crate::composition::metal_song_generator::MetalSubgenre;

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
