use crate::composition::arranger::DrumComplexity;
use crate::composition::beat_maker::{DrumHit, DrumKit, GrooveStyle};
use crate::composition::genre::{GenreConfig, MelodyDensity};
use crate::composition::music_theory::{midi_to_freq, Chord};
/// Audio rendering functions for generating tracks from arrangements
use crate::composition::{Arrangement, Genre, Key, Section, Tempo};
use crate::config;
use crate::synthesis::bass::generate_metal_bassline;
use crate::synthesis::drums::{
    generate_china, generate_crash, generate_hihat, generate_kick, generate_ride, generate_snare,
    generate_tom, DrumSoundParams,
};
use crate::synthesis::melody::{generate_melody_with_style_and_instrument, InstrumentType};
use crate::synthesis::percussion::{
    generate_bongo, generate_cowbell, generate_tambourine, generate_triangle_perc,
    generate_woodblock,
};
use crate::synthesis::{
    generate_downlifter, generate_drone, generate_impact, generate_pads, generate_riser,
    get_sample_rate,
};
use rand::Rng;

pub type DrumPattern = Vec<Vec<DrumHit>>;

/// Render drums with arrangement awareness
pub fn render_arranged_drums(
    arrangement: &Arrangement,
    _base_groove: GrooveStyle,
    bpm: f32,
    _drum_kit: DrumKit,
    genre: &Genre,
    _genre_config: &GenreConfig,
) -> Vec<f32> {
    let mut all_drums = Vec::new();
    let mut bar_idx = 0;
    let _rng = rand::thread_rng();

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

        // Always SwampMetal groove
        let groove = GrooveStyle::SwampMetal;

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

        let section_kit = DrumKit::Metal;

        // Render with kit-specific sounds and swing
        let drums = render_drum_pattern_with_kit(
            &pattern,
            bpm,
            intensity,
            section_kit,
            genre,
            &drum_params,
        );
        all_drums.extend(drums);

        bar_idx += bars;
    }

    all_drums
}

/// Generate drum pattern with complexity and transition awareness
pub fn generate_drum_pattern_with_complexity(
    _groove: GrooveStyle,
    bars: usize,
    complexity: DrumComplexity,
    add_fill: bool,
    _needs_buildup: bool,
    _needs_breakdown: bool,
) -> DrumPattern {
    // Use BeatMaker to generate patterns
    let beat_maker = crate::composition::beat_maker::BeatMaker::new(Genre::SwampMetal);
    let beat_events = beat_maker.generate_beat(120.0, bars); // Tempo doesn't affect pattern structure here

    // Convert BeatMaker events to DrumPattern (Vec<Vec<DrumHit>>)
    let steps_per_bar = 16;
    let total_steps = bars * steps_per_bar;
    let mut pattern = vec![Vec::new(); total_steps];

    for (pos, hit, _vel) in beat_events {
        let step_idx = (pos * 4.0).round() as usize;
        if step_idx < total_steps {
            pattern[step_idx].push(hit);
        }
    }

    // Ensure Rest is present for empty steps
    for step in pattern.iter_mut() {
        if step.is_empty() {
            step.push(DrumHit::Rest);
        }
    }

    let mut rng = rand::thread_rng();

    // Adjust pattern based on complexity
    match complexity {
        DrumComplexity::Simple => {
            // Simplify: remove some hi-hats/cymbals
            for step in pattern.iter_mut() {
                step.retain(|hit| {
                    matches!(
                        hit,
                        DrumHit::Kick | DrumHit::Snare | DrumHit::Rest | DrumHit::Crash
                    )
                });
            }
        }
        DrumComplexity::Medium => {
            // Keep as-is (already medium complexity)
        }
        DrumComplexity::Complex => {
            // Add more variation: occasional extra hits
            for step in pattern.iter_mut() {
                if rng.gen_range(0..100) < 15 {
                    if !step.contains(&DrumHit::China) {
                        step.push(DrumHit::China);
                    }
                }
            }
        }
    }

    // Add fill on last bar if needed (BeatMaker handles this, but we can enhance)
    if add_fill {
        // Already handled by BeatMaker's generate_fill, but let's ensure impact
        let last_step = pattern.len() - 1;
        if !pattern[last_step].contains(&DrumHit::Crash) {
            pattern[last_step].push(DrumHit::Crash);
        }
    }

    pattern
}

/// Render drum pattern with kit-specific sounds and swing
pub fn render_drum_pattern_with_kit(
    pattern: &[Vec<DrumHit>],
    bpm: f32,
    intensity: f32,
    _drum_kit: DrumKit,
    _genre: &Genre,
    _drum_params: &DrumSoundParams,
) -> Vec<f32> {
    let beat_duration = 60.0 / bpm;
    let sixteenth_duration = beat_duration / 4.0;
    
    // No swing for metal - straight grid
    let _swing_amount = 0.0;

    let total_duration = pattern.len() as f32 * sixteenth_duration;
    let total_samples = (total_duration * get_sample_rate() as f32) as usize;

    let mut output = vec![0.0; total_samples];

    for (step_idx, hits) in pattern.iter().enumerate() {
        let step_time = step_idx as f32 * sixteenth_duration;
        let start_sample = (step_time * get_sample_rate() as f32) as usize;

        for hit in hits {
            let drum_samples = match hit {
                DrumHit::Kick => generate_kick(0.8 * intensity),
                DrumHit::Snare => generate_snare(0.7 * intensity),
                DrumHit::HiHatClosed => generate_hihat(0.4 * intensity, false),
                DrumHit::HiHatOpen => generate_hihat(0.5 * intensity, true),
                DrumHit::Crash => generate_crash(0.7 * intensity),
                DrumHit::Tom => generate_tom(0.6 * intensity),
                DrumHit::Ride => generate_ride(0.45 * intensity),
                DrumHit::China => generate_china(0.6 * intensity),
                DrumHit::Rest => continue,

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
    _genre: &Genre,
    _bass_drop_cfg: &config::BassDropConfig,
    _genre_config: &crate::composition::genre::GenreConfig,
) -> Vec<f32> {
    let mut all_bass = Vec::new();
    let mut bar_idx = 0;

    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords.iter().skip(bar_idx).take(*bars).cloned().collect();

        if Arrangement::section_has_heavy_bass(*section) {
            // Always use metal bassline
            let bass = generate_metal_bassline(&section_chords, bpm, *bars);
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

        // Add crash at the start of important sections (Chorus, Bridge)
        if matches!(
            *section,
            crate::composition::Section::Chorus | crate::composition::Section::Bridge
        ) {
            // Always crash on chorus/bridge start for metal
            let crash = generate_crash(2.0);

            for (i, &sample) in crash.iter().enumerate() {
                let idx = section_start_sample + i;
                if idx < fx_track.len() {
                    fx_track[idx] += sample * 0.4; 
                }
            }
        }

        // Add impact at very start of Chorus for emphasis
        if *section == crate::composition::Section::Chorus {
            if rng.gen_range(0..100) < 50 {
                let impact = generate_impact();

                for (i, &sample) in impact.iter().enumerate() {
                    let idx = section_start_sample + i;
                    if idx < fx_track.len() {
                        fx_track[idx] += sample * 0.5;
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
    _genre: &Genre,
) -> Option<Vec<f32>> {
    if percussion_type == "None" {
        return None;
    }

    let mut percussion = Vec::new();
    let mut rng = rand::thread_rng();
    let bar_duration = tempo.bar_duration();
    let beat_duration = tempo.beat_duration();
    
    // Metal percussion is sparse
    let beat_hit_chance = 20;

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

                    if rng.gen_range(0..100) < chance {
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
