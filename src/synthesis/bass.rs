use super::synthesizer::*;
use crate::composition::music_theory::{midi_to_freq, Chord};
use rand::Rng;

/// Generate a funky bass pattern for one bar with variance
pub fn generate_funk_bass_pattern(root_freq: f32, bar_duration: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let note_duration = bar_duration / 8.0; // 8th notes

    let mut pattern = Vec::new();

    // Choose pattern variation (different patterns per bar)
    let pattern_type = rng.gen_range(0..100);
    let hits = if pattern_type < 40 {
        // Standard funk pattern
        vec![
            (0.0, 1.0, 1.0),  // Beat 1 - strong
            (0.5, 0.5, 0.7),  // Off-beat
            (1.0, 0.8, 0.8),  // Beat 2
            (1.75, 0.4, 0.6), // Syncopation
            (2.0, 1.0, 1.0),  // Beat 3 - strong
            (2.5, 0.5, 0.7),  // Off-beat
            (3.0, 0.6, 0.8),  // Beat 4
            (3.5, 0.4, 0.6),  // Off-beat
        ]
    } else if pattern_type < 70 {
        // Sparse pattern (fewer notes)
        vec![
            (0.0, 1.2, 1.0), // Beat 1 - strong
            (2.0, 1.0, 0.9), // Beat 3
            (3.5, 0.6, 0.7), // Late off-beat
        ]
    } else if pattern_type < 85 {
        // Dense pattern (more notes)
        vec![
            (0.0, 0.8, 1.0),  // Beat 1
            (0.5, 0.6, 0.8),  // Off-beat
            (1.0, 0.7, 0.8),  // Beat 2
            (1.5, 0.5, 0.7),  // Syncopation
            (2.0, 0.9, 1.0),  // Beat 3
            (2.5, 0.6, 0.8),  // Off-beat
            (3.0, 0.7, 0.8),  // Beat 4
            (3.5, 0.5, 0.7),  // Off-beat
            (3.75, 0.4, 0.6), // Extra syncopation
        ]
    } else {
        // Syncopated pattern (more off-beats)
        vec![
            (0.0, 1.0, 1.0),  // Beat 1
            (0.75, 0.6, 0.8), // Syncopated
            (1.5, 0.7, 0.8),  // Syncopated
            (2.0, 1.0, 1.0),  // Beat 3
            (2.75, 0.6, 0.8), // Syncopated
            (3.5, 0.7, 0.8),  // Syncopated
        ]
    };

    for (beat_pos, duration_mult, velocity) in hits {
        // Add swing timing variation to off-beats
        let beat_pos_f32 = beat_pos as f32;
        let beat_pos_fract = beat_pos_f32 - (beat_pos_f32.floor());
        let swing_offset = if beat_pos_fract > 0.4 && beat_pos_fract < 0.6 {
            // Off-beats get slight swing delay
            rng.gen_range(0.0..0.05) * (bar_duration / 4.0)
        } else {
            0.0
        };
        let start_time = beat_pos * (bar_duration / 4.0) + swing_offset;

        // Vary note durations more
        let duration_variation = rng.gen_range(0.85..1.15);
        let duration = note_duration * duration_mult * duration_variation;

        // More pitch variation for interest
        let freq_mult = match rng.gen_range(0..100) {
            0..=75 => 1.0,  // Root (most of the time)
            76..=85 => 0.5, // Octave down (more common)
            86..=92 => 1.5, // Fifth (more common)
            93..=96 => 2.0, // Octave up (occasional)
            _ => 1.25,      // Major third (rare)
        };

        // Vary velocity more dynamically
        let velocity_variation = rng.gen_range(0.9..1.1);
        let velocity_f32: f32 = velocity * 0.65 * velocity_variation;
        let final_velocity = velocity_f32.clamp(0.4f32, 0.9f32);

        let note = generate_bass_note(root_freq * freq_mult, duration, final_velocity);

        // Add to pattern at the right position
        let start_sample = (start_time * get_sample_rate() as f32) as usize;
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
    let total_samples = (bar_duration * get_sample_rate() as f32) as usize;
    pattern.resize(total_samples, 0.0);

    pattern
}

/// Generate a single bass note
pub fn generate_bass_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Bass envelope: VERY soft attack, gentle sustain (not aggressive)
    let envelope = Envelope {
        attack: 0.020, // Even softer attack
        decay: 0.25,   // Longer, gentler decay
        sustain: 0.6,  // Lower sustain for less intensity
        release: 0.20, // Even longer release
    };

    let note_off_time = duration * 0.88;

    // Always use sine for maximum warmth and consistency
    let harmonic_waveform = Waveform::Sine;

    // Use mostly sine waves for warm, round bass
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency);
    let mut sine_osc2 = Oscillator::new(Waveform::Sine, frequency * 0.5); // Sub octave
    let mut harmonic_osc = Oscillator::new(harmonic_waveform, frequency);
    let mut filter = LowPassFilter::new(400.0, 0.3); // VERY low, static cutoff for pure warm bass

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        // Mix multiple sine layers for warm, round bass
        let sub_octave = sine_osc2.next_sample() * 0.6; // More deep sub
        let fundamental = sine_osc.next_sample() * 0.5; // Main tone
        let harmonics = harmonic_osc.next_sample() * 0.1; // Minimal character
        let mut sample = sub_octave + fundamental + harmonics;

        // STATIC filter - no movement! Just gentle roll-off
        sample = filter.process(sample);

        samples.push(sample * env_amp * velocity * 0.75); // Softer overall
    }

    samples
}

