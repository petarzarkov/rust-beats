/// Audio rendering functions for generating tracks from arrangements
use crate::composition::{Arrangement, Genre, Key, Tempo};
use crate::composition::beat_maker::{DrumKit, DrumHit, GrooveStyle, generate_drum_pattern};
use crate::composition::arranger::DrumComplexity;
use crate::synthesis::{
    SAMPLE_RATE,
    generate_kick, generate_snare, generate_hihat, generate_clap,
    generate_pads,
    generate_rock_bassline, generate_dubstep_bassline, generate_dnb_bassline,
    generate_riser, generate_downlifter, generate_crash, generate_impact,
};
use crate::synthesis::drums::{generate_rock_kick, generate_dubstep_kick, generate_dnb_snare, generate_rock_snare};
use crate::synthesis::bass::{generate_synth_bass_note, generate_upright_bass_note, generate_finger_bass_note, generate_slap_bass_note, generate_bass_note};
use crate::synthesis::melody::generate_melody_with_style;
use crate::synthesis::percussion::{generate_tambourine, generate_cowbell, generate_bongo, generate_woodblock, generate_triangle_perc};
use crate::composition::music_theory::{Chord, midi_to_freq};
use crate::config;
use rand::Rng;

pub type DrumPattern = Vec<Vec<DrumHit>>;

/// Render drums with arrangement awareness
pub fn render_arranged_drums(
    arrangement: &Arrangement,
    base_groove: GrooveStyle,
    bpm: f32,
    drum_kit: DrumKit,
    genre: &Genre,
) -> Vec<f32> {
    let mut all_drums = Vec::new();
    let mut bar_idx = 0;
    let mut rng = rand::thread_rng();
    
    for (section_idx, (section, bars)) in arrangement.sections.iter().enumerate() {
        let complexity = Arrangement::get_drum_complexity(*section);
        let intensity = Arrangement::get_section_intensity(*section);
        
        // Allow groove style variation per section (30% chance to change)
        let groove = if rng.gen_range(0..100) < 30 {
            // Map genre to appropriate groove styles
            match genre {
                Genre::Rock => GrooveStyle::Rock,
                Genre::Dubstep => GrooveStyle::Dubstep,
                Genre::DnB => GrooveStyle::DnB,
                Genre::Jazz => GrooveStyle::Jazz,
                Genre::Funk => GrooveStyle::Funk,
                Genre::HipHop => GrooveStyle::HipHop,
                Genre::ElectroSwing => GrooveStyle::ElectroSwing,
                Genre::Lofi => GrooveStyle::Lofi,
            }
        } else {
            base_groove
        };
        
        // Check for transitions
        let is_transition = arrangement.should_add_fill(bar_idx + bars - 1);
        let needs_buildup = if section_idx > 0 {
            let prev_section = arrangement.sections[section_idx - 1].0;
            Arrangement::needs_buildup(prev_section, *section)
        } else {
            false
        };
        let needs_breakdown = if section_idx > 0 {
            let prev_section = arrangement.sections[section_idx - 1].0;
            Arrangement::needs_breakdown(prev_section, *section)
        } else {
            false
        };
        
        // Generate pattern with complexity awareness
        let pattern = generate_drum_pattern_with_complexity(
            groove,
            *bars,
            complexity,
            is_transition,
            needs_buildup,
            needs_breakdown,
        );
        
        // Render with kit-specific sounds and swing
        let drums = render_drum_pattern_with_kit(&pattern, bpm, intensity, drum_kit, genre);
        all_drums.extend(drums);
        
        bar_idx += bars;
    }
    
    all_drums
}

