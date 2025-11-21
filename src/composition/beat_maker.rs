use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrumHit {
    Kick,
    Snare,
    HiHatClosed,
    HiHatOpen,
    Clap,
    Conga,
    Shaker,
    Crash,        // Crash cymbal for transitions
    RimShot,      // Rim shot for accents
    Tom,          // Tom for fills
    Ride,         // Ride cymbal
    Rest,
}

/// Drum pattern for a full bar (16 steps)
pub type DrumPattern = Vec<Vec<DrumHit>>; // Vec of steps, each step can have multiple hits

/// Groove style determines the feel of the beat
#[derive(Debug, Clone, Copy)]
pub enum GrooveStyle {
    Funk,
    Jazz,
    ElectroSwing,
    HipHop,
    Rock,
    Lofi,
    Dubstep,
    DnB,
}

/// Drum kit type determines the sonic character of drums
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrumKit {
    Acoustic,      // Natural, warm drum sounds
    Electronic808, // Classic drum machine
    HipHop,        // Punchy, sampled-style
    Rock,          // Powerful, aggressive
    Jazz,          // Soft, brushed
    Lofi,          // Muted, vintage
}

/// Select a random drum kit with weighted probabilities
pub fn select_random_drum_kit() -> DrumKit {
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen_range(0.0..1.0);

    // Weighted distribution for variety
    if roll < 0.25 {
        DrumKit::Lofi
    } else if roll < 0.45 {
        DrumKit::Acoustic
    } else if roll < 0.60 {
        DrumKit::HipHop
    } else if roll < 0.75 {
        DrumKit::Electronic808
    } else if roll < 0.88 {
        DrumKit::Jazz
    } else {
        DrumKit::Rock
    }
}

/// Prefer drum kits defined by the genre configuration, but fall back to the global distribution
pub fn select_preferred_drum_kit(preferences: &[DrumKit]) -> DrumKit {
    let mut rng = rand::thread_rng();

    if !preferences.is_empty() {
        // 80% chance to pick from the preferred list to keep the genre identity strong
        if rng.gen_range(0..100) < 80 {
            let idx = rng.gen_range(0..preferences.len());
            return preferences[idx];
        }
    }

    select_random_drum_kit()
}