/// Generate a sub-bass drone with subtle saturation
pub fn generate_sub_bass(frequency: f32, duration: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Use fundamental and sub-octave for richer sub-bass
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency);
    let mut sub_octave_osc = Oscillator::new(Waveform::Sine, frequency * 0.5); // Octave down

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Mix fundamental and sub-octave
        let fundamental = sine_osc.next_sample();
        let sub_octave = sub_octave_osc.next_sample();
        let mut sample = fundamental * 0.6 + sub_octave * 0.6;
        
        // Subtle saturation for warmth (keep it clean)
        sample = (sample * 1.2).tanh() * 0.95;

        // Subtle envelope
        let env = 1.0 - (time / duration).powf(2.0) * 0.3;

        sample = sample * env * amplitude;
        samples.push(sample);
    }

    samples
}

/// Synth bass - analog-style with sawtooth/square waves
pub fn generate_synth_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    let envelope = Envelope {
        attack: 0.005,
        decay: 0.12,
        sustain: 0.7,
        release: 0.15,
    };

    // Sawtooth for analog character
    let mut saw1 = Oscillator::new(Waveform::Saw, freq);
    let mut saw2 = Oscillator::new(Waveform::Saw, freq * 0.995); // Slightly detuned
    let mut square = Oscillator::new(Waveform::Square, freq * 0.5); // Sub octave

    let mut filter = LowPassFilter::new(800.0, 0.5);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, None);

        // Mix sawtooth layers with sub
        let mut sample =
            saw1.next_sample() * 0.4 + saw2.next_sample() * 0.4 + square.next_sample() * 0.3;

        // Filter sweep
        filter.cutoff = 500.0 + env_amp * 600.0;
        sample = filter.process(sample);

        samples[i] = sample * env_amp * velocity * 0.7;
    }

    samples
}

/// Upright bass - woody, pizzicato tone
pub fn generate_upright_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Short attack, medium decay for plucked upright sound
    let envelope = Envelope {
        attack: 0.008,
        decay: 0.25,
        sustain: 0.3,
        release: 0.18,
    };

    // Emphasis on fundamental with specific harmonics for woody tone
    let mut fund = Oscillator::new(Waveform::Sine, freq);
    let mut h2 = Oscillator::new(Waveform::Sine, freq * 2.0);
    let mut h3 = Oscillator::new(Waveform::Sine, freq * 3.0);

    let mut filter = LowPassFilter::new(500.0, 0.3);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, None);

        let mut sample =
            fund.next_sample() * 0.6 + h2.next_sample() * 0.2 + h3.next_sample() * 0.15;

        sample = filter.process(sample);
        samples[i] = sample * env_amp * velocity * 0.75;
    }

    samples
}

/// Finger bass - smooth, rounded attack
pub fn generate_finger_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Softer attack than current bass
    let envelope = Envelope {
        attack: 0.025,
        decay: 0.3,
        sustain: 0.65,
        release: 0.22,
    };

    // Pure sine layers for smooth finger style
    let mut fund = Oscillator::new(Waveform::Sine, freq);
    let mut sub = Oscillator::new(Waveform::Sine, freq * 0.5);
    let mut h2 = Oscillator::new(Waveform::Sine, freq * 2.0);

    let mut filter = LowPassFilter::new(450.0, 0.25);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, None);

        let mut sample =
            fund.next_sample() * 0.5 + sub.next_sample() * 0.4 + h2.next_sample() * 0.1;

        sample = filter.process(sample);
        samples[i] = sample * env_amp * velocity * 0.65;
    }

    samples
}

