use super::synthesizer::*;
use crate::composition::music_theory::{Chord, midi_to_freq};
use rand::Rng;

/// Generate a bass line following a chord progression
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
        let frequency = midi_to_freq(root_note);
        
        // Generate funky bass pattern for this bar
        let mut pattern = generate_funk_bass_pattern(frequency, bar_duration);
        
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
            let drop = generate_sub_bass(drop_freq, drop_duration, bass_drop_cfg.amplitude);
            
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

/// Generate a funky bass pattern for one bar
fn generate_funk_bass_pattern(root_freq: f32, bar_duration: f32) -> Vec<f32> {
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
        
        let note = generate_bass_note(
            root_freq * freq_mult,
            duration,
            velocity * 0.65, // Gentler - reduced from 0.85
        );
        
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

/// Generate a single bass note
pub fn generate_bass_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Bass envelope: VERY soft attack, gentle sustain (not aggressive)
    let envelope = Envelope {
        attack: 0.020,   // Even softer attack
        decay: 0.25,     // Longer, gentler decay
        sustain: 0.6,    // Lower sustain for less intensity
        release: 0.20,   // Even longer release
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
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));
        
        // Mix multiple sine layers for warm, round bass
        let sub_octave = sine_osc2.next_sample() * 0.6;  // More deep sub
        let fundamental = sine_osc.next_sample() * 0.5;  // Main tone
        let harmonics = harmonic_osc.next_sample() * 0.1; // Minimal character
        let mut sample = sub_octave + fundamental + harmonics;
        
        // STATIC filter - no movement! Just gentle roll-off
        sample = filter.process(sample);
        
        samples.push(sample * env_amp * velocity * 0.75);  // Softer overall
    }
    
    samples
}

/// Generate a sub-bass drone
pub fn generate_sub_bass(frequency: f32, duration: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency * 0.5); // Octave down
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Subtle envelope
        let env = 1.0 - (time / duration).powf(2.0) * 0.3;
        
        let sample = sine_osc.next_sample() * env * amplitude;
        samples.push(sample);
    }
    
    samples
}

/// Synth bass - analog-style with sawtooth/square waves
pub fn generate_synth_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env_amp = envelope.get_amplitude(time, None);
        
        // Mix sawtooth layers with sub
        let mut sample = saw1.next_sample() * 0.4
                       + saw2.next_sample() * 0.4
                       + square.next_sample() * 0.3;
        
        // Filter sweep
        filter.cutoff = 500.0 + env_amp * 600.0;
        sample = filter.process(sample);
        
        samples[i] = sample * env_amp * velocity * 0.7;
    }
    
    samples
}

/// Upright bass - woody, pizzicato tone
pub fn generate_upright_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env_amp = envelope.get_amplitude(time, None);
        
        let mut sample = fund.next_sample() * 0.6
                       + h2.next_sample() * 0.2
                       + h3.next_sample() * 0.15;
        
        sample = filter.process(sample);
        samples[i] = sample * env_amp * velocity * 0.75;
    }
    
    samples
}

/// Finger bass - smooth, rounded attack
pub fn generate_finger_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env_amp = envelope.get_amplitude(time, None);
        
        let mut sample = fund.next_sample() * 0.5
                       + sub.next_sample() * 0.4
                       + h2.next_sample() * 0.1;
        
        sample = filter.process(sample);
        samples[i] = sample * env_amp * velocity * 0.65;
    }
    
    samples
}

/// Slap bass - percussive, funky attack
pub fn generate_slap_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env_amp = envelope.get_amplitude(time, None);
        
        let mut sample = tri.next_sample() * 0.45
                       + h2.next_sample() * 0.3
                       + h3.next_sample() * 0.15;
        
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
        let fifth_freq = midi_to_freq(root_note + 7);  // Perfect fifth
        
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
            let drop = generate_sub_bass_drop(drop_freq, drop_duration, bass_drop_cfg.rock_amplitude);
            
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
        (0.0, root_freq, 0.8),      // Beat 1
        (0.5, fifth_freq, 0.6),     // Off-beat
        (1.0, root_freq, 0.7),      // Beat 2
        (1.5, fifth_freq, 0.5),     // Off-beat
        (2.0, root_freq, 0.9),      // Beat 3 (strong)
        (2.5, fifth_freq, 0.6),     // Off-beat
        (3.0, root_freq, 0.7),      // Beat 4
        (3.5, fifth_freq, 0.5),     // Off-beat
    ];
    
    for (beat_pos, freq, velocity) in hits {
        let start_time = beat_pos * (bar_duration / 4.0);
        let duration = note_duration * 0.7;  // Staccato (70% duration)
        
        let note = generate_rock_bass_note(freq, duration, velocity);
        
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
    
    let total_samples = (bar_duration * SAMPLE_RATE() as f32) as usize;
    pattern.resize(total_samples, 0.0);
    
    pattern
}