/// Generate a drum pattern based on groove style with per-bar variation
pub fn generate_drum_pattern(style: GrooveStyle, num_bars: usize) -> DrumPattern {
    let mut pattern = Vec::new();
    let steps_per_bar = 16;
    let mut rng = rand::thread_rng();

    for bar_idx in 0..num_bars {
        if rng.gen_range(0..100) < 10 {
            if let Some(legacy) = legacy_pattern_from_create_beat(1) {
                pattern.extend(legacy);
                continue;
            }
        }

        // MUCH MORE VARIATION: 50% chance for variant patterns (up from 20%)
        // This creates much more rhythmic variety throughout the song
        let use_variant = rng.gen_range(0..100) < 50;

        // Add fills before section transitions (every 8 or 16 bars)
        let is_transition_bar = (bar_idx + 1) % 8 == 0 || (bar_idx + 1) % 16 == 0;
        let add_fill = is_transition_bar && rng.gen_range(0..100) < 70; // 70% chance
        let bar = match style {
            GrooveStyle::Funk => {
                if use_variant {
                    // Choose from multiple variants for more variety
                    if rng.gen_range(0..100) < 50 {
                        generate_funk_bar_variant()
                    } else {
                        generate_funk_bar_variant2()
                    }
                } else {
                    generate_funk_bar()
                }
            }
            GrooveStyle::Jazz => {
                if use_variant {
                    if rng.gen_range(0..100) < 50 {
                        generate_jazz_bar_variant()
                    } else {
                        generate_jazz_bar_variant2()
                    }
                } else {
                    generate_jazz_bar()
                }
            }
            GrooveStyle::ElectroSwing => {
                if use_variant {
                    generate_electro_swing_bar_variant()
                } else {
                    generate_electro_swing_bar()
                }
            }
            GrooveStyle::HipHop => {
                if use_variant {
                    if rng.gen_range(0..100) < 50 {
                        generate_hiphop_bar_variant()
                    } else {
                        generate_hiphop_bar_variant2()
                    }
                } else {
                    generate_hiphop_bar()
                }
            }
            GrooveStyle::Rock => {
                if use_variant {
                    if rng.gen_range(0..100) < 50 {
                        generate_rock_bar_variant()
                    } else {
                        generate_rock_bar_variant2()
                    }
                } else {
                    generate_rock_bar()
                }
            }
            GrooveStyle::Lofi => {
                if use_variant {
                    generate_lofi_bar_variant()
                } else {
                    generate_lofi_bar()
                }
            }
            GrooveStyle::Dubstep => generate_dubstep_bar(),
            GrooveStyle::DnB => {
                if use_variant {
                    if rng.gen_range(0..100) < 50 {
                        generate_dnb_bar_variant()
                    } else {
                        generate_dnb_bar_variant2()
                    }
                } else {
                    generate_dnb_bar()
                }
            }
        };

        // If this is a fill bar, replace the last 1-2 beats with a drum fill
        if add_fill {
            let mut fill_bar = bar.clone();
            let fill = generate_drum_fill(style, &mut rng);
            // Replace last 4-8 steps with fill
            let fill_start = 16 - fill.len().min(8);
            for (i, fill_hit) in fill.iter().enumerate() {
                if fill_start + i < 16 {
                    fill_bar[fill_start + i] = fill_hit.clone();
                }
            }
            pattern.extend(fill_bar);
        } else {
            pattern.extend(bar);
        }
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
    bar[0].push(DrumHit::Kick); // Beat 1 - strong

    if rng.gen_range(0..100) < 70 {
        bar[6].push(DrumHit::Kick); // Syncopated kick
    }

    bar[8].push(DrumHit::Kick); // Beat 3

    if rng.gen_range(0..100) < 60 {
        bar[13].push(DrumHit::Kick); // Optional kick
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

    // MUCH SPARSER percussion (was causing jungle dnb feel)
    // Only 10% chance (down from 40%) and only 1 shaker (not 2!)
    if rng.gen_range(0..100) < 10 {
        bar[13].push(DrumHit::Shaker); // Just one, at end of bar
    }

    // Very rare conga accent (5% down from 30%)
    if rng.gen_range(0..100) < 5 {
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

    // MUCH SPARSER shakers (down from 70% with 4 hits to 15% with 1-2 hits)
    if rng.gen_range(0..100) < 15 {
        // Only 1-2 shakers per bar, not 4!
        bar[5].push(DrumHit::Shaker);
        if rng.gen_range(0..100) < 40 {
            bar[13].push(DrumHit::Shaker);
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
    bar[0].push(DrumHit::Kick); // Beat 1

    if rng.gen_range(0..100) < 60 {
        bar[6].push(DrumHit::Kick); // Syncopated kick
    }

    if rng.gen_range(0..100) < 40 {
        bar[11].push(DrumHit::Kick); // Optional late kick
    }

    // Soft snare on 2 and 4 (with occasional ghost notes)
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Ghost notes (very soft snare hits)
    if rng.gen_range(0..100) < 30 {
        bar[2].push(DrumHit::Snare); // Ghost note
    }
    if rng.gen_range(0..100) < 25 {
        bar[10].push(DrumHit::Snare); // Ghost note
    }

    // Hi-hats with swing (every other 8th is slightly delayed - shuffle feel)
    let hihat_pattern = rng.gen_range(0..100);
    if hihat_pattern < 60 {
        // Swung 8ths (lofi classic)
        for i in [0, 3, 4, 7, 8, 11, 12, 15] {
            // Swung placement
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

    // VERY SPARSE shaker for texture (was 35% with up to 8 hits! Now 10% with 1-2 hits)
    if rng.gen_range(0..100) < 10 {
        // Only 1-2 shakers per bar, placed sparsely
        bar[4].push(DrumHit::Shaker);
        if rng.gen_range(0..100) < 30 {
            bar[12].push(DrumHit::Shaker);
        }
    }

    bar
}

/// Generate a dubstep drum bar with half-time feel (enhanced with more variance)
fn generate_dubstep_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Half-time feel: Kick on 1 and 3 (steps 0 and 8) with variations
    bar[0].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    
    // Double kicks for variation
    if rng.gen_range(0..100) < 40 {
        bar[1].push(DrumHit::Kick); // Double kick after beat 1
    }
    if rng.gen_range(0..100) < 35 {
        bar[9].push(DrumHit::Kick); // Double kick after beat 3
    }
    
    // Syncopated kicks for complexity
    if rng.gen_range(0..100) < 30 {
        bar[6].push(DrumHit::Kick); // Syncopated kick
    }
    if rng.gen_range(0..100) < 25 {
        bar[14].push(DrumHit::Kick); // Syncopated kick before snare
    }

    // Snare on 2 and 4 (steps 4 and 12) - classic dubstep pattern
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);
    
    // Ghost snares for texture
    if rng.gen_range(0..100) < 35 {
        bar[2].push(DrumHit::Snare); // Ghost snare
    }
    if rng.gen_range(0..100) < 30 {
        bar[10].push(DrumHit::Snare); // Ghost snare
    }

    // Snare rolls before drops (more variation)
    if rng.gen_range(0..100) < 40 {
        bar[14].push(DrumHit::Snare);
        bar[15].push(DrumHit::Snare);
    }
    // Alternative snare roll pattern
    if rng.gen_range(0..100) < 25 {
        bar[13].push(DrumHit::Snare);
        bar[14].push(DrumHit::Snare);
        bar[15].push(DrumHit::Snare);
    }

    // More dynamic hi-hat patterns with velocity variations
    // Base pattern with variation
    for i in 0..16 {
        let chance = if i % 2 == 0 { 70 } else { 60 }; // Slight variation
        if rng.gen_range(0..100) < chance {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }
    
    // Hi-hat breaks (occasional gaps for dynamics)
    if rng.gen_range(0..100) < 30 {
        let break_start = rng.gen_range(4..12);
        for i in break_start..(break_start + 2).min(16) {
            // Remove hi-hats (already handled by chance above)
        }
    }

    // Clap layered with snare for extra punch (more variation)
    if rng.gen_range(0..100) < 60 {
        bar[4].push(DrumHit::Clap);
    }
    if rng.gen_range(0..100) < 55 {
        bar[12].push(DrumHit::Clap);
    }
    
    // Occasional crash/ride accents
    if rng.gen_range(0..100) < 25 {
        let accent_pos = *[0, 4, 8, 12].choose(&mut rng).unwrap();
        bar[accent_pos].push(DrumHit::Crash);
    }
    if rng.gen_range(0..100) < 20 {
        let ride_pos = rng.gen_range(0..16);
        bar[ride_pos].push(DrumHit::Ride);
    }

    bar
}

/// Generate a DnB drum bar with fast breakbeat patterns (enhanced with more variance)
fn generate_dnb_bar() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Fast breakbeat: kick-snare-kick-snare variations with ghost kicks
    // Main kick pattern (varied)
    bar[0].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 70 {
        bar[4].push(DrumHit::Kick);
    }
    if rng.gen_range(0..100) < 60 {
        bar[8].push(DrumHit::Kick);
    }
    if rng.gen_range(0..100) < 50 {
        bar[12].push(DrumHit::Kick);
    }
    
    // Ghost kicks (off-beat, quieter kicks) for more complexity
    if rng.gen_range(0..100) < 40 {
        bar[2].push(DrumHit::Kick); // Ghost kick
    }
    if rng.gen_range(0..100) < 35 {
        bar[6].push(DrumHit::Kick); // Ghost kick
    }
    if rng.gen_range(0..100) < 30 {
        bar[10].push(DrumHit::Kick); // Ghost kick
    }
    if rng.gen_range(0..100) < 25 {
        bar[14].push(DrumHit::Kick); // Ghost kick
    }

    // Syncopated snare hits with more variations
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);
    // More syncopated snares for complexity
    if rng.gen_range(0..100) < 35 {
        bar[2].push(DrumHit::Snare); // Syncopated
    }
    if rng.gen_range(0..100) < 30 {
        bar[6].push(DrumHit::Snare); // Syncopated
    }
    if rng.gen_range(0..100) < 25 {
        bar[10].push(DrumHit::Snare); // Syncopated
    }
    if rng.gen_range(0..100) < 20 {
        bar[14].push(DrumHit::Snare); // Syncopated
    }

    // More complex hi-hat patterns with rolls and variations
    // Base pattern
    for i in 0..16 {
        if rng.gen_range(0..100) < 45 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }

    // Hi-hat rolls (rapid 16th notes) - occasional
    if rng.gen_range(0..100) < 30 {
        let roll_start = rng.gen_range(0..12);
        for i in roll_start..(roll_start + 4).min(16) {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }

    // Shuffle/swing feel on hi-hats
    for i in [1, 3, 5, 7, 9, 11, 13, 15] {
        if rng.gen_range(0..100) < 60 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }

    // Open hat accents with more variation
    if rng.gen_range(0..100) < 50 {
        let open_pos = *[6, 14].choose(&mut rng).unwrap();
        bar[open_pos].push(DrumHit::HiHatOpen);
    }
    
    // Occasional tom fills for variation
    if rng.gen_range(0..100) < 25 {
        let tom_pos = rng.gen_range(0..16);
        bar[tom_pos].push(DrumHit::Tom);
    }
    
    // Rim shots for accent
    if rng.gen_range(0..100) < 20 {
        let rim_pos = *[2, 6, 10, 14].choose(&mut rng).unwrap();
        bar[rim_pos].push(DrumHit::RimShot);
    }

    bar
}

/// Generate variant funk bar with different pattern
fn generate_funk_bar_variant() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Variant: more syncopated kick pattern
    bar[0].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 80 {
        bar[5].push(DrumHit::Kick); // Different syncopation
    }
    bar[8].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 50 {
        bar[11].push(DrumHit::Kick);
    }

    // Snare on 2 and 4
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Sparse hi-hats
    for i in [0, 4, 8, 12] {
        bar[i].push(DrumHit::HiHatClosed);
    }

    bar
}

/// Generate variant jazz bar
fn generate_jazz_bar_variant() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Variant: more sparse
    bar[0].push(DrumHit::Kick);
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Sparse ride pattern
    for i in [0, 6, 12] {
        if rng.gen_range(0..100) < 70 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }

    bar
}

/// Generate variant electro-swing bar
fn generate_electro_swing_bar_variant() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Variant: less four-on-the-floor, more syncopation
    bar[0].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 60 {
        bar[4].push(DrumHit::Kick);
    }

    bar[4].push(DrumHit::Clap);
    bar[12].push(DrumHit::Clap);

    // Sparse hi-hats
    for i in (0..16).step_by(4) {
        bar[i].push(DrumHit::HiHatClosed);
    }

    bar
}

/// Generate variant hip-hop bar
fn generate_hiphop_bar_variant() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Variant: more sparse kick pattern
    bar[0].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 70 {
        bar[8].push(DrumHit::Kick);
    }
    if rng.gen_range(0..100) < 40 {
        bar[14].push(DrumHit::Kick);
    }

    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Sparse 8th note hi-hats
    for i in [0, 2, 4, 6, 8, 10, 12, 14] {
        bar[i].push(DrumHit::HiHatClosed);
    }

    bar
}

/// Generate variant rock bar
fn generate_rock_bar_variant() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Variant: less four-on-the-floor
    bar[0].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 60 {
        bar[4].push(DrumHit::Kick);
    }
    if rng.gen_range(0..100) < 40 {
        bar[12].push(DrumHit::Kick);
    }

    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // 8th note hi-hats
    for i in [0, 2, 4, 6, 8, 10, 12, 14] {
        bar[i].push(DrumHit::HiHatClosed);
    }

    bar
}