/// Slap bass - percussive, funky attack
pub fn generate_slap_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Very fast attack with sharp transient
    let envelope = Envelope {
        attack: 0.001,
        decay: 0.08,
        sustain: 0.2,
        release: 0.12,
    };

    // Triangle for brightness, with harmonics
    let mut tri = Oscillator::new(Waveform::Triangle, freq);
    let mut h2 = Oscillator::new(Waveform::Triangle, freq * 2.0);
    let mut h3 = Oscillator::new(Waveform::Sine, freq * 3.0);

    // Add percussive click at attack
    let mut rng = rand::thread_rng();

    let mut filter = LowPassFilter::new(1800.0, 0.6);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, None);

        let mut sample =
            tri.next_sample() * 0.45 + h2.next_sample() * 0.3 + h3.next_sample() * 0.15;

        // Add sharp transient at the beginning (slap sound)
        if i < 200 {
            let click_env = (-(i as f32) / 50.0).exp();
            let click = (rng.gen_range(0.0..1.0) - 0.5) * 0.4;
            sample += click * click_env;
        }

        filter.cutoff = 1500.0 + env_amp * 1000.0;
        sample = filter.process(sample);

        samples[i] = sample * env_amp * velocity * 0.8;
    }

    samples
}

/// Generate a generic bassline using funk patterns
pub fn generate_bassline(
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    bass_drop_cfg: &crate::config::BassDropConfig,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;

    let mut bassline = Vec::new();
    let mut rng = rand::thread_rng();

    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        let root_note = chord.root;
        let root_freq = midi_to_freq(root_note);

        let mut pattern = generate_funk_bass_pattern(root_freq, bar_duration);

        // Add occasional bass drop using config values
        let should_drop = if bar_idx > 0 && bar_idx % 8 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.default_chance_8th_bar
        } else if bar_idx > 0 && bar_idx % 12 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.default_chance_12th_bar
        } else {
            false
        };

        if should_drop {
            // Add a sub-bass drop on beat 1
            let drop_freq = root_freq * 0.5; // One octave down
            let drop_duration = beat_duration * bass_drop_cfg.default_duration_beats;
            let drop =
                generate_sub_bass_drop(drop_freq, drop_duration, bass_drop_cfg.amplitude);

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

/// Generate a rock bassline: root-fifth power chord patterns, palm-muted staccato
pub fn generate_rock_bassline(
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    bass_drop_cfg: &crate::config::BassDropConfig,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;

    let mut bassline = Vec::new();
    let mut rng = rand::thread_rng();

    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        let root_note = chord.root;
        let root_freq = midi_to_freq(root_note);
        let fifth_freq = midi_to_freq(root_note + 7); // Perfect fifth

        let mut pattern = generate_rock_bass_pattern(root_freq, fifth_freq, bar_duration);

        // Add occasional bass drop using config values
        let should_drop = if bar_idx > 0 && bar_idx % 8 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.rock_chance_8th_bar
        } else if bar_idx > 0 && bar_idx % 12 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.rock_chance_12th_bar
        } else {
            false
        };

        if should_drop {
            // Add a sub-bass drop on beat 1
            let drop_freq = root_freq * 0.5; // One octave down
            let drop_duration = beat_duration * bass_drop_cfg.rock_duration_beats;
            let drop =
                generate_sub_bass_drop(drop_freq, drop_duration, bass_drop_cfg.rock_amplitude);

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

/// Generate a rock bass pattern: driving eighth-note patterns with power chords
fn generate_rock_bass_pattern(root_freq: f32, fifth_freq: f32, bar_duration: f32) -> Vec<f32> {
    let note_duration = bar_duration / 8.0; // 8th notes

    let mut pattern = Vec::new();

    // Driving eighth-note pattern: root-fifth-root-fifth
    let hits = vec![
        (0.0, root_freq, 0.8),  // Beat 1
        (0.5, fifth_freq, 0.6), // Off-beat
        (1.0, root_freq, 0.7),  // Beat 2
        (1.5, fifth_freq, 0.5), // Off-beat
        (2.0, root_freq, 0.9),  // Beat 3 (strong)
        (2.5, fifth_freq, 0.6), // Off-beat
        (3.0, root_freq, 0.7),  // Beat 4
        (3.5, fifth_freq, 0.5), // Off-beat
    ];

    for (beat_pos, freq, velocity) in hits {
        let start_time = beat_pos * (bar_duration / 4.0);
        let duration = note_duration * 0.7; // Staccato (70% duration)

        let note = generate_rock_bass_note(freq, duration, velocity);

        let start_sample = (start_time * get_sample_rate() as f32) as usize;
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

    let total_samples = (bar_duration * get_sample_rate() as f32) as usize;
    pattern.resize(total_samples, 0.0);

    pattern
}

/// Generate a rock bass note: distorted, punchy, with pick attack
pub fn generate_rock_bass_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let envelope = Envelope {
        attack: 0.005, // Quick attack (pick)
        decay: 0.15,
        sustain: 0.5,
        release: 0.10,
    };

    // Use sawtooth for aggressive character
    let mut saw_osc = Oscillator::new(Waveform::Saw, frequency);
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency);

    let mut filter = LowPassFilter::new(800.0, 0.5);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, None);

        // Mix sawtooth and sine for character
        let mut sample = saw_osc.next_sample() * 0.6 + sine_osc.next_sample() * 0.4;

        // Distortion/saturation
        sample = sample.tanh() * 1.3;

        // Pick attack transient
        if time < 0.002 {
            let click = (rand::random::<f32>() * 2.0 - 1.0) * 0.2 * (1.0 - time / 0.002);
            sample += click;
        }

        sample = filter.process(sample);
        samples.push(sample * env_amp * velocity);
    }

    samples
}

