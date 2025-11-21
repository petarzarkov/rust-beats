use crate::composition::arranger::DrumComplexity;
use crate::composition::beat_maker::{generate_drum_pattern, DrumHit, DrumKit, GrooveStyle};
use crate::composition::genre::{GenreConfig, MelodyDensity};
use crate::composition::music_theory::{midi_to_freq, Chord};
/// Audio rendering functions for generating tracks from arrangements
use crate::composition::{Arrangement, Genre, Key, Section, Tempo};
use crate::config;
use crate::synthesis::bass::{
    generate_bass_note, generate_finger_bass_note, generate_funk_bass_pattern,
    generate_slap_bass_note, generate_synth_bass_note, generate_upright_bass_note,
};
use crate::synthesis::drums::{
    generate_dnb_snare, generate_dubstep_kick, generate_rock_kick, generate_rock_snare,
};
use crate::synthesis::melody::{generate_melody_with_style_and_instrument, InstrumentType};
use crate::synthesis::percussion::{
    generate_bongo, generate_cowbell, generate_tambourine, generate_triangle_perc,
    generate_woodblock,
};
use crate::synthesis::drums::{DrumSoundParams, generate_kick_with_params, generate_snare_with_params, generate_hihat_with_params};
use crate::synthesis::{
    generate_clap, generate_crash, generate_dnb_bassline, generate_downlifter, generate_drone,
    generate_dubstep_bassline, generate_impact, generate_pads,
    generate_riser, generate_rock_bassline, get_sample_rate,
};
use rand::{seq::SliceRandom, Rng};

pub type DrumPattern = Vec<Vec<DrumHit>>;


