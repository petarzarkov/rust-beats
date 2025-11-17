use super::synthesizer::*;

/// Generate a kick drum sound
pub fn generate_kick(amplitude: f32) -> Vec<f32> {
    let duration = 0.6;
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Kick is a pitched sine wave with fast pitch bend down
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Pitch envelope: starts at ~150Hz, drops to ~35Hz quickly
        let pitch = 150.0 * (1.0 - time * 10.0).max(0.23);
        
        // Amplitude envelope: quick attack, exponential decay
        let amp_env = (-time * 7.0).exp();
        
        // Generate the tone with slight distortion for more punch
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let mut sample = phase.sin() * amp_env;
        
        // Add sub-harmonic for more bass
        let sub = (phase * 0.5).sin() * 0.3 * amp_env;
        sample += sub;
        
        // Add click transient at the start for punch
        if time < 0.003 {
            let click = (rand::random::<f32>() * 2.0 - 1.0) * 0.4 * (1.0 - time / 0.003);
            sample += click;
        }
        
        samples.push(sample * amplitude);
    }
    
    samples
}

/// Generate a snare drum sound
pub fn generate_snare(amplitude: f32) -> Vec<f32> {
    let duration = 0.3;
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Envelope for the snare - sharper attack
        let amp_env = (-time * 18.0).exp();
        
        // Body: Mixture of sine waves for tone (around 180Hz and 330Hz)
        let phase1 = 2.0 * std::f32::consts::PI * 200.0 * time;
        let phase2 = 2.0 * std::f32::consts::PI * 350.0 * time;
        let body = (phase1.sin() * 0.4 + phase2.sin() * 0.25) * amp_env;
        
        // Snare rattle: High-frequency noise (more prominent)
        let noise = noise_osc.next_sample() * 0.75 * amp_env;
        
        // Add sharp attack transient
        let transient = if time < 0.002 {
            noise_osc.next_sample() * 0.5 * (1.0 - time / 0.002)
        } else {
            0.0
        };
        
        let sample = (body + noise + transient) * amplitude;
        samples.push(sample);
    }
    
    samples
}

/// Generate a hi-hat sound
pub fn generate_hihat(amplitude: f32, open: bool) -> Vec<f32> {
    let duration = if open { 0.5 } else { 0.08 };
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Hi-hat is essentially filtered white noise with metallic overtones
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(10000.0, 0.3);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Envelope - different characteristics for open/closed
        let amp_env = if open {
            (-time * 5.0).exp()
        } else {
            (-time * 35.0).exp()
        };
        
        // Generate noise
        let mut sample = noise_osc.next_sample() * 0.8;
        
        // High-pass characteristic
        sample = filter.process(sample);
        
        // Add metallic character with multiple sine waves
        let metallic1 = (2.0 * std::f32::consts::PI * 9000.0 * time).sin() * 0.15;
        let metallic2 = (2.0 * std::f32::consts::PI * 13000.0 * time).sin() * 0.12;
        sample += metallic1 + metallic2;
        
        samples.push(sample * amp_env * amplitude);
    }
    
    samples
}

/// Generate a clap sound
pub fn generate_clap(amplitude: f32) -> Vec<f32> {
    let duration = 0.15;
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(2000.0, 0.3);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Multiple short bursts for clap effect
        let burst1 = if time < 0.01 { 1.0 } else { 0.0 };
        let burst2 = if time > 0.015 && time < 0.025 { 0.8 } else { 0.0 };
        let burst3 = if time > 0.03 && time < 0.04 { 0.6 } else { 0.0 };
        let burst_env = burst1 + burst2 + burst3;
        
        let amp_env = (-time * 20.0).exp() * burst_env;
        
        let mut sample = noise_osc.next_sample();
        sample = filter.process(sample);
        
        samples.push(sample * amp_env * amplitude);
    }
    
    samples
}

/// Generate a conga/tom sound
pub fn generate_conga(pitch: f32, amplitude: f32) -> Vec<f32> {
    let duration = 0.35;
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Pitch envelope with slight bend
        let freq = pitch * (1.0 - time * 0.5);
        
        // Amplitude envelope
        let amp_env = (-time * 10.0).exp();
        
        // Main tone
        let phase = 2.0 * std::f32::consts::PI * freq * time;
        let mut sample = phase.sin() * 0.7;
        
        // Add some harmonics for character
        sample += (phase * 2.1).sin() * 0.2;
        sample += (phase * 3.3).sin() * 0.1;
        
        samples.push(sample * amp_env * amplitude);
    }
    
    samples
}

/// Generate a shaker sound
pub fn generate_shaker(amplitude: f32) -> Vec<f32> {
    let duration = 0.08;
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(10000.0, 0.2);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Quick burst
        let amp_env = (-time * 30.0).exp();
        
        let mut sample = noise_osc.next_sample() * 0.5;
        sample = filter.process(sample);
        
        samples.push(sample * amp_env * amplitude);
    }
    
    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_drum_generation() {
        let kick = generate_kick(0.5);
        assert!(!kick.is_empty());
        
        let snare = generate_snare(0.5);
        assert!(!snare.is_empty());
        
        let hihat = generate_hihat(0.3, false);
        assert!(!hihat.is_empty());
    }
}