/// Generate a dubstep bassline: wobble bass patterns, sub-bass drops
pub fn generate_dubstep_bassline(chords: &[Chord], tempo: f32, bars: usize) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;

    let mut bassline = Vec::new();

    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        let root_note = chord.root;
        let root_freq = midi_to_freq(root_note);

        let pattern = generate_dubstep_bass_pattern(root_freq, bar_duration);
        bassline.extend(pattern);
    }

    bassline
}

/// Generate a dubstep bass pattern: wobble on beats, sub-bass drops
fn generate_dubstep_bass_pattern(root_freq: f32, bar_duration: f32) -> Vec<f32> {
    let mut pattern = Vec::new();
    let mut rng = rand::thread_rng();

    // Dubstep is typically 140 BPM, half-time feel (70 BPM)
    // We work with 16th notes grid
    let note_16th = bar_duration / 16.0;
    
    // Select a rhythm pattern template
    let pattern_type = rng.gen_range(0..100);
    
    let events = if pattern_type < 30 {
        // Classic "Wobble - Drop - Wobble - Drop" (Half-time heavy)
        // Beat 1: Long heavy wobble
        // Beat 2: Kick/Snare space (silence/sub)
        // Beat 3: Fast wobbles
        // Beat 4: Drop
        vec![
            (0, 4, "wobble_heavy"),    // Beat 1
            (4, 4, "sub_sustain"),     // Beat 2
            (8, 2, "wobble_fast"),     // Beat 3.1
            (10, 2, "wobble_fast"),    // Beat 3.2
            (12, 4, "drop_slide"),     // Beat 4
        ]
    } else if pattern_type < 60 {
        // Aggressive "Machine Gun" style
        // Rapid fire triplets or 8th notes
        vec![
            (0, 2, "wobble_mid"),
            (2, 2, "wobble_mid"),
            (4, 4, "growl"),
            (8, 2, "wobble_fast"),
            (10, 2, "wobble_fast"),
            (12, 1, "wobble_fast"),
            (13, 1, "wobble_fast"),
            (14, 2, "sub_hit"),
        ]
    } else if pattern_type < 85 {
        // "Sustain & Growl" (Atmospheric)
        vec![
            (0, 8, "growl_long"),
            (8, 4, "wobble_slow"),
            (12, 4, "sub_sustain"),
        ]
    } else {
        // "Broken" rhythm (Syncopated)
        vec![
            (0, 3, "wobble_heavy"),
            (3, 3, "wobble_heavy"),
            (6, 2, "sub_hit"),
            (8, 4, "growl"),
            (12, 2, "wobble_fast"),
            (14, 2, "drop_slide"),
        ]
    };

    // Fill 16th note grid
    let total_samples = (bar_duration * get_sample_rate() as f32) as usize;
    pattern.resize(total_samples, 0.0);

    for (start_16th, len_16ths, sound_type) in events {
        let start_time = start_16th as f32 * note_16th;
        let duration = len_16ths as f32 * note_16th;
        
        let sound = match sound_type {
            "wobble_heavy" => generate_wobble_bass(root_freq, duration, 0.9, 3.0), // Slow heavy LFO
            "wobble_mid" => generate_wobble_bass(root_freq, duration, 0.85, 6.0),  // Medium LFO
            "wobble_fast" => generate_wobble_bass(root_freq, duration, 0.8, 12.0), // Fast LFO (triplets speed)
            "wobble_slow" => generate_wobble_bass(root_freq, duration, 0.8, 1.5),  // Very slow sweep
            "growl" => generate_growl_bass(root_freq, duration, 0.9),
            "growl_long" => generate_growl_bass(root_freq, duration, 0.85),
            "sub_sustain" => generate_sub_bass(root_freq * 0.5, duration, 0.9),
            "sub_hit" => generate_sub_bass_drop(root_freq * 0.5, duration, 1.0),
            "drop_slide" => generate_tape_stop_bass(root_freq, duration, 0.9),
            _ => generate_sub_bass(root_freq, duration, 0.8),
        };

        let start_sample = (start_time * get_sample_rate() as f32) as usize;
        for (i, &sample) in sound.iter().enumerate() {
            let idx = start_sample + i;
            if idx < pattern.len() {
                pattern[idx] += sample;
            }
        }
    }

    pattern
}

