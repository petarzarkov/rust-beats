use super::instruments::{
    generate_acoustic_guitar, generate_electric_guitar, generate_mallet, generate_rhodes_note,
    generate_soft_pluck, generate_ukulele, generate_warm_organ,
};
use super::synthesizer::*;
use crate::composition::genre::MelodyDensity;
use crate::composition::music_theory::{midi_to_freq, Chord, Key, MidiNote};
use crate::composition::{Genre, Section};
use rand::Rng;

/// Melody style determines the approach to melody generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MelodyStyle {
    Sparse,       // Simple ear candy (current default)
    Syncopated,   // Offbeat rhythms
    Connected,    // Step-wise melodic phrases
    Arpeggio,     // Arpeggiated patterns
    CallResponse, // Conversational riffs
}

struct DensitySettings {
    occurrence_multiplier: f32,
    min_notes: u8,
    max_notes: u8,
    syncopated_bias: u8,
}

fn density_settings(density: MelodyDensity) -> DensitySettings {
    match density {
        MelodyDensity::Sparse => DensitySettings {
            occurrence_multiplier: 0.6,
            min_notes: 1,
            max_notes: 2,
            syncopated_bias: 15,
        },
        MelodyDensity::Moderate => DensitySettings {
            occurrence_multiplier: 1.0,
            min_notes: 2,
            max_notes: 3,
            syncopated_bias: 35,
        },
        MelodyDensity::Dense => DensitySettings {
            occurrence_multiplier: 1.4,
            min_notes: 3,
            max_notes: 5,
            syncopated_bias: 55,
        },
        MelodyDensity::Glitchy => DensitySettings {
            occurrence_multiplier: 1.6,
            min_notes: 4,
            max_notes: 6,
            syncopated_bias: 70,
        },
    }
}

