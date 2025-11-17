use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrumHit {
    Kick,
    Snare,
    HiHatClosed,
    HiHatOpen,
    Clap,
    Conga,
    Shaker,
    Rest,
}

/// Drum pattern for a full bar (16 steps)
pub type DrumPattern = Vec<Vec<DrumHit>>;  // Vec of steps, each step can have multiple hits

/// Groove style determines the feel of the beat
#[derive(Debug, Clone, Copy)]
pub enum GrooveStyle {
    Funk,
    Jazz,
    ElectroSwing,
    HipHop,
    Rock,
    Lofi,
}

/// Generate a drum pattern based on groove style
pub fn generate_drum_pattern(style: GrooveStyle, num_bars: usize) -> DrumPattern {
    let mut pattern = Vec::new();
    let steps_per_bar = 16;
    
    for _ in 0..num_bars {
        let bar = match style {
            GrooveStyle::Funk => generate_funk_bar(),
            GrooveStyle::Jazz => generate_jazz_bar(),
            GrooveStyle::ElectroSwing => generate_electro_swing_bar(),
            GrooveStyle::HipHop => generate_hiphop_bar(),
            GrooveStyle::Rock => generate_rock_bar(),
            GrooveStyle::Lofi => generate_lofi_bar(),
        };
        pattern.extend(bar);
    }
    
    // Ensure we have at least the minimum steps
    while pattern.len() < steps_per_bar {
        pattern.push(vec![DrumHit::Rest]);
    }
    
    pattern
}

/// Generate a funky drum bar with syncopation
fn generate_funk_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];
    
    // Kick pattern: emphasize 1 and 3, add some syncopation
    bar[0].push(DrumHit::Kick);  // Beat 1 - strong
    
    if rng.gen_range(0..100) < 70 {
        bar[6].push(DrumHit::Kick);  // Syncopated kick
    }
    
    bar[8].push(DrumHit::Kick);  // Beat 3
    
    if rng.gen_range(0..100) < 60 {
        bar[13].push(DrumHit::Kick);  // Optional kick
    }
    
    // Snare on 2 and 4 - vary between snare and clap
    if rng.gen_range(0..100) < 70 {
        bar[4].push(DrumHit::Snare);
    } else {
        bar[4].push(DrumHit::Clap);
    }
    
    if rng.gen_range(0..100) < 70 {
        bar[12].push(DrumHit::Snare);
    } else {
        bar[12].push(DrumHit::Clap);
    }
    
    // Optional ghost notes
    if rng.gen_range(0..100) < 40 {
        bar[2].push(DrumHit::Snare);
    }
    if rng.gen_range(0..100) < 40 {
        bar[10].push(DrumHit::Snare);
    }
    
    // Hi-hats: mix of 8ths and 16ths with open/closed variation
    let hihat_pattern = rng.gen_range(0..3);
    match hihat_pattern {
        0 => {
            // 16th notes with accents
            for i in 0..16 {
                if i % 4 == 0 {
                    bar[i].push(DrumHit::HiHatOpen); // Accent on beats
                } else if i % 2 == 0 {
                    bar[i].push(DrumHit::HiHatClosed);
                } else if rng.gen_range(0..100) < 60 {
                    bar[i].push(DrumHit::HiHatClosed);
                }
            }
        }
        1 => {
            // 8th notes steady
            for i in (0..16).step_by(2) {
                if i % 8 == 6 {
                    bar[i].push(DrumHit::HiHatOpen);
                } else {
                    bar[i].push(DrumHit::HiHatClosed);
                }
            }
        }
        _ => {
            // Mixed pattern
            for i in 0..16 {
                if rng.gen_range(0..100) < 80 {
                    if i % 8 == 6 && rng.gen_range(0..100) < 50 {
                        bar[i].push(DrumHit::HiHatOpen);
                    } else {
                        bar[i].push(DrumHit::HiHatClosed);
                    }
                }
            }
        }
    }
    
    // Add percussion variety
    if rng.gen_range(0..100) < 40 {
        bar[5].push(DrumHit::Shaker);
        bar[13].push(DrumHit::Shaker);
    }
    
    // Occasional conga accent
    if rng.gen_range(0..100) < 30 {
        bar[3].push(DrumHit::Conga);
    }
    
    bar
}