/// Generate drum pattern with complexity and transition awareness
pub fn generate_drum_pattern_with_complexity(
    groove: GrooveStyle,
    bars: usize,
    complexity: DrumComplexity,
    add_fill: bool,
    needs_buildup: bool,
    needs_breakdown: bool,
) -> DrumPattern {
    let mut pattern = generate_drum_pattern(groove, bars);
    let mut rng = rand::thread_rng();
    
    // Adjust pattern based on complexity
    match complexity {
        DrumComplexity::Simple => {
            // Simplify: remove some hi-hats and keep only kick/snare
            for step in pattern.iter_mut() {
                step.retain(|hit| matches!(hit, DrumHit::Kick | DrumHit::Snare | DrumHit::Rest));
            }
        }
        DrumComplexity::Medium => {
            // Keep as-is (already medium complexity)
        }
        DrumComplexity::Complex => {
            // Add more variation: occasional extra hits
            for step in pattern.iter_mut() {
                if rng.gen_range(0..100) < 15 {
                    if !step.contains(&DrumHit::HiHatClosed) {
                        step.push(DrumHit::HiHatClosed);
                    }
                }
            }
        }
    }
    
    // Add fill on last bar if needed
    if add_fill {
        let fill_start = pattern.len().saturating_sub(4); // Last 4 steps
        for i in fill_start..pattern.len() {
            if i % 2 == 0 {
                pattern[i].push(DrumHit::Snare);
            }
            if i % 3 == 0 {
                pattern[i].push(DrumHit::HiHatClosed);
            }
        }
    }
    
    // Add buildup: gradually increase density
    if needs_buildup {
        let buildup_start = pattern.len().saturating_sub(8); // Last 8 steps
        for (i, step) in pattern.iter_mut().enumerate().skip(buildup_start) {
            let progress = (i - buildup_start) as f32 / 8.0;
            if rng.gen_range(0.0..1.0) < progress {
                if !step.contains(&DrumHit::HiHatClosed) {
                    step.push(DrumHit::HiHatClosed);
                }
            }
        }
    }
    
    // Add breakdown: gradually decrease density
    if needs_breakdown {
        let breakdown_length = 8.min(pattern.len());
        for (i, step) in pattern.iter_mut().enumerate().take(breakdown_length) {
            let progress = i as f32 / breakdown_length as f32;
            if rng.gen_range(0.0..1.0) < (1.0 - progress) {
                step.retain(|hit| matches!(hit, DrumHit::Kick | DrumHit::Snare | DrumHit::Rest));
            }
        }
    }
    
    pattern
}

/// Render drum pattern with kit-specific sounds and swing
pub fn render_drum_pattern_with_kit(
    pattern: &[Vec<DrumHit>],
    bpm: f32,
    intensity: f32,
    drum_kit: DrumKit,
    genre: &Genre,
) -> Vec<f32> {
    let beat_duration = 60.0 / bpm;
    let sixteenth_duration = beat_duration / 4.0;
    
    // Genre-specific swing amount (applied to off-beats)
    let swing_amount = match genre {
        Genre::Lofi => 0.30,      // Heavy swing for laid-back feel
        Genre::Jazz => 0.35,      // Even more swing for jazz
        Genre::HipHop => 0.25,    // Medium swing
        Genre::Funk => 0.20,      // Subtle funk groove
        Genre::ElectroSwing => 0.28, // Swing for electro-swing
        Genre::DnB => 0.10,       // Slight swing for variation
        _ => 0.05,                // Minimal swing for rock/dubstep
    };
    
    let total_duration = pattern.len() as f32 * sixteenth_duration;
    let total_samples = (total_duration * SAMPLE_RATE() as f32) as usize;
    
    let mut output = vec![0.0; total_samples];
    
    for (step_idx, hits) in pattern.iter().enumerate() {
        // Apply swing: delay every second 16th note (odd indices)
        let swing_offset = if step_idx % 2 == 1 {
            sixteenth_duration * swing_amount
        } else {
            0.0
        };
        
        let step_time = step_idx as f32 * sixteenth_duration + swing_offset;
        let start_sample = (step_time * SAMPLE_RATE() as f32) as usize;
        
        for hit in hits {
            let drum_samples = match (hit, drum_kit) {
                (DrumHit::Kick, DrumKit::Rock) => generate_rock_kick(0.8 * intensity),
                (DrumHit::Kick, DrumKit::Electronic808) | (DrumHit::Kick, DrumKit::HipHop) => {
                    // Use dubstep kick for electronic kits
                    generate_dubstep_kick(0.8 * intensity)
                }
                (DrumHit::Kick, _) => generate_kick(0.8 * intensity),
                
                (DrumHit::Snare, DrumKit::Rock) => generate_rock_snare(0.7 * intensity),
                (DrumHit::Snare, DrumKit::Electronic808) | (DrumHit::Snare, DrumKit::HipHop) => {
                    // Use DnB snare for electronic kits
                    generate_dnb_snare(0.7 * intensity)
                }
                (DrumHit::Snare, _) => generate_snare(0.7 * intensity),
                
                (DrumHit::HiHatClosed, _) => generate_hihat(0.4 * intensity, false),
                (DrumHit::HiHatOpen, _) => generate_hihat(0.5 * intensity, true),
                (DrumHit::Clap, _) => generate_clap(0.6 * intensity),
                (DrumHit::Conga, _) => crate::synthesis::generate_conga(200.0, 0.5 * intensity),
                (DrumHit::Shaker, _) => crate::synthesis::generate_shaker(0.3 * intensity),
                (DrumHit::Rest, _) => continue,
            };
            
            // Mix the drum hit into the output
            for (i, &sample) in drum_samples.iter().enumerate() {
                let idx = start_sample + i;
                if idx < output.len() {
                    output[idx] += sample;
                }
            }
        }
    }
    
    output
}