/// Generate melody with style, section awareness, and optional instrument preference
pub fn generate_melody_with_style_and_instrument(
    key: &Key,
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    melody_cfg: &crate::config::MelodyConfig,
    melody_density: MelodyDensity,
    section: Option<Section>,
    genre: Option<Genre>,
    instrument_preference: Option<InstrumentType>,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;

    let scale_notes = key.get_scale_notes_range(2); // Only 2 octaves (more focused)
    let mut melody = Vec::new();
    let mut rng = rand::thread_rng();
    let density_cfg = density_settings(melody_density);

    // Determine melody style based on section
    let style = determine_melody_style(section, genre, melody_density, &mut rng);

    // Determine instrument based on section/genre and preference
    let instrument = determine_instrument(section, genre, instrument_preference, &mut rng);

    let mut last_note = scale_notes[scale_notes.len() / 2]; // Start in middle of scale

    // Generate melody with selected style
    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];

        // Use config for melody occurrence chance
        let occurrence =
            (melody_cfg.occurrence_chance * density_cfg.occurrence_multiplier).clamp(0.05, 0.98);
        let chance_percent = (occurrence * 100.0) as u32;
        if rng.gen_range(0..100) < chance_percent {
            let pattern = match style {
                MelodyStyle::Sparse => generate_ear_candy_bar_with_instrument(
                    &scale_notes,
                    chord,
                    bar_duration,
                    &mut rng,
                    melody_cfg,
                    instrument,
                ),
                MelodyStyle::Syncopated => {
                    // Avoid syncopated patterns in Lofi/Jazz genres (causes jungle feel)
                    let avoid_syncopation = matches!(genre, Some(Genre::Lofi) | Some(Genre::Jazz));
                    if avoid_syncopation {
                        // Use ghost notes instead for these genres
                        generate_ear_candy_bar_with_instrument(
                            &scale_notes,
                            chord,
                            bar_duration,
                            &mut rng,
                            melody_cfg,
                            instrument,
                        )
                    } else {
                        // Use syncopated pattern but soften it (50% volume)
                        let mut bar =
                            vec![0.0; (bar_duration * get_sample_rate() as f32) as usize];
                        let chord_tones = chord.get_notes();
                        generate_syncopated_rhythm_candy_with_instrument(
                            &mut bar,
                            &chord_tones,
                            bar_duration,
                            &mut rng,
                            instrument,
                        );
                        // Soften the syncopated pattern
                        for sample in bar.iter_mut() {
                            *sample *= 0.6;
                        }
                        bar
                    }
                }
                MelodyStyle::Connected => {
                    let (phrase, new_last_note) = generate_connected_melody_phrase(
                        &scale_notes,
                        chord,
                        bar_duration,
                        last_note,
                        &mut rng,
                        melody_cfg,
                        melody_density,
                    );
                    last_note = new_last_note;
                    phrase
                }
                MelodyStyle::Arpeggio => generate_arpeggio(chord, tempo, bar_duration),
                MelodyStyle::CallResponse => {
                    generate_call_response_phrase(chord, bar_duration, tempo, &mut rng)
                }
            };
            melody.extend(pattern);

            // Add strategic ear candy in important sections (10-15% of bars)
            let add_ear_candy = match section {
                Some(Section::Chorus) | Some(Section::Bridge) => rng.gen_range(0..100) < 15,
                Some(Section::Verse) => rng.gen_range(0..100) < 10,
                _ => rng.gen_range(0..100) < 5, // Rare in intro/outro
            };

            if add_ear_candy && style != MelodyStyle::Sparse {
                // Choose appropriate ear candy type
                let candy_choice = rng.gen_range(0..100);
                let mut candy_bar = vec![0.0; (bar_duration * get_sample_rate() as f32) as usize];
                let chord_tones = chord.get_notes();

                if candy_choice < 40 {
                    // On-beat hits - good, use often
                    generate_on_beat_hits_candy_with_instrument(
                        &mut candy_bar,
                        &chord_tones,
                        bar_duration,
                        &mut rng,
                        instrument,
                    );
                } else if candy_choice < 70 {
                    // Chord movement - good, melodic
                    generate_chord_movement_candy_with_instrument(
                        &mut candy_bar,
                        &chord_tones,
                        bar_duration,
                        &mut rng,
                        instrument,
                    );
                } else {
                    // Ghost note fills - subtle, good
                    generate_ghost_note_fills_candy_with_instrument(
                        &mut candy_bar,
                        &chord_tones,
                        bar_duration,
                        &mut rng,
                        instrument,
                    );
                }

                // Mix ear candy into melody (soften to 40% to avoid overwhelming)
                let candy_start = melody.len().saturating_sub(candy_bar.len());
                for (i, &candy_sample) in candy_bar.iter().enumerate() {
                    let idx = candy_start + i;
                    if idx < melody.len() {
                        melody[idx] += candy_sample * 0.4;
                    }
                }
            }
        } else {
            // Rest bar
            let silence_samples = (bar_duration * get_sample_rate() as f32) as usize;
            melody.extend(vec![0.0; silence_samples]);
        }
    }

    melody
}