/// Generate variant lofi bar
fn generate_lofi_bar_variant() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Variant: even sparser
    bar[0].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 50 {
        bar[8].push(DrumHit::Kick);
    }

    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Very sparse hi-hats
    for i in [0, 8] {
        if rng.gen_range(0..100) < 60 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }

    bar
}

/// Generate variant DnB bar
fn generate_dnb_bar_variant() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Variant: different breakbeat pattern
    bar[0].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 80 {
        bar[6].push(DrumHit::Kick);
    }
    if rng.gen_range(0..100) < 60 {
        bar[10].push(DrumHit::Kick);
    }

    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Sparse hi-hats
    for i in 0..16 {
        if rng.gen_range(0..100) < 25 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }

    bar
}

/// Generate a drum fill for transitions between sections
fn generate_drum_fill(style: GrooveStyle, rng: &mut impl Rng) -> DrumPattern {
    let mut fill = vec![vec![DrumHit::Rest]; 8]; // 2 beats worth of fill

    let fill_type = rng.gen_range(0..100);

    match style {
        GrooveStyle::Rock | GrooveStyle::HipHop => {
            // Tom rolls and crash
            if fill_type < 50 {
                // Tom roll with crash
                fill[0].push(DrumHit::Tom);
                fill[1].push(DrumHit::Tom);
                fill[2].push(DrumHit::Tom);
                fill[3].push(DrumHit::Tom);
                fill[4].push(DrumHit::Snare);
                fill[5].push(DrumHit::Snare);
                fill[6].push(DrumHit::Snare);
                fill[7].push(DrumHit::Crash);
                fill[7].push(DrumHit::Kick);
            } else {
                // Snare roll
                for i in 0..8 {
                    if i % 2 == 0 {
                        fill[i].push(DrumHit::Snare);
                    }
                }
                fill[7].push(DrumHit::Crash);
            }
        }
        GrooveStyle::DnB | GrooveStyle::Dubstep => {
            // Fast snare roll
            for i in 0..8 {
                fill[i].push(DrumHit::Snare);
                if i % 2 == 0 {
                    fill[i].push(DrumHit::HiHatClosed);
                }
            }
            fill[7].push(DrumHit::Crash);
        }
        _ => {
            // Gentle fill for jazz/funk/lofi
            fill[0].push(DrumHit::Snare);
            fill[2].push(DrumHit::Tom);
            fill[4].push(DrumHit::Snare);
            fill[6].push(DrumHit::Tom);
            fill[7].push(DrumHit::RimShot);
        }
    }

    fill
}

