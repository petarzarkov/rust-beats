use super::synthesizer::*;

/// Heavy Rhythm Guitar - Generates a full Power Chord (Root + 5th + Octave) per trigger
pub fn generate_heavy_rhythm_guitar(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Palm mute logic: Low velocity = closed filter (chug), High velocity = open filter
    let mute_factor = (1.0 - velocity).max(0.0); 
    
    // Envelope
    let envelope = Envelope {
        attack: 0.005,
        decay: 0.15 + (velocity * 0.2), // Longer decay on harder hits
        sustain: 0.5 + (velocity * 0.5),
        release: 0.05,
    };

    // OSCILLATORS: The "Wall of Sound"
    // To sound heavy, we don't just play one note. We play the Root and the 5th.
    // When these two distort together, they create intermodulation distortion characteristic of metal.
    let mut osc_root = Oscillator::new(Waveform::Saw, freq);
    let mut osc_fifth = Oscillator::new(Waveform::Saw, freq * 1.4983); // Perfect 5th
    let mut osc_oct = Oscillator::new(Waveform::Saw, freq * 2.0); // Octave for definition

    // Pre-distortion Filter (The "Wah" or Pick attack focus)
    // Dynamic cutoff based on velocity (The Palm Mute)
    let cutoff_base = 300.0;
    let cutoff_mod = 3000.0 * velocity * velocity; // Exponential opening
    let mut filter = LowPassFilter::new(cutoff_base + cutoff_mod, 0.6);

    // Cabinet Sim (Fixed EQ curve)
    let mut cab_low_cut = LowPassFilter::new(90.0, 0.0); // Tighten low end

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(duration * 0.95));

        // 1. Sum Oscillators
        let raw = osc_root.next_sample() * 0.5 
                + osc_fifth.next_sample() * 0.4 
                + osc_oct.next_sample() * 0.2;

        // 2. Apply Dynamic Filter (Palm Mute simulation)
        // If muted, the filter closes down quickly after attack
        if mute_factor > 0.3 {
            filter.cutoff = (cutoff_base + cutoff_mod) * (1.0 - time * 10.0).max(0.2);
        }
        let filtered = filter.process(raw);

        // 3. HARD Distortion (The Amp)
        // Gain staging: Boost signal -> Clip -> Attenuate
        let drive = 50.0; // Massive gain
        let driven = filtered * drive;
        let distorted = driven.tanh(); // Soft clip edges but hard center

        // 4. Cabinet Simulation (Simple Low Pass + High Pass)
        // Remove high fizz > 5k, Remove mud < 100
        let cabbed = cab_low_cut.process(distorted);
        // We simulate high cut by just mixing some raw sine in? No, simple math:
        // Use a crude iterative lowpass for the cab roll-off
        let final_tone = cabbed * 0.2 + (cabbed * 0.8 * (1.0 - time * 2.0).max(0.0)); // Fake dampening

        samples[i] = final_tone * env_amp * 1.5; // Boost output
    }

    samples
}

/// Lead Guitar - Screaming, high gain, sustained
pub fn generate_lead_guitar(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    let envelope = Envelope { attack: 0.02, decay: 0.0, sustain: 1.0, release: 0.2 };
    
    // Square wave gives that "flute-like" high gain sustain
    let mut osc1 = Oscillator::new(Waveform::Square, freq);
    let mut osc2 = Oscillator::new(Waveform::Saw, freq * 1.003); // Detune
    let mut lfo = LFO::new(6.0, 0.015); // Vibrato

    let mut filter = LowPassFilter::new(2500.0, 0.7);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(duration * 0.9));
        
        // Vibrato delay (kicks in after note starts)
        let vib_amount = (time * 5.0).min(1.0);
        let vib = lfo.next_value() * vib_amount;
        
        osc1.frequency = freq * (1.0 + vib);
        osc2.frequency = freq * 1.003 * (1.0 + vib);

        let signal = (osc1.next_sample() + osc2.next_sample()) * 0.5;
        let filtered = filter.process(signal);
        
        // Fuzz distortion
        let distorted = (filtered * 20.0).clamp(-0.95, 0.95);
        
        samples[i] = distorted * env_amp * velocity * 0.8;
    }
    samples
}

// Fallback
pub fn generate_electric_guitar(freq: f32, duration: f32, velocity: f32, _: f32) -> Vec<f32> {
    generate_heavy_rhythm_guitar(freq, duration, velocity)
}