/// Determine melody style based on section and genre with MUCH more variation
fn determine_melody_style(
    section: Option<Section>,
    genre: Option<Genre>,
    melody_density: MelodyDensity,
    rng: &mut impl Rng,
) -> MelodyStyle {
    // Add random factor to ensure consecutive sections rarely use same style
    let random_seed = rng.gen_range(0..100);

    // Density influences but doesn't dominate (reduced from 60% to 25%)
    match melody_density {
        MelodyDensity::Sparse => {
            if random_seed < 25 {
                return MelodyStyle::Sparse;
            }
        }
        MelodyDensity::Dense => {
            if random_seed < 20 {
                return MelodyStyle::Connected;
            }
        }
        MelodyDensity::Glitchy => {
            if random_seed < 25 {
                return MelodyStyle::CallResponse;
            }
        }
        MelodyDensity::Moderate => {}
    }

    // Genre-specific biases (reduced probabilities for more mixing)
    match genre {
        Some(Genre::Jazz) | Some(Genre::Funk) => {
            if random_seed < 20 {
                return MelodyStyle::Syncopated;
            }
        }
        Some(Genre::Rock) => {
            if random_seed < 15 {
                return MelodyStyle::Arpeggio;
            }
        }
        Some(Genre::Dubstep) | Some(Genre::DnB) => {
            if random_seed < 18 {
                return MelodyStyle::CallResponse;
            }
        }
        _ => {}
    }

    // Section-based selection with MUCH more variety and balance
    match section {
        Some(Section::Intro) => {
            // Intro: mostly sparse but with some variety
            match rng.gen_range(0..100) {
                0..=55 => MelodyStyle::Sparse,
                56..=75 => MelodyStyle::Arpeggio,
                76..=90 => MelodyStyle::Connected,
                _ => MelodyStyle::CallResponse,
            }
        }
        Some(Section::Verse) => {
            // Verse: balanced variety, avoid same style in consecutive bars
            match rng.gen_range(0..100) {
                0..=22 => MelodyStyle::Sparse,
                23..=45 => MelodyStyle::Connected,
                46..=65 => MelodyStyle::Arpeggio,
                66..=82 => MelodyStyle::CallResponse,
                _ => MelodyStyle::Syncopated,
            }
        }
        Some(Section::Chorus) => {
            // Chorus: ALWAYS contrasting - prefer energetic styles
            match rng.gen_range(0..100) {
                0..=28 => MelodyStyle::Connected,
                29..=52 => MelodyStyle::Arpeggio,
                53..=72 => MelodyStyle::CallResponse,
                73..=88 => MelodyStyle::Syncopated,
                _ => MelodyStyle::Sparse,
            }
        }
        Some(Section::Bridge) => {
            // Bridge: ALWAYS distinctive - prefer complex patterns
            match rng.gen_range(0..100) {
                0..=32 => MelodyStyle::Arpeggio,
                33..=58 => MelodyStyle::CallResponse,
                59..=78 => MelodyStyle::Connected,
                79..=92 => MelodyStyle::Syncopated,
                _ => MelodyStyle::Sparse,
            }
        }
        Some(Section::Outro) => {
            // Outro: mostly calm but not always sparse
            match rng.gen_range(0..100) {
                0..=48 => MelodyStyle::Sparse,
                49..=72 => MelodyStyle::Arpeggio,
                73..=88 => MelodyStyle::Connected,
                _ => MelodyStyle::CallResponse,
            }
        }
        None => {
            // Default: full variety
            match rng.gen_range(0..5) {
                0 => MelodyStyle::Sparse,
                1 => MelodyStyle::Connected,
                2 => MelodyStyle::Arpeggio,
                3 => MelodyStyle::CallResponse,
                _ => MelodyStyle::Syncopated,
            }
        }
    }
}

/// Instrument generator type
#[derive(Clone, Copy)]
pub enum InstrumentType {
    Rhodes,
    WarmOrgan,
    Mallet,
    SoftPluck,
    AcousticGuitar,
    Ukulele,
    ElectricGuitar,
}

/// Determine instrument generator function based on section/genre with optional preference
fn determine_instrument(
    section: Option<Section>,
    genre: Option<Genre>,
    preference: Option<InstrumentType>,
    rng: &mut impl Rng,
) -> InstrumentType {
    // If preference is provided, use it with 70% probability for stereo width
    if let Some(pref) = preference {
        if rng.gen_range(0..100) < 70 {
            return pref;
        }
    }

    // For pads/atmospheric sections, use warm organ
    if section == Some(Section::Intro) || section == Some(Section::Bridge) {
        if rng.gen_range(0..100) < 40 {
            return InstrumentType::WarmOrgan;
        }
    }

    // Genre-based selection
    match genre {
        Some(Genre::Jazz) | Some(Genre::Funk) => match rng.gen_range(0..100) {
            0..=50 => InstrumentType::Rhodes,
            51..=70 => InstrumentType::Mallet,
            71..=85 => InstrumentType::SoftPluck,
            _ => InstrumentType::AcousticGuitar,
        },
        Some(Genre::Rock) => match rng.gen_range(0..100) {
            0..=40 => InstrumentType::ElectricGuitar,
            41..=70 => InstrumentType::AcousticGuitar,
            _ => InstrumentType::Rhodes,
        },
        _ => {
            // Default: variety of instruments
            match rng.gen_range(0..100) {
                0..=50 => InstrumentType::Rhodes,
                51..=70 => InstrumentType::Ukulele,
                71..=85 => InstrumentType::AcousticGuitar,
                _ => InstrumentType::SoftPluck,
            }
        }
    }
}