/// Generate growl bass (formant-like filter + proper FM phase modulation)
fn generate_growl_bass(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // FM Synthesis with proper phase modulation
    // Carrier oscillator (will be phase-modulated)
    let mut carrier_phase = 0.0;
    let mut modulator = Oscillator::new(Waveform::Sine, frequency * 2.5); // Modulator frequency
    
    // Use resonant filter for formant-like character
    // Temporarily use LowPassFilter until ResonantFilter is fully debugged
    let mut filter = LowPassFilter::new(1000.0, 0.85);

    let sample_rate = get_sample_rate() as f32;
    let two_pi = 2.0 * std::f32::consts::PI;

    for i in 0..num_samples {
        let time = i as f32 / sample_rate;
        let norm_time = time / duration; // 0.0 to 1.0

        // Modulator envelope - increases modulation index over time for aggressive growl
        let mod_idx = 8.0 + 4.0 * (norm_time * two_pi).sin(); // 4.0 to 12.0 modulation index
        
        // Get modulator signal
        let mod_signal = modulator.next_sample();
        
        // Proper FM: modulate phase, not frequency
        // Phase modulation: phase += mod_signal * mod_index
        let phase_mod = mod_signal * mod_idx;
        let modulated_phase = carrier_phase + phase_mod;
        
        // Generate carrier with phase modulation (sawtooth waveform)
        let saw_phase = modulated_phase.fract();
        let mut sample = 2.0 * saw_phase - 1.0; // Sawtooth from phase
        
        // Add harmonics for richer growl
        let harmonic2 = (modulated_phase * 2.0 * two_pi).sin() * 0.3;
        let harmonic3 = (modulated_phase * 3.0 * two_pi).sin() * 0.15;
        sample += harmonic2 + harmonic3;

        // Advance carrier phase
        carrier_phase += frequency / sample_rate;
        carrier_phase = carrier_phase.fract();

        // Formant filter sweep - creates "talking" effect
        // Multiple formant peaks: 300Hz, 800Hz, 2000Hz
        let formant1_env = (norm_time * two_pi * 0.5).sin();
        let formant2_env = (norm_time * two_pi * 0.7 + 0.3).sin();
        
        // Sweep between formant frequencies
        let cutoff_base = 300.0 + formant1_env * 700.0; // 300-1000Hz
        let cutoff_peak = 1500.0 + formant2_env * 1000.0; // 1500-2500Hz
        filter.cutoff = cutoff_base + (cutoff_peak - cutoff_base) * 0.6;
        
        sample = filter.process(sample);
        
        // Heavy distortion after filtering for aggressive character
        sample = (sample * 3.0).tanh();
        
        // Additional saturation for warmth
        sample = (sample * 1.5).tanh() * 0.9;

        samples.push(sample * velocity);
    }
    samples
}

