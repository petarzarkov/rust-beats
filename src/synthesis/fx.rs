/// Audio effects and transitions for production polish
use super::synthesizer::*;

/// Generate a white noise sweep (riser/downlifter)
/// start_freq and end_freq define the filter sweep range
pub fn generate_white_noise_sweep(duration: f32, start_freq: f32, end_freq: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(start_freq, 0.7);

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

        samples.push(sample * amp_env * 0.4); // Reduced from 0.6 to 0.4
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

/// Generate a crash cymbal / impact sound (gentler version)
pub fn generate_crash(decay: f32) -> Vec<f32> {
    let duration = decay;
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);

    // Lower filter cutoffs to reduce harsh high frequencies
    let mut filter_high = LowPassFilter::new(8000.0, 0.3); // Reduced from 12000Hz to 8000Hz
    let mut filter_mid = LowPassFilter::new(4000.0, 0.4);  // Reduced from 6000Hz to 4000Hz
    let mut filter_low = LowPassFilter::new(2000.0, 0.3);  // Added low-pass for warmth

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Gentler, longer decay envelope
        let amp_env = (-time * (2.0 / decay)).exp(); // Slower decay (2.0 instead of 3.0)

        // Softer attack transient
        let attack = if time < 0.015 {
            1.0 + (1.0 - time / 0.015) * 1.2 // Reduced from 2.0 to 1.2, longer attack
        } else {
            1.0
        };

        let noise = noise_osc.next_sample();

        // Layer multiple filtered versions with more emphasis on mid/low
        let high = filter_high.process(noise) * 0.3; // Reduced from 0.6
        let mid = filter_mid.process(noise) * 0.5;  // Increased from 0.4
        let low = filter_low.process(noise) * 0.2;  // Added low layer

        let sample = (high + mid + low) * amp_env * attack;

        samples.push(sample * 0.3); // Further reduced from 0.5 to 0.3
    }

    samples
}

/// Generate a short impact hit (for transitions)
pub fn generate_impact() -> Vec<f32> {
    let duration = 0.3;
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut sine_osc = Oscillator::new(Waveform::Sine, 80.0);
    let mut filter = LowPassFilter::new(4000.0, 0.8);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Very fast attack, quick decay
        let amp_env = (-time * 15.0).exp();

        // Combine low sine (sub) with filtered noise (body)
        let sub = sine_osc.next_sample() * 0.5;
        let noise = noise_osc.next_sample() * 0.5;

        let sample = filter.process(sub + noise) * amp_env;

        samples.push(sample * 0.6); // Reduced from 0.8 to 0.6
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