/// Generate note with selected instrument
fn generate_note_with_instrument(
    instrument: InstrumentType,
    frequency: f32,
    duration: f32,
    velocity: f32,
) -> Vec<f32> {
    match instrument {
        InstrumentType::Rhodes => generate_rhodes_note(frequency, duration, velocity),
        InstrumentType::WarmOrgan => generate_warm_organ(frequency, duration, velocity),
        InstrumentType::Mallet => generate_mallet(frequency, duration, velocity),
        InstrumentType::SoftPluck => generate_soft_pluck(frequency, duration, velocity),
        InstrumentType::AcousticGuitar => generate_acoustic_guitar(frequency, duration, velocity),
        InstrumentType::Ukulele => generate_ukulele(frequency, duration, velocity),
        InstrumentType::ElectricGuitar => {
            generate_electric_guitar(frequency, duration, velocity, 0.3)
        } // Default distortion
    }
}

/// Generate ear candy bar with instrument selection
fn generate_ear_candy_bar_with_instrument(
    _scale_notes: &[MidiNote],
    chord: &Chord,
    bar_duration: f32,
    rng: &mut impl Rng,
    _melody_cfg: &crate::config::MelodyConfig,
    instrument: InstrumentType,
) -> Vec<f32> {
    let mut bar = vec![0.0; (bar_duration * get_sample_rate() as f32) as usize];

    let chord_tones = chord.get_notes();

    // Choose ear candy type
    let candy_type = rng.gen_range(0..100);

    if candy_type < 50 {
        generate_on_beat_hits_candy_with_instrument(
            &mut bar,
            &chord_tones,
            bar_duration,
            rng,
            instrument,
        );
    } else if candy_type < 75 {
        generate_chord_movement_candy_with_instrument(
            &mut bar,
            &chord_tones,
            bar_duration,
            rng,
            instrument,
        );
    } else {
        generate_ghost_note_fills_candy_with_instrument(
            &mut bar,
            &chord_tones,
            bar_duration,
            rng,
            instrument,
        );
    }

    bar
}

/// On-beat hits with instrument selection
fn generate_on_beat_hits_candy_with_instrument(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
    instrument: InstrumentType,
) {
    let beat_duration = bar_duration / 4.0;
    let beats_to_hit = vec![0, 1, 2, 3];
    let num_hits = rng.gen_range(1..=2);

    for i in 0..num_hits {
        let beat = beats_to_hit[i * 2];
        let time = beat as f32 * beat_duration;
        let note = chord_tones[rng.gen_range(0..chord_tones.len())] + 12;
        let freq = midi_to_freq(note);

        let hit = generate_note_with_instrument(instrument, freq, 0.4, 0.6);

        let start_sample = (time * get_sample_rate() as f32) as usize;
        for (j, &sample) in hit.iter().enumerate() {
            let idx = start_sample + j;
            if idx < bar.len() {
                bar[idx] += sample * 0.35;
            }
        }
    }
}

/// Chord movement with instrument selection
fn generate_chord_movement_candy_with_instrument(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
    instrument: InstrumentType,
) {
    let beat_duration = bar_duration / 4.0;
    let num_chords = rng.gen_range(1..=2);

    for i in 0..num_chords {
        let time = (i as f32 * 2.0) * beat_duration;
        let note_idx = i % chord_tones.len();
        let note = chord_tones[note_idx] + 12;
        let freq = midi_to_freq(note);

        let hit = generate_note_with_instrument(instrument, freq, 0.5, 0.55);

        let start_sample = (time * get_sample_rate() as f32) as usize;
        for (j, &sample) in hit.iter().enumerate() {
            let idx = start_sample + j;
            if idx < bar.len() {
                bar[idx] += sample * 0.4;
            }
        }
    }
}