/// Generate additional funk variant patterns for more variety
fn generate_funk_bar_variant2() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Ghost note heavy funk pattern
    bar[0].push(DrumHit::Kick);
    bar[6].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    bar[14].push(DrumHit::Kick);

    // Snare with many ghost notes
    for i in [2, 4, 6, 10, 12, 14] {
        if rng.gen_range(0..100) < 70 {
            bar[i].push(DrumHit::Snare);
        }
    }

    // Sparse hi-hats
    for i in [0, 4, 8, 12] {
        bar[i].push(DrumHit::HiHatClosed);
    }

    bar
}

fn generate_rock_bar_variant2() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Double kick metal pattern
    for i in [0, 1, 4, 5, 8, 9, 12, 13] {
        bar[i].push(DrumHit::Kick);
    }

    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Ride pattern instead of hi-hat
    for i in 0..16 {
        if i % 2 == 0 {
            bar[i].push(DrumHit::Ride);
        }
    }

    bar
}

fn generate_jazz_bar_variant2() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Brush pattern - very sparse
    bar[0].push(DrumHit::Kick);
    bar[4].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);

    // Ride cymbal instead of hi-hat
    for i in [0, 3, 6, 9, 12, 15] {
        if rng.gen_range(0..100) < 60 {
            bar[i].push(DrumHit::Ride);
        }
    }

    bar
}