/// Generate a rock bass note: distorted, punchy, with pick attack
pub fn generate_rock_bass_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let envelope = Envelope {
        attack: 0.005,   // Quick attack (pick)
        decay: 0.15,
        sustain: 0.5,
        release: 0.10,
    };
    
    // Use sawtooth for aggressive character
    let mut saw_osc = Oscillator::new(Waveform::Saw, frequency);
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency);
    
    let mut filter = LowPassFilter::new(800.0, 0.5);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
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
pub fn generate_dubstep_bassline(
    chords: &[Chord],
    tempo: f32,
    bars: usize,
) -> Vec<f32> {
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
    
    // Wobble bass on beats 1 and 3
    let wobble_duration = bar_duration / 4.0;
    
    // Beat 1: Wobble bass
    let wobble1 = generate_wobble_bass(root_freq, wobble_duration, 0.9);
    pattern.extend(wobble1);
    
    // Beat 2: Sub-bass drop
    let sub1 = generate_sub_bass_drop(root_freq * 0.5, wobble_duration, 1.0);
    pattern.extend(sub1);
    
    // Beat 3: Wobble bass
    let wobble2 = generate_wobble_bass(root_freq, wobble_duration, 0.9);
    pattern.extend(wobble2);
    
    // Beat 4: Sub-bass drop
    let sub2 = generate_sub_bass_drop(root_freq * 0.5, wobble_duration, 1.0);
    pattern.extend(sub2);
    
    pattern
}

/// Generate wobble bass: square wave with LFO on low-pass filter cutoff
pub fn generate_wobble_bass(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Square wave for aggressive character
    let mut square_osc = Oscillator::new(Waveform::Square, frequency);
    
    // LFO for wobble effect (modulating filter cutoff)
    let lfo_rate = 4.0;  // 4 Hz wobble
    let lfo_depth = 0.5;  // Depth of modulation
    let mut filter = LowPassFilter::new(2000.0, 0.7);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // LFO modulates filter cutoff
        let lfo = (2.0 * std::f32::consts::PI * lfo_rate * time).sin();
        filter.cutoff = 500.0 + lfo * lfo_depth * 1500.0;  // 500-2000 Hz sweep
        
        let mut sample = square_osc.next_sample();
        sample = filter.process(sample);
        
        // Envelope
        let env = if time < duration * 0.1 {
            time / (duration * 0.1)
        } else {
            (-(time - duration * 0.1) * 5.0).exp()
        };
        
        samples.push(sample * env * velocity * 0.8);
    }
    
    samples
}

/// Generate sub-bass drop: pure sine wave sub-bass
pub fn generate_sub_bass_drop(frequency: f32, duration: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Quick attack, sustained, slow release
        let env = if time < 0.01 {
            time / 0.01
        } else if time < duration * 0.8 {
            1.0
        } else {
            (-(time - duration * 0.8) * 2.0).exp()
        };
        
        let sample = sine_osc.next_sample() * env * amplitude;
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
            let drop = generate_sub_bass_drop(drop_freq, drop_duration, bass_drop_cfg.dnb_amplitude);
            
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
            0.9  // Strong on beats
        } else {
            0.6  // Softer on off-beats
        };
        
        // Use reese bass
        let note = generate_rees_bass(root_freq, note_duration * 0.8, velocity);
        
        let start_sample = (start_time * SAMPLE_RATE() as f32) as usize;
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
    
    let total_samples = (bar_duration * SAMPLE_RATE() as f32) as usize;
    pattern.resize(total_samples, 0.0);
    
    pattern
}

/// Generate reese bass: detuned saw waves with chorus effect
pub fn generate_rees_bass(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Detuned saw waves (chorus effect)
    let detune1 = frequency * 0.99;  // Slightly flat
    let detune2 = frequency * 1.01;  // Slightly sharp
    
    let mut saw1 = Oscillator::new(Waveform::Saw, detune1);
    let mut saw2 = Oscillator::new(Waveform::Saw, detune2);
    let mut saw3 = Oscillator::new(Waveform::Saw, frequency);
    
    // Filter sweep
    let mut filter = LowPassFilter::new(2000.0, 0.6);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Mix detuned saws for chorus effect
        let mut sample = saw1.next_sample() * 0.33
                       + saw2.next_sample() * 0.33
                       + saw3.next_sample() * 0.34;
        
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
    use crate::composition::music_theory::{Key, generate_chord_progression};
    
    #[test]
    fn test_bassline_generation() {
        let key = Key::random_funky();
        let chords = generate_chord_progression(&key, 4);
        let bassline = generate_bassline(&chords, 110.0, 4);
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