/// Render bass with arrangement awareness and genre-specific routing
pub fn render_arranged_bass(
    arrangement: &Arrangement,
    chords: &[Chord],
    bpm: f32,
    genre: &Genre,
    bass_drop_cfg: &config::BassDropConfig,
    genre_config: &crate::composition::genre::GenreConfig,
) -> Vec<f32> {
    let mut all_bass = Vec::new();
    let mut bar_idx = 0;
    
    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords
            .iter()
            .skip(bar_idx)
            .take(*bars)
            .cloned()
            .collect();
        
        if Arrangement::section_has_heavy_bass(*section) {
            // Route to genre-specific bass generator
            let bass = match genre {
                Genre::Rock => generate_rock_bassline(&section_chords, bpm, *bars, bass_drop_cfg),
                Genre::Dubstep => generate_dubstep_bassline(&section_chords, bpm, *bars),
                Genre::DnB => generate_dnb_bassline(&section_chords, bpm, *bars, bass_drop_cfg),
                Genre::Funk => {
                    // Use slap bass for funk sections
                    generate_bassline_with_style(
                        &section_chords,
                        bpm,
                        *bars,
                        bass_drop_cfg,
                        Some(crate::composition::genre::BassStyle::Slap),
                    )
                }
                Genre::Jazz => {
                    // Use upright bass for jazz
                    generate_bassline_with_style(
                        &section_chords,
                        bpm,
                        *bars,
                        bass_drop_cfg,
                        Some(crate::composition::genre::BassStyle::Upright),
                    )
                }
                _ => {
                    // Use genre config bass style
                    generate_bassline_with_style(
                        &section_chords,
                        bpm,
                        *bars,
                        bass_drop_cfg,
                        Some(genre_config.bass_style),
                    )
                }
            };
            all_bass.extend(bass);
        } else {
            // Light bass or no bass - just add silence
            let bar_duration = 60.0 / bpm * 4.0;
            let silence_samples = (bar_duration * *bars as f32 * SAMPLE_RATE() as f32) as usize;
            all_bass.extend(vec![0.0; silence_samples]);
        }
        
        bar_idx += bars;
    }
    
    all_bass
}

/// Generate bassline with specific bass style
pub fn generate_bassline_with_style(
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    bass_drop_cfg: &config::BassDropConfig,
    style: Option<crate::composition::genre::BassStyle>,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;
    
    let mut bassline = Vec::new();
    let mut rng = rand::thread_rng();
    
    // Select bass note generator based on style
    let bass_note_fn: fn(f32, f32, f32) -> Vec<f32> = match style {
        Some(crate::composition::genre::BassStyle::Synth) => generate_synth_bass_note,
        Some(crate::composition::genre::BassStyle::Upright) => generate_upright_bass_note,
        Some(crate::composition::genre::BassStyle::Finger) => generate_finger_bass_note,
        Some(crate::composition::genre::BassStyle::Slap) => generate_slap_bass_note,
        _ => generate_bass_note, // Default
    };
    
    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        let root_note = chord.root;
        let frequency = midi_to_freq(root_note);
        
        // Generate funky bass pattern for this bar
        let mut pattern = generate_funk_bass_pattern_with_style(frequency, bar_duration, bass_note_fn);
        
        // Add occasional bass drop using config values
        let should_drop = if bar_idx > 0 && bar_idx % 8 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.default_chance_8th_bar
        } else if bar_idx > 0 && bar_idx % 12 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.default_chance_12th_bar
        } else {
            false
        };
        
        if should_drop {
            // Add a sub-bass drop on beat 1 (start of bar)
            let drop_freq = frequency * 0.5; // One octave down
            let drop_duration = beat_duration * bass_drop_cfg.default_duration_beats;
            let drop = crate::synthesis::bass::generate_sub_bass(drop_freq, drop_duration, bass_drop_cfg.amplitude);
            
            // Mix the drop into the pattern
            for (i, &drop_sample) in drop.iter().enumerate() {
                if i < pattern.len() {
                    pattern[i] += drop_sample;
                }
            }
        }
        
        bassline.extend(pattern);
    }
    
    bassline
}