/// Generate tape stop bass (pitch drop) with proper sub-bass
fn generate_tape_stop_bass(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut osc = Oscillator::new(Waveform::Square, frequency);
    let mut sub = Oscillator::new(Waveform::Sine, frequency * 0.5);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let norm_time = time / duration;
        
        // Pitch drops exponentially
        let pitch_mod = 1.0 - norm_time.powf(0.5); // Fast drop
        let current_freq = frequency * pitch_mod;
        
        osc.frequency = current_freq;
        sub.frequency = current_freq * 0.5;

        // Mix square and sub, with subtle saturation on sub
        let square = osc.next_sample();
        let sub_raw = sub.next_sample();
        let sub_saturated = (sub_raw * 1.2).tanh() * 0.9;
        let sample = square * 0.5 + sub_saturated * 0.7;
        
        samples.push(sample * velocity);
    }
    samples
}

/// Generate wobble bass: square wave with LFO on low-pass filter cutoff
pub fn generate_wobble_bass(frequency: f32, duration: f32, velocity: f32, lfo_rate_hz: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Layer multiple oscillators for rich harmonics
    let mut square_osc = Oscillator::new(Waveform::Square, frequency);
    let mut saw_osc = Oscillator::new(Waveform::Saw, frequency);
    let mut sub_osc = Oscillator::new(Waveform::Sine, frequency * 0.5); // Sub octave

    // Use resonant filter for proper dubstep character
    // Temporarily use LowPassFilter with high resonance until ResonantFilter is fully debugged
    let mut filter = LowPassFilter::new(2000.0, 0.9); // High resonance for peak

    // LFO oscillator for wobble (use triangle for sharper transitions)
    let mut lfo_osc = Oscillator::new(Waveform::Triangle, lfo_rate_hz);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Get LFO value (-1 to 1) and convert to 0-1 range
        let lfo_raw = lfo_osc.next_sample();
        let lfo = (lfo_raw + 1.0) * 0.5; // Normalize to 0-1
        
        // Dramatic cutoff sweep: 200Hz to 8000Hz
        let cutoff_min = 200.0;
        let cutoff_max = 8000.0;
        filter.cutoff = cutoff_min + lfo * (cutoff_max - cutoff_min);

        // Mix square and sawtooth for aggressive character
        let mut sample = square_osc.next_sample() * 0.6 + saw_osc.next_sample() * 0.4;
        
        // Heavy distortion before filter for grit and aggression
        sample = (sample * 3.5).tanh();
        
        // Apply resonant filter (creates the characteristic wobble peak)
        sample = filter.process(sample);
        
        // Add clean sub-bass (not filtered, stays pure)
        let sub = sub_osc.next_sample();
        // Subtle saturation on sub for warmth
        let sub_saturated = (sub * 1.2).tanh() * 0.85;
        sample += sub_saturated * 0.8;

        // Sharper envelope for punch
        let env = if time < 0.005 {
            time / 0.005 // Faster attack
        } else if time > duration - 0.02 {
            (duration - time) / 0.02 // Faster release
        } else {
            1.0
        };

        samples.push(sample * env * velocity);
    }

    samples
}

/// Generate sub-bass drop: pure sine wave sub-bass with subtle saturation
pub fn generate_sub_bass_drop(frequency: f32, duration: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Use fundamental and sub-octave for richer sub-bass
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency);
    let mut sub_octave_osc = Oscillator::new(Waveform::Sine, frequency * 0.5);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Mix fundamental and sub-octave
        let fundamental = sine_osc.next_sample();
        let sub_octave = sub_octave_osc.next_sample();
        let mut sample = fundamental * 0.7 + sub_octave * 0.5;
        
        // Subtle saturation for warmth (not too much to keep it clean)
        sample = (sample * 1.3).tanh() * 0.95;

        // Quick attack, sustained, slow release
        let env = if time < 0.01 {
            time / 0.01
        } else if time < duration * 0.8 {
            1.0
        } else {
            (-(time - duration * 0.8) * 2.0).exp()
        };

        sample = sample * env * amplitude;
        samples.push(sample);
    }

    samples
}