fn generate_hiphop_bar_variant2() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Boom bap with rim shots
    bar[0].push(DrumHit::Kick);
    bar[8].push(DrumHit::Kick);
    if rng.gen_range(0..100) < 50 {
        bar[12].push(DrumHit::Kick);
    }

    bar[4].push(DrumHit::Snare);
    bar[4].push(DrumHit::RimShot); // Layered
    bar[12].push(DrumHit::Snare);
    bar[12].push(DrumHit::RimShot); // Layered

    // Sparse hi-hats
    for i in [2, 6, 10, 14] {
        bar[i].push(DrumHit::HiHatClosed);
    }

    bar
}

fn generate_dnb_bar_variant2() -> DrumPattern {
    let mut rng = rand::thread_rng();
    let mut bar = vec![vec![DrumHit::Rest]; 16];

    // Amen break style
    bar[0].push(DrumHit::Kick);
    bar[3].push(DrumHit::Kick);
    bar[7].push(DrumHit::Kick);
    bar[10].push(DrumHit::Kick);

    bar[4].push(DrumHit::Snare);
    bar[6].push(DrumHit::Snare);
    bar[12].push(DrumHit::Snare);
    bar[14].push(DrumHit::Snare);

    // Sparse hi-hats
    for i in 0..16 {
        if rng.gen_range(0..100) < 20 {
            bar[i].push(DrumHit::HiHatClosed);
        }
    }

    bar
}

