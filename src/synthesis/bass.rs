use super::synthesizer::*;
use crate::composition::music_theory::{midi_to_freq, Chord};

/// Generate a single metal bass note (distorted, heavy)
pub fn generate_metal_bass_note(freq: f32, duration: f32) -> Vec<f32> {
    let sample_rate = get_sample_rate() as f32;
    let num_samples = (duration * sample_rate) as usize;
    let mut buffer = vec![0.0; num_samples];

    // 1. Oscillators
    // Main sub oscillator (Sine) for weight
    let mut sub_osc = Oscillator::new(Waveform::Sine, freq);
    // Grit oscillator (Saw) for texture
    let mut saw_osc = Oscillator::new(Waveform::Saw, freq);
    // Pulse oscillator for "clank"
    let mut pulse_osc = Oscillator::new(Waveform::Square, freq);

    // 2. Envelopes
    // Amp envelope: Fast attack, long sustain
    let amp_env = Envelope {
        attack: 0.01,
        decay: 0.2,
        sustain: 0.8,
        release: 0.1,
    };
    // Filter envelope: Plucky for attack
    let filter_env = Envelope {
        attack: 0.005,
        decay: 0.1,
        sustain: 0.4,
        release: 0.1,
    };

    // 3. Filter (Low pass with resonance)
    let mut filter = LowPassFilter::new(800.0, 0.5);

    // 4. Distortion
    let distortion = Distortion::new(3.0, 1.0);

    for i in 0..num_samples {
        let time = i as f32 / sample_rate;
        
        // Mix oscillators
        let sub = sub_osc.next_sample() * 0.6;
        let saw = saw_osc.next_sample() * 0.4;
        let pulse = pulse_osc.next_sample() * 0.3;
        
        let raw = sub + saw + pulse;
        
        // Apply envelopes
        let amp_val = amp_env.get_amplitude(time, Some(duration));
        let filter_mod = filter_env.get_amplitude(time, Some(duration));
        
        // Modulate filter cutoff
        filter.cutoff = 200.0 + filter_mod * 2000.0;
        
        let filtered = filter.process(raw);
        
        // Apply distortion and cab sim
        let distorted = distortion.process(filtered);
        let mut cab_sim = CabinetSimulator::new();
        let cab_sound = cab_sim.process(distorted);
        
        buffer[i] = cab_sound * amp_val;
    }

    buffer
}

/// Generate a metal bass pattern (chugging 8th notes)
pub fn generate_metal_bass_pattern(root_freq: f32, bar_duration: f32) -> Vec<f32> {
    let beat_duration = bar_duration / 4.0;
    let eighth_duration = beat_duration / 2.0;
    let mut pattern = Vec::new();

    // 8 eighth notes per bar
    for _ in 0..8 {
        // Slight duration variation for human feel
        let note_dur = eighth_duration * 0.9; 
        let note = generate_metal_bass_note(root_freq, note_dur);
        pattern.extend(note);
        
        // Short gap
        let gap_samples = ((eighth_duration - note_dur) * get_sample_rate() as f32) as usize;
        pattern.extend(vec![0.0; gap_samples]);
    }

    pattern
}

/// Generate a full bassline following chords
pub fn generate_metal_bassline(
    chords: &[Chord],
    bpm: f32,
    bars: usize,
) -> Vec<f32> {
    let mut bassline = Vec::new();
    let beat_duration = 60.0 / bpm;
    let bar_duration = beat_duration * 4.0;

    for i in 0..bars {
        let chord = &chords[i % chords.len()];
        let root_freq = midi_to_freq(chord.root - 12); // Drop octave
        
        let pattern = generate_metal_bass_pattern(root_freq, bar_duration);
        bassline.extend(pattern);
    }

    bassline
}

// Legacy/Unused functions removed as per plan
