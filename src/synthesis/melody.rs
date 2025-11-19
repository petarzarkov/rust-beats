use super::synthesizer::*;
use super::instruments::{generate_rhodes_note, generate_warm_organ, generate_mallet, generate_soft_pluck, generate_acoustic_guitar, generate_ukulele, generate_electric_guitar};
use crate::composition::music_theory::{Key, Chord, midi_to_freq, MidiNote};
use crate::composition::{Section, Genre};
use rand::Rng;

/// Melody style determines the approach to melody generation
#[derive(Debug, Clone, Copy)]
pub enum MelodyStyle {
    Sparse,        // Simple ear candy (current default)
    Syncopated,    // Offbeat rhythms
    Connected,     // Step-wise melodic phrases
    Arpeggio,      // Arpeggiated patterns
}

/// Generate a melody following the key and chord progression
pub fn generate_melody(
    key: &Key,
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    melody_cfg: &crate::config::MelodyConfig,
) -> Vec<f32> {
    generate_melody_with_style(key, chords, tempo, bars, melody_cfg, None, None)
}

/// Generate melody with style and section awareness
pub fn generate_melody_with_style(
    key: &Key,
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    melody_cfg: &crate::config::MelodyConfig,
    section: Option<Section>,
    genre: Option<Genre>,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;
    
    let scale_notes = key.get_scale_notes_range(2); // Only 2 octaves (more focused)
    let mut melody = Vec::new();
    let mut rng = rand::thread_rng();
    
    // Determine melody style based on section
    let style = determine_melody_style(section, genre, &mut rng);
    
    // Determine instrument based on section/genre
    let instrument = determine_instrument(section, genre, &mut rng);
    
    let mut last_note = scale_notes[scale_notes.len() / 2]; // Start in middle of scale
    
    // Generate melody with selected style
    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        
        // Use config for melody occurrence chance
        let chance_percent = (melody_cfg.occurrence_chance * 100.0) as u32;
        if rng.gen_range(0..100) < chance_percent {
            let pattern = match style {
                MelodyStyle::Sparse => {
                    generate_ear_candy_bar_with_instrument(
                        &scale_notes,
                        chord,
                        bar_duration,
                        &mut rng,
                        melody_cfg,
                        instrument,
                    )
                }
                MelodyStyle::Syncopated => {
                    let mut bar = vec![0.0; (bar_duration * SAMPLE_RATE() as f32) as usize];
                    let chord_tones = chord.get_notes();
                    generate_syncopated_rhythm_candy_with_instrument(&mut bar, &chord_tones, bar_duration, &mut rng, instrument);
                    bar
                }
                MelodyStyle::Connected => {
                    let (phrase, new_last_note) = generate_connected_melody_phrase(
                        &scale_notes,
                        chord,
                        bar_duration,
                        last_note,
                        &mut rng,
                        melody_cfg,
                    );
                    last_note = new_last_note;
                    phrase
                }
                MelodyStyle::Arpeggio => {
                    generate_arpeggio(chord, tempo, bar_duration)
                }
            };
            melody.extend(pattern);
        } else {
            // Rest bar
            let silence_samples = (bar_duration * SAMPLE_RATE() as f32) as usize;
            melody.extend(vec![0.0; silence_samples]);
        }
    }
    
    melody
}

/// Determine melody style based on section and genre
fn determine_melody_style(section: Option<Section>, genre: Option<Genre>, rng: &mut impl Rng) -> MelodyStyle {
    match section {
        Some(Section::Intro) => MelodyStyle::Sparse,  // Keep intro sparse
        Some(Section::Verse) => {
            // Verse: mix of sparse and connected
            if rng.gen_range(0..100) < 60 {
                MelodyStyle::Sparse
            } else {
                MelodyStyle::Connected
            }
        }
        Some(Section::Chorus) => {
            // Chorus: more variety
            match rng.gen_range(0..100) {
                0..=40 => MelodyStyle::Sparse,
                41..=70 => MelodyStyle::Connected,
                71..=85 => MelodyStyle::Arpeggio,
                _ => MelodyStyle::Syncopated,
            }
        }
        Some(Section::Bridge) => {
            // Bridge: arpeggios and connected phrases
            if rng.gen_range(0..100) < 50 {
                MelodyStyle::Arpeggio
            } else {
                MelodyStyle::Connected
            }
        }
        Some(Section::Outro) => MelodyStyle::Sparse,  // Keep outro sparse
        None => MelodyStyle::Sparse,  // Default
    }
}

