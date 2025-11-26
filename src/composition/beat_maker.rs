use super::genre::Genre;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrumHit {
    Kick,
    Snare,
    HiHatClosed,
    HiHatOpen,
    Crash,
    Ride,
    China, // Added for metal
    Tom,
    Rest,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GrooveStyle {
    SwampMetal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrumKit {
    Metal,
}

pub struct BeatMaker {
    pub genre: Genre,
}

impl BeatMaker {
    pub fn new(genre: Genre) -> Self {
        BeatMaker { genre }
    }

    pub fn generate_beat(&self, tempo: f32, bars: usize) -> Vec<(f32, DrumHit, f32)> {
        let mut beat = Vec::new();
        let _rng = rand::thread_rng();

        for bar in 0..bars {
            let bar_offset = bar as f32 * 4.0;
            
            // Determine section intensity
            let is_fill_bar = (bar + 1) % 4 == 0;
            let is_heavy_section = bar >= 4; // Second half is heavier

            let pattern = if is_fill_bar {
                self.generate_fill(tempo)
            } else {
                self.generate_swamp_metal_pattern(is_heavy_section)
            };

            for (pos, hit, velocity) in pattern {
                beat.push((bar_offset + pos, hit, velocity));
            }
        }

        beat
    }

    fn generate_swamp_metal_pattern(&self, heavy: bool) -> Vec<(f32, DrumHit, f32)> {
        let mut pattern = Vec::new();
        let mut rng = rand::thread_rng();

        // Swamp metal: Slow, heavy, sludge-like groove
        // Half-time feel often, but with driving double kicks

        // Basic Kick/Snare skeleton
        // Beat 1: Kick
        // Beat 2: Snare (Heavy)
        // Beat 3: Kick
        // Beat 4: Snare (Heavy)
        
        // Kick pattern
        pattern.push((0.0, DrumHit::Kick, 1.0));
        
        if heavy {
            // Double kick bursts
            pattern.push((0.25, DrumHit::Kick, 0.9)); // 16th note kick
            pattern.push((2.25, DrumHit::Kick, 0.9));
            pattern.push((2.5, DrumHit::Kick, 0.8));
        } else {
            // Slower, more spacious kicks
            if rng.gen_bool(0.5) {
                pattern.push((2.5, DrumHit::Kick, 0.8)); // Kick on 'and' of 3
            }
        }

        // Snare pattern (Backbeat)
        pattern.push((1.0, DrumHit::Snare, 1.0));
        pattern.push((3.0, DrumHit::Snare, 1.0));
        
        // Ghost notes on snare
        if rng.gen_bool(0.3) {
            pattern.push((1.75, DrumHit::Snare, 0.4));
        }
        if rng.gen_bool(0.3) {
            pattern.push((3.75, DrumHit::Snare, 0.4));
        }

        // Cymbals
        // Heavy: Ride or China
        // Light: Loose HiHat
        
        if heavy {
            // Quarter notes on China or Ride
            let cymbal = if rng.gen_bool(0.3) { DrumHit::China } else { DrumHit::Ride };
            for i in 0..4 {
                pattern.push((i as f32, cymbal, 0.8));
            }
        } else {
            // 8th notes on HiHat (Slushy/Open)
            for i in 0..8 {
                let pos = i as f32 * 0.5;
                let hit = if i % 2 == 0 { DrumHit::HiHatOpen } else { DrumHit::HiHatClosed };
                pattern.push((pos, hit, 0.7 + rng.gen_range(-0.1..0.1)));
            }
        }
        
        // Occasional Crash on 1
        if rng.gen_bool(0.1) {
            pattern.push((0.0, DrumHit::Crash, 0.9));
        }

        pattern
    }

    fn generate_fill(&self, _tempo: f32) -> Vec<(f32, DrumHit, f32)> {
        let mut pattern = Vec::new();
        let mut rng = rand::thread_rng();

        // Metal fill: Tom rolls, snare flams, double kick runs
        
        // First half: Groove
        pattern.push((0.0, DrumHit::Kick, 1.0));
        pattern.push((1.0, DrumHit::Snare, 1.0));
        pattern.push((0.0, DrumHit::Crash, 0.9));

        // Second half: Fill
        let fill_type = rng.gen_range(0..3);
        
        match fill_type {
            0 => {
                // Tom Roll
                pattern.push((2.0, DrumHit::Tom, 0.9));
                pattern.push((2.25, DrumHit::Tom, 0.8));
                pattern.push((2.5, DrumHit::Tom, 0.9));
                pattern.push((2.75, DrumHit::Tom, 0.8));
                pattern.push((3.0, DrumHit::Snare, 1.0)); // Flam finish
                pattern.push((3.5, DrumHit::Kick, 1.0));
            },
            1 => {
                // Snare build
                for i in 0..8 {
                    let pos = 2.0 + i as f32 * 0.25;
                    let vel = 0.5 + (i as f32 / 16.0); // Crescendo
                    pattern.push((pos, DrumHit::Snare, vel));
                }
            },
            _ => {
                // Double kick run
                for i in 0..8 {
                    let pos = 2.0 + i as f32 * 0.25;
                    pattern.push((pos, DrumHit::Kick, 0.9));
                    if i % 2 == 0 {
                        pattern.push((pos, DrumHit::China, 0.7));
                    }
                }
            }
        }
        
        // Always crash on next 1 (handled by next bar generation usually, but let's end with impact)
        // pattern.push((4.0, DrumHit::Crash, 1.0)); // BeatMaker handles bars individually

        pattern
    }
}

pub fn select_preferred_drum_kit(_genre: Genre) -> DrumKit {
    DrumKit::Metal
}

pub fn get_groove_style(genre: Genre) -> GrooveStyle {
    match genre {
        Genre::SwampMetal => GrooveStyle::SwampMetal,
    }
}