/// Choose a random groove style weighted toward lofi/chill
#[allow(dead_code)]
pub fn random_groove_style() -> GrooveStyle {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..100);

    match roll {
        0..=50 => GrooveStyle::Lofi,          // Most common (lofi focus)
        51..=65 => GrooveStyle::Jazz,         // Jazzy lofi
        66..=80 => GrooveStyle::HipHop,       // Hip-hop beats
        81..=90 => GrooveStyle::ElectroSwing, // Occasional variety
        91..=95 => GrooveStyle::Funk,         // Rare funky element
        _ => GrooveStyle::Rock,               // Very rare
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
                if roll < 85 {
                    DrumSound::Kick
                } else {
                    DrumSound::HiHat
                }
            }
            4 => {
                if roll < 80 {
                    DrumSound::Snare
                } else {
                    DrumSound::HiHat
                }
            }
            8 => {
                if roll < 70 {
                    DrumSound::Kick
                } else {
                    DrumSound::HiHat
                }
            }
            12 => {
                if roll < 80 {
                    DrumSound::Snare
                } else {
                    DrumSound::HiHat
                }
            }
            2 | 6 | 10 | 14 => {
                if roll < 90 {
                    DrumSound::HiHat
                } else {
                    DrumSound::Rest
                }
            }
            _ => {
                if roll < 60 {
                    DrumSound::HiHat
                } else if roll < 90 {
                    DrumSound::Rest
                } else {
                    DrumSound::Snare
                }
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

#[allow(deprecated)]
fn legacy_pattern_from_create_beat(num_bars: usize) -> Option<DrumPattern> {
    let total_steps = (num_bars * 16) as u32;
    let legacy_sequence = create_beat(total_steps).ok()?;

    let mut pattern = Vec::with_capacity(legacy_sequence.len());
    for sound in legacy_sequence {
        let mut hits = Vec::new();
        match sound {
            DrumSound::Kick => hits.push(DrumHit::Kick),
            DrumSound::Snare => hits.push(DrumHit::Snare),
            DrumSound::HiHat => hits.push(DrumHit::HiHatClosed),
            DrumSound::Rest => {}
        }

        if hits.is_empty() {
            pattern.push(vec![DrumHit::Rest]);
        } else {
            pattern.push(hits);
        }
    }

    Some(pattern)
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