/// Instrument generator type
#[derive(Clone, Copy)]
enum InstrumentType {
    Rhodes,
    WarmOrgan,
    Mallet,
    SoftPluck,
    AcousticGuitar,
    Ukulele,
    ElectricGuitar,
}

/// Determine instrument generator function based on section/genre
fn determine_instrument(
    section: Option<Section>,
    genre: Option<Genre>,
    rng: &mut impl Rng,
) -> InstrumentType {
    // For pads/atmospheric sections, use warm organ
    if section == Some(Section::Intro) || section == Some(Section::Bridge) {
        if rng.gen_range(0..100) < 40 {
            return InstrumentType::WarmOrgan;
        }
    }
    
    // Genre-based selection
    match genre {
        Some(Genre::Jazz) | Some(Genre::Funk) => {
            match rng.gen_range(0..100) {
                0..=50 => InstrumentType::Rhodes,
                51..=70 => InstrumentType::Mallet,
                71..=85 => InstrumentType::SoftPluck,
                _ => InstrumentType::AcousticGuitar,
            }
        }
        Some(Genre::Rock) => {
            match rng.gen_range(0..100) {
                0..=40 => InstrumentType::ElectricGuitar,
                41..=70 => InstrumentType::AcousticGuitar,
                _ => InstrumentType::Rhodes,
            }
        }
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
        InstrumentType::ElectricGuitar => generate_electric_guitar(frequency, duration, velocity, 0.3), // Default distortion
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
    let mut bar = vec![0.0; (bar_duration * SAMPLE_RATE() as f32) as usize];
    
    let chord_tones = chord.get_notes();
    
    // Choose ear candy type
    let candy_type = rng.gen_range(0..100);
    
    if candy_type < 50 {
        generate_on_beat_hits_candy_with_instrument(&mut bar, &chord_tones, bar_duration, rng, instrument);
    } else if candy_type < 75 {
        generate_chord_movement_candy_with_instrument(&mut bar, &chord_tones, bar_duration, rng, instrument);
    } else {
        generate_ghost_note_fills_candy_with_instrument(&mut bar, &chord_tones, bar_duration, rng, instrument);
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
        
        let start_sample = (time * SAMPLE_RATE() as f32) as usize;
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
        
        let start_sample = (time * SAMPLE_RATE() as f32) as usize;
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
        
        let start_sample = (time * SAMPLE_RATE() as f32) as usize;
        for (i, &sample) in ghost.iter().enumerate() {
            let idx = start_sample + i;
            if idx < bar.len() {
                bar[idx] += sample * 0.3;
            }
        }
    }
}

/// Generate ear candy - fun sound effects like bells, beeps, dual notes
fn generate_ear_candy_bar(
    _scale_notes: &[MidiNote],
    chord: &Chord,
    bar_duration: f32,
    rng: &mut impl Rng,
    _melody_cfg: &crate::config::MelodyConfig,
) -> Vec<f32> {
    let mut bar = vec![0.0; (bar_duration * SAMPLE_RATE() as f32) as usize];
    
    let chord_tones = chord.get_notes();
    
    // Choose ear candy type for this bar - favor simpler, more spacious patterns
    let candy_type = rng.gen_range(0..100);
    
    if candy_type < 50 {
        // Simple on-beat hits (most common - very simple)
        generate_on_beat_hits_candy(&mut bar, &chord_tones, bar_duration, rng);
    } else if candy_type < 75 {
        // Chord movement (melodic, not rhythmic)
        generate_chord_movement_candy(&mut bar, &chord_tones, bar_duration, rng);
    } else {
        // Ghost notes (very subtle)
        generate_ghost_note_fills_candy(&mut bar, &chord_tones, bar_duration, rng);
    }
    // Removed syncopated rhythm entirely - it was causing the jungle dnb feel!
    
    bar
}

/// On-beat hits - simple, grounded rhythmic hits (VERY sparse)
fn generate_on_beat_hits_candy(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
) {
    // Only 1-2 hits per bar (down from 2-3) - much sparser!
    let beat_duration = bar_duration / 4.0;
    let beats_to_hit = vec![0, 1, 2, 3];
    let num_hits = rng.gen_range(1..=2);  // Reduced from 2..=3
    
    for i in 0..num_hits {
        let beat = beats_to_hit[i * 2]; // Skip beats (0, 2 instead of 0, 1, 2)
        let time = beat as f32 * beat_duration;
        let note = chord_tones[rng.gen_range(0..chord_tones.len())] + 12;
        let freq = midi_to_freq(note);
        
        // Longer, softer hit (less staccato)
        let hit = generate_rhodes_note(freq, 0.4, 0.6);  // Increased duration, reduced velocity
        
        let start_sample = (time * SAMPLE_RATE() as f32) as usize;
        for (j, &sample) in hit.iter().enumerate() {
            let idx = start_sample + j;
            if idx < bar.len() {
                bar[idx] += sample * 0.35;  // Reduced volume from 0.45
            }
        }
    }
}

/// Syncopated rhythm - offbeat accents
pub fn generate_syncopated_rhythm_candy(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
) {
    generate_syncopated_rhythm_candy_with_instrument(bar, chord_tones, bar_duration, rng, InstrumentType::Rhodes);
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
        if rng.gen_range(0..100) < 70 { // 70% chance for each hit
            let time = beat_offset * beat_duration;
            let note = chord_tones[rng.gen_range(0..chord_tones.len())] + 12;
            let freq = midi_to_freq(note);
            
            // Quick staccato
            let hit = generate_note_with_instrument(instrument, freq, 0.12, 0.6);
            
            let start_sample = (time * SAMPLE_RATE() as f32) as usize;
            for (i, &sample) in hit.iter().enumerate() {
                let idx = start_sample + i;
                if idx < bar.len() {
                    bar[idx] += sample * 0.4;
                }
            }
        }
    }
}