/// Generate a jazz-style drum bar with swing feel
fn generate_jazz_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];
    
    // Jazz ride pattern (simplified for 16-step grid)
    for i in [0, 3, 6, 9, 12, 15] {
        if rng.gen_range(0..100) < 85 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }
    
    // Kick: sparse, emphasizing 1 and 3
    bar[0].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 70 {
        bar[8].push(DrumHit::Kick);
    }
    
    // Snare: 2 and 4 with some ghost notes
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);
    
    if rng.gen_range(0..100) < 30 {
        bar[7].push(DrumHit::Snare);
    }
    
    bar
}

/// Generate an electro-swing style bar
fn generate_electro_swing_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];
    
    // Four-on-the-floor kick
    bar[0].push(DrumHit::Kick);
    bar[4].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    bar[12].push(DrumHit::Kick);
    
    // Claps on 2 and 4 (vintage feel)
    bar[4].push(DrumHit::Clap);
    bar[12].push(DrumHit::Clap);
    
    // Steady hi-hats with variations
    for i in 0..16 {
        if i % 2 == 0 {
            bar[i].push(DrumHit::HiHatClosed);
        } else if rng.gen_range(0..100) < 60 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }
    
    // Add some shuffle with shakers
    if rng.gen_range(0..100) < 70 {
        for i in [1, 5, 9, 13] {
            bar[i].push(DrumHit::Shaker);
        }
    }
    
    bar
}

/// Generate a hip-hop style bar
fn generate_hiphop_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];
    
    // Heavy kick pattern
    bar[0].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    
    // Optional kick hits
    if rng.gen_range(0..100) < 60 {
        bar[3].push(DrumHit::Kick);
    }
    if rng.gen_range(0..100) < 50 {
        bar[14].push(DrumHit::Kick);
    }
    
    // Snare on 2 and 4 - heavy
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);
    
    // Hi-hats: 8ths or 16ths
    let hihat_density = rng.gen_range(0..100);
    if hihat_density < 50 {
        // 8th notes
        for i in [0, 2, 4, 6, 8, 10, 12, 14] {
            bar[i].push(DrumHit::HiHatClosed);
        }
    } else {
        // 16th notes
        for i in 0..16 {
            if rng.gen_range(0..100) < 85 {
                bar[i].push(DrumHit::HiHatClosed);
            }
        }
    }
    
    // Optional open hat
    if rng.gen_range(0..100) < 40 {
        bar[7].push(DrumHit::HiHatOpen);
    }
    
    bar
}

/// Generate a rock drum bar with driving energy
fn generate_rock_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];
    
    // Four-on-the-floor kicks (driving rhythm)
    bar[0].push(DrumHit::Kick);
    bar[4].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    bar[12].push(DrumHit::Kick);
    
    // Heavy snare backbeat on 2 and 4
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);
    
    // Optional double-kick for variety
    if rng.gen_range(0..100) < 30 {
        bar[2].push(DrumHit::Kick);
    }
    if rng.gen_range(0..100) < 30 {
        bar[10].push(DrumHit::Kick);
    }
    
    // Consistent 8th note hi-hats (rock steady)
    for i in [0, 2, 4, 6, 8, 10, 12, 14] {
        bar[i].push(DrumHit::HiHatClosed);
    }
    
    // Occasional open hat for accent
    if rng.gen_range(0..100) < 40 {
        let open_pos = *[6, 14].choose(&mut rng).unwrap();
        bar[open_pos].push(DrumHit::HiHatOpen);
    }
    
    // Optional clap for extra power
    if rng.gen_range(0..100) < 30 {
        bar[4].push(DrumHit::Clap);
        bar[12].push(DrumHit::Clap);
    }
    
    bar
}

