use super::synthesizer::*;
use crate::composition::music_theory::{midi_to_freq, Chord};
use rand::Rng;

/// Generate atmospheric pad layer following chord progression
pub fn generate_pads(chords: &[Chord], tempo: f32, bars: usize) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;

    let mut pads = Vec::new();
    let mut rng = rand::thread_rng();

    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];

        // Only add pads 40% of the time for space
        if rng.gen_range(0..100) < 40 {
            let pad_section = generate_pad_chord(chord, bar_duration);
            pads.extend(pad_section);
        } else {
            // Silence
            let silence_samples = (bar_duration * get_sample_rate() as f32) as usize;
            pads.extend(vec![0.0; silence_samples]);
        }
    }

    pads
}

/// Generate a sustained pad chord
fn generate_pad_chord(chord: &Chord, duration: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    let chord_notes = chord.get_notes();
    if chord_notes.is_empty() {
        return samples;
    }

    // Very slow attack and release for pad (longer for lofi ambient feel)
    let envelope = Envelope {
        attack: duration * 0.4,  // 40% of duration for soft attack
        decay: 0.0,              // No decay
        sustain: 1.0,            // Full sustain
        release: duration * 0.5, // 50% of duration for long release
    };

    let note_off_time = duration * 0.8;

    // Layer multiple detuned sine waves for each chord note (pure, warm tones)
    let mut oscillators: Vec<(Oscillator, f32)> = Vec::new();

    for &midi_note in &chord_notes {
        let base_freq = midi_to_freq(midi_note + 12); // Octave up for pads

        // Add 5 detuned sine oscillators per note for rich chorus effect
        oscillators.push((Oscillator::new(Waveform::Sine, base_freq * 0.996), 0.12));
        oscillators.push((Oscillator::new(Waveform::Sine, base_freq * 0.999), 0.14));
        oscillators.push((Oscillator::new(Waveform::Sine, base_freq), 0.18));
        oscillators.push((Oscillator::new(Waveform::Sine, base_freq * 1.001), 0.14));
        oscillators.push((Oscillator::new(Waveform::Sine, base_freq * 1.004), 0.12));

        // Add more sine harmonics for richness (no harsh waveforms)
        oscillators.push((Oscillator::new(Waveform::Sine, base_freq * 2.0), 0.08)); // Octave up
        oscillators.push((Oscillator::new(Waveform::Sine, base_freq * 0.5), 0.10));
        // Sub octave
    }

    // Low-pass filter for warmth (lower cutoff for darker sound)
    let mut filter = LowPassFilter::new(600.0, 0.25);

    // Disable LFO modulation entirely to prevent scratching/alien sounds
    // Static filter only - no movement

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        // Sum all oscillators
        let mut sample = 0.0;
        for (osc, amp) in &mut oscillators {
            sample += osc.next_sample() * *amp;
        }

        // Static filter - no modulation to prevent scratching
        filter.cutoff = 600.0; // Fixed cutoff, no LFO
        sample = filter.process(sample);

        // Very gentle overall amplitude (pads should be subtle and warm)
        samples[i] = sample * env_amp * 0.12;
    }

    samples
}

/// Generate a drone/sustained note (for intros/outros)
pub fn generate_drone(frequency: f32, duration: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Very slow envelope
    let envelope = Envelope {
        attack: duration * 0.4,
        decay: 0.0,
        sustain: 1.0,
        release: duration * 0.4,
    };

    let note_off_time = duration * 0.7;

    // Layer sine waves at different octaves
    let mut osc1 = Oscillator::new(Waveform::Sine, frequency * 0.5); // Sub octave
    let mut osc2 = Oscillator::new(Waveform::Sine, frequency); // Root
    let mut osc3 = Oscillator::new(Waveform::Sine, frequency * 2.0); // Octave up
    let mut osc4 = Oscillator::new(Waveform::Sine, frequency * 3.0); // Octave + fifth

    let mut filter = LowPassFilter::new(600.0, 0.4);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let mut sample = osc1.next_sample() * 0.4
            + osc2.next_sample() * 0.3
            + osc3.next_sample() * 0.2
            + osc4.next_sample() * 0.1;

        sample = filter.process(sample);
        samples[i] = sample * env_amp * amplitude;
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::music_theory::{generate_chord_progression, Key};

    #[test]
    fn test_pad_generation() {
        let key = Key::random_funky();
        let chords = generate_chord_progression(&key, 4);
        let pads = generate_pads(&chords, 110.0, 4);
        assert!(!pads.is_empty());
    }

    #[test]
    fn test_drone_generation() {
        let drone = generate_drone(220.0, 4.0, 0.3);
        assert!(!drone.is_empty());
    }
}