/// Ghost note fills with instrument selection
fn generate_ghost_note_fills_candy_with_instrument(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
    instrument: InstrumentType,
) {
    let beat_duration = bar_duration / 4.0;
    let fill_times = vec![3.25, 3.5, 3.75];

    for &beat_offset in &fill_times {
        let time = beat_offset * beat_duration;
        let note = chord_tones[rng.gen_range(0..chord_tones.len())] + 12;
        let freq = midi_to_freq(note);

        let ghost = generate_note_with_instrument(instrument, freq, 0.08, 0.4);

        let start_sample = (time * get_sample_rate() as f32) as usize;
        for (i, &sample) in ghost.iter().enumerate() {
            let idx = start_sample + i;
            if idx < bar.len() {
                bar[idx] += sample * 0.3;
            }
        }
    }
}

/// Syncopated rhythm with instrument selection
fn generate_syncopated_rhythm_candy_with_instrument(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
    instrument: InstrumentType,
) {
    let beat_duration = bar_duration / 4.0;
    // Syncopated pattern: offbeats and "and" of beats
    let hit_times = vec![0.5, 1.5, 2.25, 3.0];

    for &beat_offset in &hit_times {
        if rng.gen_range(0..100) < 70 {
            // 70% chance for each hit
            let time = beat_offset * beat_duration;
            let note = chord_tones[rng.gen_range(0..chord_tones.len())] + 12;
            let freq = midi_to_freq(note);

            // Quick staccato
            let hit = generate_note_with_instrument(instrument, freq, 0.12, 0.6);

            let start_sample = (time * get_sample_rate() as f32) as usize;
            for (i, &sample) in hit.iter().enumerate() {
                let idx = start_sample + i;
                if idx < bar.len() {
                    bar[idx] += sample * 0.4;
                }
            }
        }
    }
}

fn generate_call_response_phrase(
    chord: &Chord,
    bar_duration: f32,
    tempo: f32,
    rng: &mut impl Rng,
) -> Vec<f32> {
    let total_samples = (bar_duration * get_sample_rate() as f32) as usize;
    let mut call_layer = vec![0.0; total_samples];
    let mut response_layer = vec![0.0; total_samples];

    let chord_notes = chord.get_notes();
    if chord_notes.is_empty() {
        return call_layer;
    }

    let root = chord_notes[0];
    let accent = chord_notes[rng.gen_range(0..chord_notes.len())];
    let call = generate_note(
        Waveform::Triangle,
        midi_to_freq(root + 12),
        bar_duration * 0.4,
        &Envelope::lead(),
        0.25,
    );
    for (i, &sample) in call.iter().enumerate() {
        if i < call_layer.len() {
            call_layer[i] += sample;
        }
    }

    let response = generate_note(
        Waveform::Saw,
        midi_to_freq(accent + 19),
        bar_duration * 0.35,
        &Envelope::lead(),
        0.2,
    );
    let response_start = ((60.0 / tempo) * get_sample_rate() as f32) as usize;
    for (i, &sample) in response.iter().enumerate() {
        let idx = response_start + i;
        if idx < response_layer.len() {
            response_layer[idx] += sample;
        }
    }

    let layers = vec![call_layer, response_layer];
    let mut phrase = mix_buffers(&layers);
    let mut filter = LowPassFilter::new(4500.0, 0.35);
    filter.process_buffer(&mut phrase);

    phrase
}

