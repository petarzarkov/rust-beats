use super::synthesizer::*;
use crate::composition::music_theory::{Chord, midi_to_freq};
use rand::Rng;

/// Generate a bass line following a chord progression
pub fn generate_bassline(
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
        let frequency = midi_to_freq(root_note);
        
        // Generate funky bass pattern for this bar
        let pattern = generate_funk_bass_pattern(frequency, bar_duration);
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
        let start_sample = (start_time * SAMPLE_RATE as f32) as usize;
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
    let total_samples = (bar_duration * SAMPLE_RATE as f32) as usize;
    pattern.resize(total_samples, 0.0);
    
    pattern
}

/// Generate a single bass note
fn generate_bass_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let rng = rand::thread_rng();
    
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
        let time = i as f32 / SAMPLE_RATE as f32;
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
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut sine_osc = Oscillator::new(Waveform::Sine, frequency * 0.5); // Octave down
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Subtle envelope
        let env = 1.0 - (time / duration).powf(2.0) * 0.3;
        
        let sample = sine_osc.next_sample() * env * amplitude;
        samples.push(sample);
    }
    
    samples
}

/// Synth bass - analog-style with sawtooth/square waves
pub fn generate_synth_bass_note(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE as f32;
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
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE as f32;
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
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE as f32;
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
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
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
        let time = i as f32 / SAMPLE_RATE as f32;
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

