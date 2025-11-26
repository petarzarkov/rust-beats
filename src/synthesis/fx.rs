/// Audio effects and transitions for production polish
use super::synthesizer::*;

/// Generate a white noise sweep (riser/downlifter)
/// start_freq and end_freq define the filter sweep range
pub fn generate_white_noise_sweep(duration: f32, start_freq: f32, end_freq: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = ResonantFilter::new(start_freq, 0.7); // Use resonant filter for more bite

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let progress = time / duration;

        // Exponential sweep for more natural sound
        let log_start = start_freq.ln();
        let log_end = end_freq.ln();
        let log_freq = log_start + (log_end - log_start) * progress;
        filter.cutoff = log_freq.exp();

        // Amplitude envelope: fade in and crescendo
        let amp_env = (progress * std::f32::consts::PI / 2.0).sin() * progress.sqrt();

        let mut sample = noise_osc.next_sample();
        sample = filter.process(sample);

        // Add some distortion for metal texture
        sample = (sample * 1.5).tanh();

        samples.push(sample * amp_env * 0.5);
    }

    samples
}

/// Generate an upward riser (build-up effect)
pub fn generate_riser(duration: f32) -> Vec<f32> {
    generate_white_noise_sweep(duration, 200.0, 8000.0)
}

/// Generate a downward lifter (breakdown effect)
pub fn generate_downlifter(duration: f32) -> Vec<f32> {
    generate_white_noise_sweep(duration, 8000.0, 200.0)
}

/// Generate a heavy metal crash cymbal
pub fn generate_crash(decay: f32) -> Vec<f32> {
    let duration = decay;
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut metal_osc = Oscillator::new(Waveform::Square, 300.0); // Metallic ring

    // Multiple filters for complex spectrum
    let mut filter_high = ResonantFilter::new(12000.0, 0.1);
    let mut filter_mid = ResonantFilter::new(5000.0, 0.3);
    let mut filter_low = ResonantFilter::new(1000.0, 0.2);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Explosive envelope
        let amp_env = (-time * (3.0 / decay)).exp();

        let noise = noise_osc.next_sample();
        let metal = metal_osc.next_sample() * 0.2; // Add some metallic tone

        let mix = noise + metal;

        let high = filter_high.process(mix) * 0.5;
        let mid = filter_mid.process(mix) * 0.4;
        let low = filter_low.process(mix) * 0.3;

        let sample = (high + mid + low) * amp_env;

        samples.push(sample * 0.6);
    }

    samples
}

/// Generate a heavy impact hit (sub drop + noise burst)
pub fn generate_impact() -> Vec<f32> {
    let duration = 1.5; // Longer tail
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut sub_osc = Oscillator::new(Waveform::Sine, 60.0); // Sub drop start freq
    let mut filter = LowPassFilter::new(3000.0, 0.8);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Fast attack, long decay
        let amp_env = (-time * 2.0).exp();

        // Pitch drop for sub
        let sub_freq = 60.0 * (-time * 3.0).exp();
        sub_osc.frequency = sub_freq;

        let sub = sub_osc.next_sample() * 0.8;
        let noise = noise_osc.next_sample() * 0.4;

        let sample = filter.process(sub + noise) * amp_env;

        // Add saturation
        let distorted = (sample * 1.2).tanh();

        samples.push(distorted * 0.8);
    }

    samples
}

/// Generate guitar feedback squeal
pub fn generate_feedback(duration: f32, freq: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut osc1 = Oscillator::new(Waveform::Sine, freq);
    let mut osc2 = Oscillator::new(Waveform::Sine, freq * 2.0); // Octave up
    let mut lfo = Oscillator::new(Waveform::Sine, 6.0); // Vibrato

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        
        // Slow fade in
        let fade_in = (time / (duration * 0.5)).min(1.0);
        
        // Vibrato
        let vibrato = lfo.next_sample() * 5.0;
        osc1.frequency = freq + vibrato;
        osc2.frequency = (freq * 2.0) + vibrato;

        let s1 = osc1.next_sample();
        let s2 = osc2.next_sample() * 0.5;

        let mix = s1 + s2;
        
        // Heavy distortion
        let distorted = (mix * 5.0).tanh();

        samples.push(distorted * fade_in * 0.4);
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riser() {
        let riser = generate_riser(2.0);
        assert!(!riser.is_empty());
        assert!(riser.len() > 80000);
    }

    #[test]
    fn test_crash() {
        let crash = generate_crash(2.0);
        assert!(!crash.is_empty());
    }

    #[test]
    fn test_impact() {
        let impact = generate_impact();
        assert!(!impact.is_empty());
    }
}