/// Generate connected melodic phrases with step-wise motion
pub fn generate_connected_melody_phrase(
    scale_notes: &[MidiNote],
    chord: &Chord,
    bar_duration: f32,
    last_note: MidiNote,
    rng: &mut impl Rng,
    melody_cfg: &crate::config::MelodyConfig,
    melody_density: MelodyDensity,
) -> (Vec<f32>, MidiNote) {
    let mut phrase = vec![0.0; (bar_duration * get_sample_rate() as f32) as usize];

    // Get chord tones for this chord (emphasize these)
    let chord_tones = chord.get_notes();

    let density_cfg = density_settings(melody_density);
    let num_notes = rng
        .gen_range(density_cfg.min_notes..=density_cfg.max_notes)
        .max(1) as usize;

    let mut current_note = last_note;

    // Choose rhythmic pattern: straight, swung, triplet-based, or mixed
    let mut rhythmic_pattern = match rng.gen_range(0..100) {
        0..=50 => "straight", // Straight timing
        51..=75 => "swung",   // Swung 8ths
        76..=90 => "triplet", // Triplet-based
        _ => "mixed",         // Mixed timing
    };
    if rng.gen_range(0..100) < density_cfg.syncopated_bias {
        rhythmic_pattern = "swung";
    }

    for note_idx in 0..num_notes {
        // Position in the bar with more rhythmic variety
        let base_position = (note_idx as f32) * (4.0 / num_notes as f32);
        let mut beat_position = match rhythmic_pattern {
            "straight" => {
                if rng.gen_range(0..100) < 70 {
                    base_position // On beat
                } else {
                    base_position + 0.5 // Syncopated
                }
            }
            "swung" => {
                // Swung feel: delay off-beats
                if note_idx % 2 == 0 {
                    base_position
                } else {
                    base_position + 0.3 // Swung off-beat
                }
            }
            "triplet" => {
                // Triplet subdivision
                let triplet_pos = (note_idx % 3) as f32;
                base_position + (triplet_pos * 0.33)
            }
            _ => {
                // Mixed: random syncopation
                base_position + rng.gen_range(0.0..0.75)
            }
        };
        if rng.gen_range(0..100) < density_cfg.syncopated_bias {
            beat_position += 0.25;
        }

        let start_time = (beat_position / 4.0) * bar_duration;

        // Vary note durations more
        let duration_multiplier = match rng.gen_range(0..100) {
            0..=40 => 1.0,  // Normal
            41..=70 => 0.7, // Staccato
            71..=85 => 1.5, // Legato
            _ => 2.0,       // Very long (rare)
        };
        let duration = (bar_duration / (num_notes as f32 * 1.2)) * duration_multiplier;

        // Choose next note based on MELODIC MOTION with more variance
        let midi_note = choose_next_melodic_note(current_note, scale_notes, &chord_tones, rng);

        current_note = midi_note;
        let frequency = midi_to_freq(midi_note);

        // Add occasional larger intervals for more interest (10% chance)
        let final_frequency = if rng.gen_range(0..100) < 10 {
            // Occasional octave jump or larger interval
            let jump = rng.gen_range(-12..13);
            midi_to_freq((midi_note as i16 + jump).max(48).min(96) as u8)
        } else {
            frequency
        };

        // Humanization: Random timing offset with more variance (±20ms)
        let timing_offset = rng.gen_range(-0.020..0.020);
        let humanized_start = (start_time + timing_offset).max(0.0);

        // Humanization: More dynamic velocity variation
        let beat_strength = if (beat_position % 1.0) < 0.1 {
            1.0
        } else {
            0.85
        };
        let accent = if rng.gen_range(0..100) < 15 { 1.2 } else { 1.0 }; // Occasional accents
        let base_velocity = 0.75 * beat_strength * accent;
        let velocity: f32 = base_velocity + rng.gen_range(-0.15..0.20);
        let velocity = velocity.clamp(0.5f32, 1.0f32); // Wider range for more dynamics

        // Humanization: Slight duration variation
        let duration_variation = rng.gen_range(0.9..1.1);
        let humanized_duration = duration * duration_variation;

        let note = generate_melody_note(final_frequency, humanized_duration, velocity, melody_cfg);

        // Add to phrase with humanized timing
        let start_sample = (humanized_start * get_sample_rate() as f32) as usize;
        for (i, &sample) in note.iter().enumerate() {
            let idx = start_sample + i;
            if idx < phrase.len() {
                phrase[idx] += sample;
            }
        }
    }

    (phrase, current_note)
}