/// Generate a lofi drum bar with laid-back swing feel
fn generate_lofi_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];
    
    // Sparse, laid-back kick pattern
    bar[0].push(DrumHit::Kick);  // Beat 1
    
    if rng.gen_range(0..100) < 60 {
        bar[6].push(DrumHit::Kick);  // Syncopated kick
    }
    
    if rng.gen_range(0..100) < 40 {
        bar[11].push(DrumHit::Kick); // Optional late kick
    }
    
    // Soft snare on 2 and 4 (with occasional ghost notes)
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);
    
    // Ghost notes (very soft snare hits)
    if rng.gen_range(0..100) < 30 {
        bar[2].push(DrumHit::Snare);  // Ghost note
    }
    if rng.gen_range(0..100) < 25 {
        bar[10].push(DrumHit::Snare); // Ghost note
    }
    
    // Hi-hats with swing (every other 8th is slightly delayed - shuffle feel)
    let hihat_pattern = rng.gen_range(0..100);
    if hihat_pattern < 60 {
        // Swung 8ths (lofi classic)
        for i in [0, 3, 4, 7, 8, 11, 12, 15] {  // Swung placement
            if rng.gen_range(0..100) < 85 {
                bar[i].push(DrumHit::HiHatClosed);
            }
        }
        // Occasional open hat for accent
        if rng.gen_range(0..100) < 40 {
            bar[7].push(DrumHit::HiHatOpen);
        }
    } else {
        // Sparse 4s
        for i in [0, 4, 8, 12] {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }
    
    // Optional shaker for texture
    if rng.gen_range(0..100) < 35 {
        for i in (0..16).step_by(2) {
            if rng.gen_range(0..100) < 70 {
                bar[i].push(DrumHit::Shaker);
            }
        }
    }
    
    bar
}

/// Choose a random groove style weighted toward lofi/chill
pub fn random_groove_style() -> GrooveStyle {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..100);
    
    match roll {
        0..=50 => GrooveStyle::Lofi,          // Most common (lofi focus)
        51..=65 => GrooveStyle::Jazz,          // Jazzy lofi
        66..=80 => GrooveStyle::HipHop,       // Hip-hop beats
        81..=90 => GrooveStyle::ElectroSwing, // Occasional variety
        91..=95 => GrooveStyle::Funk,          // Rare funky element
        _ => GrooveStyle::Rock,                // Very rare
    }
}

// Keep old function for backwards compatibility but mark deprecated
#[deprecated(note = "Use generate_drum_pattern instead")]
pub fn create_beat(num_steps: u32) -> Result<Vec<DrumSound>, &'static str> {
    if num_steps < 8 || num_steps > 64 {
        return Err("Number of steps must be between 8 and 64");
    }

    let mut beat = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..num_steps {
        let position_in_measure = i % 16;
        
        let roll = rng.gen_range(0..100);

        let sound_to_add = match position_in_measure {
            0 => {
                if roll < 85 { DrumSound::Kick } else { DrumSound::HiHat }
            }
            4 => {
                if roll < 80 { DrumSound::Snare } else { DrumSound::HiHat }
            }
            8 => {
                if roll < 70 { DrumSound::Kick } else { DrumSound::HiHat }
            }
            12 => {
                if roll < 80 { DrumSound::Snare } else { DrumSound::HiHat }
            }
            2 | 6 | 10 | 14 => {
                 if roll < 90 { DrumSound::HiHat } else { DrumSound::Rest }
            }
            _ => {
                if roll < 60 { DrumSound::HiHat }
                else if roll < 90 { DrumSound::Rest }
                else { DrumSound::Snare }
            }
        };

        beat.push(sound_to_add);
    }

    Ok(beat)
}

// Old enum for backwards compatibility
#[derive(Debug, Clone)]
pub enum DrumSound {
    Kick,
    Snare,
    HiHat,
    Rest,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_generation() {
        let styles = [
            GrooveStyle::Funk,
            GrooveStyle::Jazz,
            GrooveStyle::ElectroSwing,
            GrooveStyle::HipHop,
        ];
        
        for style in &styles {
            let pattern = generate_drum_pattern(*style, 2);
            assert!(pattern.len() >= 16);
            println!("{:?} pattern length: {}", style, pattern.len());
        }
    }
}