/// Generate a funky bass pattern with custom bass note generator
pub fn generate_funk_bass_pattern_with_style(
    root_freq: f32,
    bar_duration: f32,
    bass_note_fn: fn(f32, f32, f32) -> Vec<f32>,
) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let note_duration = bar_duration / 8.0; // 8th notes
    
    let mut pattern = Vec::new();
    
    // Typical funk bass pattern: emphasize 1 and 3, add syncopation
    let hits = vec![
        (0.0, 1.0, 1.0),      // Beat 1 - strong
        (0.5, 0.5, 0.7),      // Off-beat
        (1.0, 0.8, 0.8),      // Beat 2
        (1.75, 0.4, 0.6),     // Syncopation
        (2.0, 1.0, 1.0),      // Beat 3 - strong
        (2.5, 0.5, 0.7),      // Off-beat
        (3.0, 0.6, 0.8),      // Beat 4
        (3.5, 0.4, 0.6),      // Off-beat
    ];
    
    for (beat_pos, duration_mult, velocity) in hits {
        let start_time = beat_pos * (bar_duration / 4.0);
        let duration = note_duration * duration_mult;
        
        // Mostly stick to root note for consistency
        let freq_mult = match rng.gen_range(0..100) {
            0..=92 => 1.0,        // Root (most of the time)
            93..=97 => 0.5,       // Octave down (rare)
            _ => 1.5,             // Fifth (very rare)
        };
        
        let note = bass_note_fn(root_freq * freq_mult, duration, velocity * 0.65);
        
        // Add to pattern at the right position
        let start_sample = (start_time * SAMPLE_RATE() as f32) as usize;
        if start_sample + note.len() > pattern.len() {
            pattern.resize(start_sample + note.len(), 0.0);
        }
        
        for (i, &sample) in note.iter().enumerate() {
            let idx = start_sample + i;
            if idx < pattern.len() {
                pattern[idx] += sample;
            }
        }
    }
    
    // Fill to full bar duration
    let total_samples = (bar_duration * SAMPLE_RATE() as f32) as usize;
    pattern.resize(total_samples, 0.0);
    
    pattern
}

/// Generate pads with arrangement awareness
pub fn generate_pads_with_arrangement(
    arrangement: &Arrangement,
    chords: &[Chord],
    bpm: f32,
    num_bars: usize,
) -> Vec<f32> {
    // Temporarily disable drones - they might be causing the scratching issue
    // Just use regular pads for now
    generate_pads(chords, bpm, num_bars)
}

/// Render melody with arrangement awareness
pub fn render_arranged_melody(
    arrangement: &Arrangement,
    key: &Key,
    chords: &[Chord],
    bpm: f32,
    melody_cfg: &config::MelodyConfig,
    genre: &Genre,
) -> Vec<f32> {
    let mut all_melody = Vec::new();
    let mut bar_idx = 0;
    
    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords
            .iter()
            .skip(bar_idx)
            .take(*bars)
            .cloned()
            .collect();
        
        if Arrangement::section_has_melody(*section) {
            // Use new melody generation with style awareness
            let melody = generate_melody_with_style(
                key,
                &section_chords,
                bpm,
                *bars,
                melody_cfg,
                Some(*section),
                Some(*genre),
            );
            all_melody.extend(melody);
        } else {
            // No melody - just add silence
            let bar_duration = 60.0 / bpm * 4.0;
            let silence_samples = (bar_duration * *bars as f32 * SAMPLE_RATE() as f32) as usize;
            all_melody.extend(vec![0.0; silence_samples]);
        }
        
        bar_idx += bars;
    }
    
    all_melody
}