/// Choose the next note based on melodic motion (step-wise preferred)
pub fn choose_next_melodic_note(
    current_note: MidiNote,
    scale_notes: &[MidiNote],
    chord_tones: &[MidiNote],
    rng: &mut impl Rng,
) -> MidiNote {
    // Prefer chord tones (80% of the time)
    let target_notes = if rng.gen_range(0..100) < 80 && !chord_tones.is_empty() {
        chord_tones
    } else {
        scale_notes
    };

    // Find notes within different interval ranges from current note
    let mut steps = Vec::new(); // 1-2 semitones away
    let mut small_jumps = Vec::new(); // 3-5 semitones away
    let mut medium_jumps = Vec::new(); // 6-8 semitones away

    for &note in target_notes {
        // Calculate interval (convert to i16 for abs())
        let interval = (note as i16 - current_note as i16).abs() as u8;

        if interval == 0 {
            continue; // Skip repeating the same note
        } else if interval <= 2 {
            steps.push(note);
        } else if interval <= 5 {
            small_jumps.push(note);
        } else if interval <= 8 {
            medium_jumps.push(note);
        }
    }

    // Weighted choice: prefer step-wise motion
    let choice = rng.gen_range(0..100);

    let next_note = if choice < 60 && !steps.is_empty() {
        // 60% - step-wise motion (smooth, connected)
        steps[rng.gen_range(0..steps.len())]
    } else if choice < 90 && !small_jumps.is_empty() {
        // 30% - small jumps (still melodic)
        small_jumps[rng.gen_range(0..small_jumps.len())]
    } else if !medium_jumps.is_empty() {
        // 10% - medium jumps (occasional color)
        medium_jumps[rng.gen_range(0..medium_jumps.len())]
    } else if !small_jumps.is_empty() {
        // Fallback to small jumps
        small_jumps[rng.gen_range(0..small_jumps.len())]
    } else if !steps.is_empty() {
        // Fallback to steps
        steps[rng.gen_range(0..steps.len())]
    } else {
        // Last resort: stay on current note or pick closest
        target_notes[rng.gen_range(0..target_notes.len())]
    };

    next_note
}

/// Generate melody (simplified wrapper for generate_melody_with_style_and_instrument)
pub fn generate_melody(
    key: &Key,
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    melody_cfg: &crate::config::MelodyConfig,
    melody_density: MelodyDensity,
) -> Vec<f32> {
    generate_melody_with_style_and_instrument(
        key,
        chords,
        tempo,
        bars,
        melody_cfg,
        melody_density,
        None,
        None,
        None,
    )
}

/// Generate a single melody note with humanization and warmth
pub fn generate_melody_note(
    frequency: f32,
    duration: f32,
    velocity: f32,
    melody_cfg: &crate::config::MelodyConfig,
) -> Vec<f32> {
    let mut rng = rand::thread_rng();

    // Use config for Rhodes usage percentage
    let use_rhodes = rng.gen_range(0..100) < melody_cfg.rhodes_usage_percent;

    if use_rhodes {
        // Use Rhodes for warm, smooth lofi sound
        return generate_rhodes_note(frequency, duration, velocity);
    }

    // Fallback to warm synth with humanization
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Soft envelope
    let envelope = Envelope {
        attack: 0.015,
        decay: 0.25,
        sustain: 0.65,
        release: 0.35,
    };

    let note_off_time = duration * 0.75;

    // Mostly sine waves for smoothness
    let mut sine1 = Oscillator::new(Waveform::Sine, frequency);
    let mut sine2 = Oscillator::new(Waveform::Sine, frequency * 2.01); // Slight detune
    let mut triangle = Oscillator::new(Waveform::Triangle, frequency);

    // Gentle filter
    let mut filter = LowPassFilter::new(1800.0, 0.35);

    // Random vibrato depth for human imperfection
    let vibrato_depth = rng.gen_range(0.002..0.008);
    let mut vibrato_lfo = LFO::new(4.5, vibrato_depth);

    // Pitch drift at start (portamento effect)
    let pitch_drift_amount = rng.gen_range(0.0..0.02);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        // Pitch drift (slides into pitch at start)
        let drift_factor = if time < 0.05 {
            1.0 - pitch_drift_amount * (1.0 - time / 0.05)
        } else {
            1.0
        };

        // Apply vibrato (starts after attack)
        let vibrato = if time > 0.1 {
            vibrato_lfo.next_value()
        } else {
            0.0
        };

        let freq_mod = frequency * drift_factor * (1.0 + vibrato);

        sine1.frequency = freq_mod;
        sine2.frequency = freq_mod * 2.01;
        triangle.frequency = freq_mod;

        let mut sample =
            sine1.next_sample() * 0.6 + sine2.next_sample() * 0.2 + triangle.next_sample() * 0.2;

        // Gentle filter movement
        filter.cutoff = 1500.0 + env_amp * 1000.0;
        sample = filter.process(sample);

        samples[i] = sample * env_amp * velocity * 0.7;
    }

    samples
}