/// Render drums with arrangement awareness
pub fn render_arranged_drums(
    arrangement: &Arrangement,
    base_groove: GrooveStyle,
    bpm: f32,
    drum_kit: DrumKit,
    genre: &Genre,
    genre_config: &GenreConfig,
) -> Vec<f32> {
    let mut all_drums = Vec::new();
    let mut bar_idx = 0;
    let mut rng = rand::thread_rng();
    
    // Generate per-song drum sound parameters for consistent variation
    let drum_params = DrumSoundParams::generate();

    for (section_idx, (section, bars)) in arrangement.sections.iter().enumerate() {
        let complexity = Arrangement::get_drum_complexity(*section);
        let mut intensity = Arrangement::get_section_intensity(*section);

        if arrangement.is_section_start(bar_idx) {
            intensity *= 0.95;
        }
        if let Some(current_section) = arrangement.get_section_at_bar(bar_idx) {
            if current_section == Section::Bridge {
                intensity *= 1.05;
            }
        }

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
        let mut pattern = generate_drum_pattern_with_complexity(
            groove,
            *bars,
            complexity,
            is_transition,
            needs_buildup,
            needs_breakdown,
        );

        if let Some((_from, to)) = arrangement.get_transition(bar_idx + bars) {
            if matches!(to, Section::Chorus | Section::Bridge) {
                for step in pattern.iter_mut().rev().take(4) {
                    step.push(DrumHit::Snare);
                }
            }
        }

        let mut section_kit = drum_kit;
        if let Some(preferred) = genre_config.drum_kit_preference.choose(&mut rng) {
            if rng.gen_range(0..100) < 25 {
                section_kit = *preferred;
            }
        }

        // Render with kit-specific sounds and swing
        let drums = render_drum_pattern_with_kit(&pattern, bpm, intensity, section_kit, genre, &drum_params);
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
    drum_params: &DrumSoundParams,
) -> Vec<f32> {
    let beat_duration = 60.0 / bpm;
    let sixteenth_duration = beat_duration / 4.0;
    let mut rng = rand::thread_rng();

    // Genre-specific base swing amount (applied to off-beats)
    let base_swing = match genre {
        Genre::Lofi => 0.30,         // Heavy swing for laid-back feel
        Genre::Jazz => 0.35,         // Even more swing for jazz
        Genre::HipHop => 0.25,       // Medium swing
        Genre::Funk => 0.20,         // Subtle funk groove
        Genre::ElectroSwing => 0.28, // Swing for electro-swing
        Genre::DnB => 0.10,          // Slight swing for variation
        _ => 0.05,                   // Minimal swing for rock/dubstep
    };

    // Add per-song variation: ±15% swing variance
    let swing_variation = rng.gen_range(-0.15..0.15);
    let swing_amount_f32: f32 = base_swing + swing_variation;
    let swing_amount = swing_amount_f32.max(0.0f32).min(0.5f32);

    let total_duration = pattern.len() as f32 * sixteenth_duration;
    let total_samples = (total_duration * get_sample_rate() as f32) as usize;

    let mut output = vec![0.0; total_samples];

    for (step_idx, hits) in pattern.iter().enumerate() {
        // Apply swing: delay every second 16th note (odd indices)
        let swing_offset = if step_idx % 2 == 1 {
            sixteenth_duration * swing_amount
        } else {
            0.0
        };

        let step_time = step_idx as f32 * sixteenth_duration + swing_offset;
        let start_sample = (step_time * get_sample_rate() as f32) as usize;

        for hit in hits {
            let drum_samples = match (hit, drum_kit) {
                (DrumHit::Kick, DrumKit::Rock) => generate_rock_kick(0.8 * intensity),
                (DrumHit::Kick, DrumKit::Electronic808) | (DrumHit::Kick, DrumKit::HipHop) => {
                    // Use dubstep kick for electronic kits
                    generate_dubstep_kick(0.8 * intensity)
                }
                (DrumHit::Kick, _) => generate_kick_with_params(0.8 * intensity, Some(drum_params)),

                (DrumHit::Snare, DrumKit::Rock) => generate_rock_snare(0.7 * intensity),
                (DrumHit::Snare, DrumKit::Electronic808) | (DrumHit::Snare, DrumKit::HipHop) => {
                    // Use DnB snare for electronic kits
                    generate_dnb_snare(0.7 * intensity)
                }
                (DrumHit::Snare, _) => generate_snare_with_params(0.7 * intensity, Some(drum_params)),

                (DrumHit::HiHatClosed, _) => generate_hihat_with_params(0.4 * intensity, false, Some(drum_params)),
                (DrumHit::HiHatOpen, _) => generate_hihat_with_params(0.5 * intensity, true, Some(drum_params)),
                (DrumHit::Clap, _) => generate_clap(0.6 * intensity),
                (DrumHit::Conga, _) => crate::synthesis::generate_conga(200.0, 0.5 * intensity),
                (DrumHit::Shaker, _) => crate::synthesis::generate_shaker(0.3 * intensity),
                (DrumHit::Crash, _) => crate::synthesis::drums::generate_crash(0.7 * intensity),
                (DrumHit::RimShot, _) => crate::synthesis::drums::generate_rimshot(0.5 * intensity),
                (DrumHit::Tom, _) => crate::synthesis::drums::generate_tom(0.6 * intensity),
                (DrumHit::Ride, _) => crate::synthesis::drums::generate_ride(0.45 * intensity),
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
        let section_chords: Vec<_> = chords.iter().skip(bar_idx).take(*bars).cloned().collect();

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
            let silence_samples = (bar_duration * *bars as f32 * get_sample_rate() as f32) as usize;
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
        let mut pattern =
            generate_funk_bass_pattern_with_style(frequency, bar_duration, bass_note_fn);

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
            let drop = crate::synthesis::bass::generate_sub_bass(
                drop_freq,
                drop_duration,
                bass_drop_cfg.amplitude,
            );

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

/// Generate a funky bass pattern with custom bass note generator - uses varied patterns from bass.rs
pub fn generate_funk_bass_pattern_with_style(
    root_freq: f32,
    bar_duration: f32,
    _bass_note_fn: fn(f32, f32, f32) -> Vec<f32>,
) -> Vec<f32> {
    // Use the varied funk_bass_pattern from bass.rs which has 4 different pattern types
    // (standard, sparse, dense, syncopated) for much more variance
    generate_funk_bass_pattern(root_freq, bar_duration)
}

/// Generate pads with arrangement awareness
pub fn generate_pads_with_arrangement(
    arrangement: &Arrangement,
    chords: &[Chord],
    bpm: f32,
    num_bars: usize,
) -> Vec<f32> {
    let beat_duration = 60.0 / bpm;
    let bar_duration = beat_duration * 4.0;
    let mut pads = Vec::new();
    let mut chord_index = 0;

    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = (0..*bars)
            .map(|offset| chords[(chord_index + offset) % chords.len()].clone())
            .collect();

        if Arrangement::section_has_pads(*section) {
            pads.extend(generate_pads(&section_chords, bpm, *bars));
        } else if matches!(section, Section::Intro | Section::Outro) {
            let root_note = section_chords
                .first()
                .and_then(|chord| {
                    let notes = chord.get_notes();
                    notes.first().copied()
                })
                .unwrap_or(48);
            let freq = midi_to_freq(root_note);
            let duration = bar_duration * *bars as f32;
            pads.extend(generate_drone(freq, duration, 0.15));
        } else {
            let silence_samples = (bar_duration * *bars as f32 * get_sample_rate() as f32) as usize;
            pads.extend(vec![0.0; silence_samples]);
        }

        chord_index += *bars;
    }

    if pads.is_empty() {
        generate_pads(chords, bpm, num_bars)
    } else {
        pads
    }
}

/// Render melody with arrangement awareness and instrument preference for stereo width
pub fn render_arranged_melody_with_instrument(
    arrangement: &Arrangement,
    key: &Key,
    chords: &[Chord],
    bpm: f32,
    melody_cfg: &config::MelodyConfig,
    melody_density: MelodyDensity,
    genre: &Genre,
    instrument_preference: Option<InstrumentType>,
) -> Vec<f32> {
    let mut all_melody = Vec::new();
    let mut bar_idx = 0;

    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords.iter().skip(bar_idx).take(*bars).cloned().collect();

        if Arrangement::section_has_melody(*section) {
            // Use new melody generation with style and instrument awareness
            let melody = generate_melody_with_style_and_instrument(
                key,
                &section_chords,
                bpm,
                *bars,
                melody_cfg,
                melody_density,
                Some(*section),
                Some(*genre),
                instrument_preference,
            );
            all_melody.extend(melody);
        } else {
            // No melody - just add silence
            let bar_duration = 60.0 / bpm * 4.0;
            let silence_samples = (bar_duration * *bars as f32 * get_sample_rate() as f32) as usize;
            all_melody.extend(vec![0.0; silence_samples]);
        }

        bar_idx += bars;
    }

    all_melody
}

/// Render transition FX track (risers, crashes, downlifters)
pub fn render_fx_track(arrangement: &Arrangement, bpm: f32) -> Vec<f32> {
    let bar_duration = 60.0 / bpm * 4.0;
    let total_samples =
        (bar_duration * arrangement.total_bars as f32 * get_sample_rate() as f32) as usize;
    let mut fx_track = vec![0.0; total_samples];

    let mut current_bar = 0;
    let mut rng = rand::thread_rng();

    for (section_idx, (section, bars)) in arrangement.sections.iter().enumerate() {
        let section_start_sample =
            (current_bar as f32 * bar_duration * get_sample_rate() as f32) as usize;

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

        // Add crash at the start of important sections (Chorus, Bridge) - much rarer now
        if matches!(
            *section,
            crate::composition::Section::Chorus | crate::composition::Section::Bridge
        ) {
            // Only 25% chance to add crash (was 100%)
            if rng.gen_range(0..100) < 25 {
                let crash = generate_crash(2.0);

                for (i, &sample) in crash.iter().enumerate() {
                    let idx = section_start_sample + i;
                    if idx < fx_track.len() {
                        fx_track[idx] += sample * 0.25; // Reduced from 0.5 to 0.25
                    }
                }
            }
        }

        // Add impact at very start of Chorus for emphasis - also rarer
        if *section == crate::composition::Section::Chorus {
            // Only 30% chance to add impact
            if rng.gen_range(0..100) < 30 {
                let impact = generate_impact();

                for (i, &sample) in impact.iter().enumerate() {
                    let idx = section_start_sample + i;
                    if idx < fx_track.len() {
                        fx_track[idx] += sample * 0.4; // Reduced from 0.6 to 0.4
                    }
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
    genre: &Genre,
) -> Option<Vec<f32>> {
    if percussion_type == "None" {
        return None;
    }

    let mut percussion = Vec::new();
    let mut rng = rand::thread_rng();
    let bar_duration = tempo.bar_duration();
    let beat_duration = tempo.beat_duration();
    let (beat_hit_chance, syncopated_hit_chance) = match genre {
        Genre::Funk => (55, 45),
        Genre::Jazz => (45, 35),
        Genre::HipHop => (50, 25),
        Genre::ElectroSwing => (65, 55),
        Genre::Dubstep | Genre::DnB => (35, 60),
        _ => (40, 30),
    };

    let mut absolute_bar = 0;

    for (section, bars) in &arrangement.sections {
        if Arrangement::section_has_melody(*section) {
            // Add percussion to sections with melody
            for bar_offset in 0..*bars {
                let bar_samples = (bar_duration * get_sample_rate() as f32) as usize;
                let mut bar = vec![0.0; bar_samples];

                // Add percussion hits on beats
                for beat in 0..4 {
                    let mut chance = beat_hit_chance;
                    if arrangement.is_section_start(absolute_bar + bar_offset) && beat == 0 {
                        chance += 15;
                    }
                    if let Some((_from, to)) =
                        arrangement.get_transition(absolute_bar + bar_offset + 1)
                    {
                        if matches!(to, Section::Chorus) && beat >= 2 {
                            chance += 10;
                        }
                    }

                    if rng.gen_range(0..100) < chance {
                        // 40% chance per beat
                        let hit_time = beat as f32 * beat_duration;
                        let hit_sample = (hit_time * get_sample_rate() as f32) as usize;

                        let hit = match percussion_type {
                            "Tambourine" => generate_tambourine(0.25),
                            "Cowbell" => generate_cowbell(0.3),
                            "Bongo" => generate_bongo(rng.gen_range(0..100) < 50, 0.25),
                            "Woodblock" => generate_woodblock(0.25),
                            _ => generate_triangle_perc(0.2),
                        };

                        for (i, &sample) in hit.iter().enumerate() {
                            let idx = hit_sample + i;
                            if idx < bar.len() {
                                bar[idx] += sample;
                            }
                        }
                    }
                }

                // Add syncopated hits on the "and" of beats for swing-heavy genres
                for subdivision in [0.5, 1.5, 2.5, 3.5] {
                    if rng.gen_range(0..100) < syncopated_hit_chance {
                        let hit_time = subdivision * beat_duration;
                        let hit_sample = (hit_time * get_sample_rate() as f32) as usize;
                        let hit = match genre {
                            Genre::Funk | Genre::Jazz => generate_tambourine(0.22),
                            Genre::Dubstep | Genre::DnB => generate_triangle_perc(0.15),
                            Genre::HipHop => generate_woodblock(0.25),
                            _ => generate_cowbell(0.2),
                        };

                        for (i, &sample) in hit.iter().enumerate() {
                            let idx = hit_sample + i;
                            if idx < bar.len() {
                                bar[idx] += sample * 0.8;
                            }
                        }
                    }
                }

                percussion.extend(bar);
            }
        } else {
            // Silence for sections without melody
            let silence_samples = (bar_duration * *bars as f32 * get_sample_rate() as f32) as usize;
            percussion.extend(vec![0.0; silence_samples]);
        }
        absolute_bar += bars;
    }

    Some(percussion)
}