/// Render transition FX track (risers, crashes, downlifters)
pub fn render_fx_track(arrangement: &Arrangement, bpm: f32) -> Vec<f32> {
    let bar_duration = 60.0 / bpm * 4.0;
    let total_samples = (bar_duration * arrangement.total_bars as f32 * SAMPLE_RATE() as f32) as usize;
    let mut fx_track = vec![0.0; total_samples];
    
    let mut current_bar = 0;
    
    for (section_idx, (section, bars)) in arrangement.sections.iter().enumerate() {
        let section_start_sample = (current_bar as f32 * bar_duration * SAMPLE_RATE() as f32) as usize;
        
        // Check if we need a buildup to this section
        if section_idx > 0 {
            let prev_section = arrangement.sections[section_idx - 1].0;
            
            if Arrangement::needs_buildup(prev_section, *section) {
                // Add riser before section starts (2 bars)
                let riser_duration = bar_duration * 2.0;
                let riser = generate_riser(riser_duration);
                let riser_start = section_start_sample.saturating_sub(riser.len());
                
                for (i, &sample) in riser.iter().enumerate() {
                    let idx = riser_start + i;
                    if idx < fx_track.len() {
                        fx_track[idx] += sample;
                    }
                }
            }
            
            if Arrangement::needs_breakdown(prev_section, *section) {
                // Add downlifter at start of section (1 bar)
                let downlifter_duration = bar_duration;
                let downlifter = generate_downlifter(downlifter_duration);
                
                for (i, &sample) in downlifter.iter().enumerate() {
                    let idx = section_start_sample + i;
                    if idx < fx_track.len() {
                        fx_track[idx] += sample;
                    }
                }
            }
        }
        
        // Add crash at the start of important sections (Chorus, Bridge)
        if matches!(*section, crate::composition::Section::Chorus | crate::composition::Section::Bridge) {
            let crash = generate_crash(2.0);
            
            for (i, &sample) in crash.iter().enumerate() {
                let idx = section_start_sample + i;
                if idx < fx_track.len() {
                    fx_track[idx] += sample * 0.5; // Reduced from 0.7 to 0.5
                }
            }
        }
        
        // Add impact at very start of Chorus for emphasis
        if *section == crate::composition::Section::Chorus {
            let impact = generate_impact();
            
            for (i, &sample) in impact.iter().enumerate() {
                let idx = section_start_sample + i;
                if idx < fx_track.len() {
                    fx_track[idx] += sample * 0.6; // Reduced from 0.8 to 0.6
                }
            }
        }
        
        current_bar += bars;
    }
    
    fx_track
}

/// Add percussion track if enabled
pub fn add_percussion_track(
    percussion_type: &str,
    arrangement: &Arrangement,
    tempo: &Tempo,
) -> Option<Vec<f32>> {
    if percussion_type == "None" {
        return None;
    }
    
    let mut percussion = Vec::new();
    let mut rng = rand::thread_rng();
    let bar_duration = tempo.bar_duration();
    let beat_duration = tempo.beat_duration();
    
    for (section, bars) in &arrangement.sections {
        if Arrangement::section_has_melody(*section) {
            // Add percussion to sections with melody
            for _bar_idx in 0..*bars {
                let bar_samples = (bar_duration * SAMPLE_RATE() as f32) as usize;
                let mut bar = vec![0.0; bar_samples];
                
                // Add percussion hits on beats
                for beat in 0..4 {
                    if rng.gen_range(0..100) < 40 {
                        // 40% chance per beat
                        let hit_time = beat as f32 * beat_duration;
                        let hit_sample = (hit_time * SAMPLE_RATE() as f32) as usize;
                        
                        let hit = match percussion_type {
                            "Tambourine" => generate_tambourine(0.4),
                            "Cowbell" => generate_cowbell(0.5),
                            "Bongo" => generate_bongo(rng.gen_range(0..100) < 50, 0.4),
                            "Woodblock" => generate_woodblock(0.4),
                            _ => generate_triangle_perc(0.3),
                        };
                        
                        for (i, &sample) in hit.iter().enumerate() {
                            let idx = hit_sample + i;
                            if idx < bar.len() {
                                bar[idx] += sample;
                            }
                        }
                    }
                }
                
                percussion.extend(bar);
            }
        } else {
            // Silence for sections without melody
            let silence_samples = (bar_duration * *bars as f32 * SAMPLE_RATE() as f32) as usize;
            percussion.extend(vec![0.0; silence_samples]);
        }
    }
    
    Some(percussion)
}