/// Generate an arpeggio from a chord
pub fn generate_arpeggio(chord: &Chord, tempo: f32, duration: f32) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let sixteenth_duration = beat_duration / 4.0;
    let mut rng = rand::thread_rng();

    let chord_notes = chord.get_notes();
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut arpeggio = vec![0.0; num_samples];

    // Vary the arpeggio pattern - not just 0,1,2,0,1,2...
    let pattern_type = rng.gen_range(0..100);
    let pattern: Vec<usize> = if pattern_type < 25 {
        vec![0, 1, 2, 1] // Up and down
    } else if pattern_type < 50 {
        vec![0, 2, 1, 2] // Skip pattern
    } else if pattern_type < 75 {
        vec![2, 1, 0, 1] // Descending
    } else {
        vec![0, 1, 2, 0, 2, 1] // Complex
    };

    let mut pattern_idx = 0;
    let mut time = 0.0;

    while time < duration {
        let chord_idx = pattern[pattern_idx % pattern.len()] % chord_notes.len();
        let mut midi_note = chord_notes[chord_idx] + 24; // Two octaves up
        
        // Occasionally vary octave (20% chance)
        if rng.gen_range(0..100) < 20 {
            midi_note = (midi_note as i16 + if rng.gen_bool(0.5) { 12 } else { -12 })
                .clamp(36, 96) as u8;
        }
        
        let frequency = midi_to_freq(midi_note);

        // Vary duration and velocity
        let duration_mult = rng.gen_range(0.7..0.9);
        let velocity = rng.gen_range(0.15..0.3);
        let note = generate_arp_note(frequency, sixteenth_duration * duration_mult, velocity);

        let start_sample = (time * get_sample_rate() as f32) as usize;
        for (i, &sample) in note.iter().enumerate() {
            let idx = start_sample + i;
            if idx < arpeggio.len() {
                arpeggio[idx] += sample;
            }
        }

        time += sixteenth_duration;
        pattern_idx += 1;
    }

    arpeggio
}

/// Generate a single arpeggio note
pub fn generate_arp_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let envelope = Envelope {
        attack: 0.001,
        decay: 0.1,
        sustain: 0.3,
        release: 0.05,
    };

    let note_off_time = duration * 0.9;
    let mut square_osc = Oscillator::new(Waveform::Square, frequency);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let sample = square_osc.next_sample() * env_amp * velocity;
        samples.push(sample);
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::generate_melody;
    use crate::composition::genre::MelodyDensity;
    use crate::composition::music_theory::{generate_chord_progression, Key};
    use crate::config::MelodyConfig;

    #[test]
    fn test_melody_generation() {
        let key = Key::random_funky();
        let chords = generate_chord_progression(&key, 4);
        let melody_cfg = MelodyConfig {
            occurrence_chance: 0.15,
            rhodes_usage_percent: 60,
        };
        let melody = generate_melody(
            &key,
            &chords,
            110.0,
            2,
            &melody_cfg,
            MelodyDensity::Moderate,
        );
        assert!(!melody.is_empty());
    }
}