/// Chord movement - simple 1-2 note chord progression (VERY simple and slow)
fn generate_chord_movement_candy(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
) {
    let beat_duration = bar_duration / 4.0;
    let num_chords = rng.gen_range(1..=2);  // Reduced from 2..=3
    
    for i in 0..num_chords {
        let time = (i as f32 * 2.0) * beat_duration;  // Spread out more (every 2 beats instead of 1.5)
        let note_idx = i % chord_tones.len();
        let note = chord_tones[note_idx] + 12;
        let freq = midi_to_freq(note);
        
        // Longer, softer hit (more atmospheric)
        let hit = generate_rhodes_note(freq, 0.5, 0.55);  // Longer duration, softer velocity
        
        let start_sample = (time * SAMPLE_RATE() as f32) as usize;
        for (j, &sample) in hit.iter().enumerate() {
            let idx = start_sample + j;
            if idx < bar.len() {
                bar[idx] += sample * 0.4;  // Reduced volume from 0.5
            }
        }
    }
}

/// Ghost note fills - quiet, subtle rhythmic decoration
fn generate_ghost_note_fills_candy(
    bar: &mut [f32],
    chord_tones: &[MidiNote],
    bar_duration: f32,
    rng: &mut impl Rng,
) {
    let beat_duration = bar_duration / 4.0;
    // Quick fill at end of bar
    let fill_times = vec![3.25, 3.5, 3.75];
    
    for &beat_offset in &fill_times {
        let time = beat_offset * beat_duration;
        let note = chord_tones[rng.gen_range(0..chord_tones.len())] + 12;
        let freq = midi_to_freq(note);
        
        // Very quiet and short (ghost notes)
        let ghost = generate_rhodes_note(freq, 0.08, 0.4);
        
        let start_sample = (time * SAMPLE_RATE() as f32) as usize;
        for (i, &sample) in ghost.iter().enumerate() {
            let idx = start_sample + i;
            if idx < bar.len() {
                bar[idx] += sample * 0.3; // Very quiet
            }
        }
    }
}

