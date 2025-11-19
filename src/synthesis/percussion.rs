use super::synthesizer::{Oscillator, Waveform, LowPassFilter, SAMPLE_RATE};
use rand::Rng;

/// Tambourine - jingle rattle sound
pub fn generate_tambourine(velocity: f32) -> Vec<f32> {
    let duration = 0.15;
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = vec![0.0; num_samples];
    
    let mut rng = rand::thread_rng();
    
    // Multiple high-frequency metallic overtones for jingle effect
    let freqs = [2500.0, 3200.0, 4100.0, 5300.0, 6800.0];
    let mut oscs: Vec<Oscillator> = freqs.iter()
        .map(|&f| Oscillator::new(Waveform::Sine, f * rng.gen_range(0.95..1.05)))
        .collect();
    
    let mut filter = LowPassFilter::new(8000.0, 0.3);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env = (-time * 12.0).exp();
        
        // Mix multiple metallic tones
        let mut sample = 0.0;
        for osc in &mut oscs {
            sample += osc.next_sample() * 0.2;
        }
        
        // Add some noise for rattle
        let noise = (rng.gen_range(0.0..1.0) - 0.5) * 0.3;
        sample += noise * env * 0.5;
        
        sample = filter.process(sample);
        samples[i] = sample * env * velocity * 0.7;
    }
    
    samples
}

/// Cowbell - metallic tone
pub fn generate_cowbell(velocity: f32) -> Vec<f32> {
    let duration = 0.25;
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = vec![0.0; num_samples];
    
    // Inharmonic partials for metallic bell sound
    let mut osc1 = Oscillator::new(Waveform::Square, 800.0);
    let mut osc2 = Oscillator::new(Waveform::Square, 540.0);
    
    let mut filter = LowPassFilter::new(3500.0, 0.5);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env = (-time * 8.0).exp();
        
        let mut sample = osc1.next_sample() * 0.5
                       + osc2.next_sample() * 0.4;
        
        sample = filter.process(sample);
        samples[i] = sample * env * velocity * 0.75;
    }
    
    samples
}

/// Bongo - hand drum sound
pub fn generate_bongo(pitch_high: bool, velocity: f32) -> Vec<f32> {
    let duration = 0.20;
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = vec![0.0; num_samples];
    
    let base_freq = if pitch_high { 400.0 } else { 250.0 };
    
    let mut filter = LowPassFilter::new(1200.0, 0.4);
    let mut rng = rand::thread_rng();
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env = (-time * 14.0).exp();
        
        // Pitch envelope - recreate oscillator for pitch bend
        let pitch_bend = 1.0 - (1.0 - (-time * 40.0).exp()) * 0.3;
        let mut osc = Oscillator::new(Waveform::Sine, base_freq * pitch_bend);
        
        let mut sample = osc.next_sample();
        
        // Add snap/skin noise at attack
        if i < 500 {
            let noise_env = (-(i as f32) / 100.0).exp();
            let noise = (rng.gen_range(0.0..1.0) - 0.5) * 0.4;
            sample += noise * noise_env;
        }
        
        sample = filter.process(sample);
        samples[i] = sample * env * velocity * 0.8;
    }
    
    samples
}

/// Woodblock - sharp click
pub fn generate_woodblock(velocity: f32) -> Vec<f32> {
    let duration = 0.08;
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = vec![0.0; num_samples];
    
    let mut osc = Oscillator::new(Waveform::Sine, 1200.0);
    let mut filter = LowPassFilter::new(2500.0, 0.3);
    let mut rng = rand::thread_rng();
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        let env = (-time * 35.0).exp();
        
        let mut sample = osc.next_sample() * 0.3;
        
        // Sharp transient click
        if i < 150 {
            let click_env = (-(i as f32) / 30.0).exp();
            let click = rng.gen_range(0.0..1.0) - 0.5;
            sample += click * click_env * 0.6;
        }
        
        sample = filter.process(sample);
        samples[i] = sample * env * velocity * 0.85;
    }
    
    samples
}

/// Triangle - bell tone
pub fn generate_triangle_perc(velocity: f32) -> Vec<f32> {
    let duration = 1.5;
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = vec![0.0; num_samples];
    
    // High pitched bell-like overtones
    let mut osc1 = Oscillator::new(Waveform::Sine, 2500.0);
    let mut osc2 = Oscillator::new(Waveform::Sine, 3200.0);
    let mut osc3 = Oscillator::new(Waveform::Sine, 4100.0);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        // Very slow decay for ringing triangle
        let env = (-time * 1.5).exp();
        
        let sample = osc1.next_sample() * 0.5
                   + osc2.next_sample() * 0.3
                   + osc3.next_sample() * 0.2;
        
        samples[i] = sample * env * velocity * 0.5;
    }
    
    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tambourine() {
        let tamb = generate_tambourine(0.7);
        assert!(!tamb.is_empty());
    }
    
    #[test]
    fn test_cowbell() {
        let bell = generate_cowbell(0.8);
        assert!(!bell.is_empty());
    }
    
    #[test]
    fn test_bongo() {
        let bongo = generate_bongo(true, 0.7);
        assert!(!bongo.is_empty());
    }
    
    #[test]
    fn test_woodblock() {
        let wood = generate_woodblock(0.8);
        assert!(!wood.is_empty());
    }
    
    #[test]
    fn test_triangle() {
        let tri = generate_triangle_perc(0.6);
        assert!(!tri.is_empty());
    }
}

