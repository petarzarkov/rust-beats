use super::synthesizer::*;
use rand::Rng;

/// Generate a soft, sub-heavy lofi kick drum with variance
pub fn generate_kick(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    
    // Add variance: duration, pitch, decay
    let duration = rng.gen_range(0.6..0.75);
    let start_pitch = rng.gen_range(110.0..135.0);  // Vary starting pitch
    let decay_rate = rng.gen_range(6.0..8.0);       // Vary decay speed
    
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Kick is a pitched sine wave with pitch bend to deep sub
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Pitch envelope: varied per song
        let pitch = start_pitch * (1.0 - time * decay_rate).max(0.29);
        
        // Amplitude envelope: punchier, tighter
        let amp_env = (-time * 6.5).exp();  // Faster decay for punch
        
        // Generate the tone - more sub, less click
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let mut sample = phase.sin() * amp_env * 0.85;
        
        // Add more sub-harmonic for deep lofi bass
        let sub = (phase * 0.5).sin() * 0.5 * amp_env;
        sample += sub;
        
        // Minimal click transient for softer attack
        if time < 0.005 {
            let click = (rand::random::<f32>() * 2.0 - 1.0) * 0.15 * (1.0 - time / 0.005);
            sample += click;
        }
        
        samples.push(sample * amplitude * 1.0);  // Fuller volume
    }
    
    samples
}

/// Generate a muted, soft lofi snare drum with variance
pub fn generate_snare(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    
    // Add variance: duration, tuning, decay
    let duration = rng.gen_range(0.30..0.40);
    let decay_speed = rng.gen_range(14.0..18.0);
    let freq1 = rng.gen_range(170.0..200.0);  // Vary body frequencies
    let freq2 = rng.gen_range(280.0..330.0);
    
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Envelope for the snare - varied decay
        let amp_env = (-time * decay_speed).exp();
        
        // Body: Varied frequencies per song
        let phase1 = 2.0 * std::f32::consts::PI * freq1 * time;
        let phase2 = 2.0 * std::f32::consts::PI * freq2 * time;
        let body = (phase1.sin() * 0.5 + phase2.sin() * 0.3) * amp_env;
        
        // Snare rattle: Less prominent noise for muted character
        let noise = noise_osc.next_sample() * 0.45 * amp_env;
        
        // Softer attack transient
        let transient = if time < 0.004 {
            noise_osc.next_sample() * 0.25 * (1.0 - time / 0.004)
        } else {
            0.0
        };
        
        let sample = (body + noise + transient) * amplitude * 0.95;  // More present
        samples.push(sample);
    }
    
    samples
}

/// Generate a dark, muted lofi hi-hat sound with variance
pub fn generate_hihat(amplitude: f32, open: bool) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    
    // Add variance: duration, brightness
    let duration_base = if open { 0.45 } else { 0.06 };
    let duration = duration_base * rng.gen_range(0.9..1.15);
    let brightness = rng.gen_range(0.85..1.15);  // Vary timbre
    
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Hi-hat with more noise, less metallic for darker lofi character
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let cutoff_base = 8500.0 * brightness;  // Apply brightness variance
    let mut filter = LowPassFilter::new(cutoff_base, 0.3);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        
        // Softer envelopes for lofi feel
        let amp_env = if open {
            (-time * 4.5).exp()
        } else {
            (-time * 30.0).exp()
        };
        
        // Generate noise (more prominent)
        let mut sample = noise_osc.next_sample() * 0.9;
        
        // Filter for darkness (varied per song)
        sample = filter.process(sample);
        
        // Vary metallic character with brightness
        let metallic1 = (2.0 * std::f32::consts::PI * 7000.0 * time).sin() * 0.08 * brightness;
        let metallic2 = (2.0 * std::f32::consts::PI * 10000.0 * time).sin() * 0.06 * brightness;
        sample += metallic1 + metallic2;
        
        samples.push(sample * amp_env * amplitude * 0.75);
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