/// Generate connected melodic phrases with step-wise motion
pub fn generate_connected_melody_phrase(
    scale_notes: &[MidiNote],
    chord: &Chord,
    bar_duration: f32,
    last_note: MidiNote,
    rng: &mut impl Rng,
    melody_cfg: &crate::config::MelodyConfig,
) -> (Vec<f32>, MidiNote) {
    let mut phrase = vec![0.0; (bar_duration * SAMPLE_RATE() as f32) as usize];
    
    // Get chord tones for this chord (emphasize these)
    let chord_tones = chord.get_notes();
    
    // Generate 1-3 notes per bar (sparse, intentional)
    let num_notes = rng.gen_range(1..=3);
    
    let mut current_note = last_note;
    
    for note_idx in 0..num_notes {
        // Position in the bar (prefer on-beat)
        let beat_position = if rng.gen_range(0..100) < 70 {
            // On beat
            (note_idx as f32) * (4.0 / num_notes as f32)
        } else {
            // Syncopated
            (note_idx as f32) * (4.0 / num_notes as f32) + 0.5
        };
        
        let start_time = (beat_position / 4.0) * bar_duration;
        let duration = bar_duration / (num_notes as f32 * 1.5);
        
        // Choose next note based on MELODIC MOTION
        let midi_note = choose_next_melodic_note(
            current_note,
            scale_notes,
            &chord_tones,
            rng,
        );
        
        current_note = midi_note;
        let frequency = midi_to_freq(midi_note);
        
        // Humanization: Random timing offset (±5-15ms)
        let timing_offset = rng.gen_range(-0.015..0.015);
        let humanized_start = (start_time + timing_offset).max(0.0);
        
        // Humanization: Higher velocity for happier sound
        let base_velocity = if beat_position.fract() < 0.1 { 0.85 } else { 0.75 }; // Brighter!
        let velocity: f32 = base_velocity + rng.gen_range(-0.10..0.15);
        let velocity = velocity.clamp(0.65, 1.0);  // Higher range!
        
        // Humanization: Slight duration variation
        let duration_variation = rng.gen_range(0.9..1.1);
        let humanized_duration = duration * duration_variation;
        
        let note = generate_melody_note(frequency, humanized_duration, velocity, melody_cfg);
        
        // Add to phrase with humanized timing
        let start_sample = (humanized_start * SAMPLE_RATE() as f32) as usize;
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
    let mut steps = Vec::new();       // 1-2 semitones away
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

/// Generate a single melody note with humanization and warmth
pub fn generate_melody_note(frequency: f32, duration: f32, velocity: f32, melody_cfg: &crate::config::MelodyConfig) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    
    // Use config for Rhodes usage percentage
    let use_rhodes = rng.gen_range(0..100) < melody_cfg.rhodes_usage_percent;
    
    if use_rhodes {
        // Use Rhodes for warm, smooth lofi sound
        return generate_rhodes_note(frequency, duration, velocity);
    }
    
    // Fallback to warm synth with humanization
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE() as f32;
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
        
        let mut sample = sine1.next_sample() * 0.6
                       + sine2.next_sample() * 0.2
                       + triangle.next_sample() * 0.2;
        
        // Gentle filter movement
        filter.cutoff = 1500.0 + env_amp * 1000.0;
        sample = filter.process(sample);
        
        samples[i] = sample * env_amp * velocity * 0.7;
    }
    
    samples
}

/// Generate an arpeggio from a chord
pub fn generate_arpeggio(
    chord: &Chord,
    tempo: f32,
    duration: f32,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let sixteenth_duration = beat_duration / 4.0;
    
    let chord_notes = chord.get_notes();
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut arpeggio = vec![0.0; num_samples];
    
    let mut note_idx = 0;
    let mut time = 0.0;
    
    while time < duration {
        let midi_note = chord_notes[note_idx % chord_notes.len()] + 24; // Two octaves up
        let frequency = midi_to_freq(midi_note);
        
        let note = generate_arp_note(frequency, sixteenth_duration * 0.8, 0.2);
        
        let start_sample = (time * SAMPLE_RATE() as f32) as usize;
        for (i, &sample) in note.iter().enumerate() {
            let idx = start_sample + i;
            if idx < arpeggio.len() {
                arpeggio[idx] += sample;
            }
        }
        
        time += sixteenth_duration;
        note_idx += 1;
    }
    
    arpeggio
}

/// Generate a single arpeggio note
pub fn generate_arp_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));
        
        let sample = square_osc.next_sample() * env_amp * velocity;
        samples.push(sample);
    }
    
    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::music_theory::{Key, generate_chord_progression};
    
    #[test]
    fn test_melody_generation() {
        let key = Key::random_funky();
        let chords = generate_chord_progression(&key, 4);
        let melody = generate_melody(&key, &chords, 110.0, 2);
        assert!(!melody.is_empty());
    }
}