/// Generate a DnB bassline: fast rolling patterns, reese bass
pub fn generate_dnb_bassline(
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    bass_drop_cfg: &crate::config::BassDropConfig,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;

    let mut bassline = Vec::new();
    let mut rng = rand::thread_rng();

    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        let root_note = chord.root;
        let root_freq = midi_to_freq(root_note);

        let mut pattern = generate_dnb_bass_pattern(root_freq, bar_duration);

        // Add occasional bass drop using config values
        let should_drop = if bar_idx > 0 && bar_idx % 8 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.dnb_chance_8th_bar
        } else if bar_idx > 0 && bar_idx % 16 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.dnb_chance_16th_bar
        } else {
            false
        };

        if should_drop {
            // Add a sub-bass drop on beat 1
            let drop_freq = root_freq * 0.5; // One octave down
            let drop_duration = beat_duration * bass_drop_cfg.dnb_duration_beats;
            let drop =
                generate_sub_bass_drop(drop_freq, drop_duration, bass_drop_cfg.dnb_amplitude);

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

/// Generate a DnB bass pattern: fast 16th note rolls
fn generate_dnb_bass_pattern(root_freq: f32, bar_duration: f32) -> Vec<f32> {
    let note_duration = bar_duration / 16.0; // 16th notes

    let mut pattern = Vec::new();

    // Fast rolling pattern: reese bass on every 16th note
    for i in 0..16 {
        let beat_pos = i as f32;
        let start_time = beat_pos * note_duration;

        // Vary velocity for groove
        let velocity = if i % 4 == 0 {
            0.9 // Strong on beats
        } else {
            0.6 // Softer on off-beats
        };

        // Use reese bass
        let note = generate_rees_bass(root_freq, note_duration * 0.8, velocity);

        let start_sample = (start_time * get_sample_rate() as f32) as usize;
        if start_sample + note.len() > pattern.len() {
            pattern.resize(start_sample + note.len(), 0.0);
        }

        for (j, &sample) in note.iter().enumerate() {
            let idx = start_sample + j;
            if idx < pattern.len() {
                pattern[idx] += sample;
            }
        }
    }

    let total_samples = (bar_duration * get_sample_rate() as f32) as usize;
    pattern.resize(total_samples, 0.0);

    pattern
}

/// Generate reese bass: detuned saw waves with chorus effect
pub fn generate_rees_bass(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Detuned saw waves (chorus effect)
    let detune1 = frequency * 0.99; // Slightly flat
    let detune2 = frequency * 1.01; // Slightly sharp

    let mut saw1 = Oscillator::new(Waveform::Saw, detune1);
    let mut saw2 = Oscillator::new(Waveform::Saw, detune2);
    let mut saw3 = Oscillator::new(Waveform::Saw, frequency);

    // Filter sweep
    let mut filter = LowPassFilter::new(2000.0, 0.6);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Mix detuned saws for chorus effect
        let mut sample =
            saw1.next_sample() * 0.33 + saw2.next_sample() * 0.33 + saw3.next_sample() * 0.34;

        // Filter sweep (low-pass opens up)
        filter.cutoff = 500.0 + (time / duration) * 1500.0;
        sample = filter.process(sample);

        // Envelope
        let env = if time < duration * 0.1 {
            time / (duration * 0.1)
        } else {
            (-(time - duration * 0.1) * 8.0).exp()
        };

        samples.push(sample * env * velocity * 0.7);
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::music_theory::{generate_chord_progression, Key};

    #[test]
    fn test_bassline_generation() {
        use crate::config::BassDropConfig;
        let key = Key::random_funky();
        let chords = generate_chord_progression(&key, 4);
        // Create a default bass drop config for testing
        let bass_drop_cfg = BassDropConfig {
            default_chance_8th_bar: 0.6,
            default_chance_12th_bar: 0.4,
            rock_chance_8th_bar: 0.5,
            rock_chance_12th_bar: 0.3,
            dnb_chance_8th_bar: 0.5,
            dnb_chance_16th_bar: 0.4,
            amplitude: 0.4,
            rock_amplitude: 0.35,
            dnb_amplitude: 0.4,
            default_duration_beats: 1.5,
            rock_duration_beats: 1.5,
            dnb_duration_beats: 2.0,
        };
        let bassline = generate_bassline(&chords, 110.0, 4, &bass_drop_cfg);
        assert!(!bassline.is_empty());
    }

    #[test]
    fn test_synth_bass() {
        let note = generate_synth_bass_note(55.0, 0.5, 0.7);
        assert!(!note.is_empty());
    }

    #[test]
    fn test_upright_bass() {
        let note = generate_upright_bass_note(55.0, 0.5, 0.7);
        assert!(!note.is_empty());
    }
}
