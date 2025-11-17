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
    let mut rng = rand::thread_rng();
    
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
}

