use super::synthesizer::*;
use rand::Rng;

/// Generate a soft, sub-heavy lofi kick drum with variance
pub fn generate_kick(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    
    // Add variance: duration, pitch, decay
    let duration = rng.gen_range(0.6..0.75);
    let start_pitch = rng.gen_range(110.0..135.0);  // Vary starting pitch
    let decay_rate = rng.gen_range(6.0..8.0);       // Vary decay speed
    
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Kick is a pitched sine wave with pitch bend to deep sub
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
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
    
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
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
    
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    // Hi-hat with more noise, less metallic for darker lofi character
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let cutoff_base = 8500.0 * brightness;  // Apply brightness variance
    let mut filter = LowPassFilter::new(cutoff_base, 0.3);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
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
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(2000.0, 0.3);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
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
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
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
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(10000.0, 0.2);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Quick burst
        let amp_env = (-time * 30.0).exp();
        
        let mut sample = noise_osc.next_sample() * 0.5;
        sample = filter.process(sample);
        
        samples.push(sample * amp_env * amplitude);
    }
    
    samples
}

/// Generate a rock kick drum: longer attack, more punch, distortion
pub fn generate_rock_kick(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.4..0.5);
    let start_pitch = rng.gen_range(80.0..100.0);  // Lower, more powerful
    let decay_rate = rng.gen_range(4.0..5.0);      // Slower decay for sustain
    
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Pitch envelope: slower decay for more body
        let pitch = start_pitch * (1.0 - time * decay_rate).max(0.3);
        
        // Amplitude envelope: longer attack, more sustain
        let amp_env = if time < 0.01 {
            time / 0.01  // Gradual attack
        } else {
            (-(time - 0.01) * 5.0).exp()  // Slower decay
        };
        
        // Generate tone with harmonics for punch
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let mut sample = phase.sin() * amp_env;
        
        // Add harmonics for punch
        sample += (phase * 2.0).sin() * 0.3 * amp_env;
        sample += (phase * 3.0).sin() * 0.15 * amp_env;
        
        // Distortion/saturation for rock character
        sample = sample.tanh() * 1.2;
        
        // Strong click transient
        if time < 0.003 {
            let click = (rand::random::<f32>() * 2.0 - 1.0) * 0.3 * (1.0 - time / 0.003);
            sample += click;
        }
        
        samples.push(sample * amplitude * 1.1);
    }
    
    samples
}

/// Generate a dubstep kick: sub-bass heavy, sidechain-style ducking
pub fn generate_dubstep_kick(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.5..0.7);  // Longer for sub-bass
    let start_pitch = rng.gen_range(50.0..70.0);  // Very low sub-bass
    let decay_rate = rng.gen_range(2.0..3.0);     // Very slow decay
    
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Pitch envelope: very slow decay for sustained sub
        let pitch = start_pitch * (1.0 - time * decay_rate).max(0.2);
        
        // Amplitude envelope: quick attack, long sustain, slow release
        let amp_env = if time < 0.005 {
            time / 0.005  // Quick attack
        } else if time < 0.1 {
            1.0  // Sustained
        } else {
            (-(time - 0.1) * 3.0).exp()  // Slow release
        };
        
        // Pure sine wave for clean sub-bass
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let mut sample = phase.sin() * amp_env;
        
        // Add sub-harmonic (octave down) for extra weight
        sample += (phase * 0.5).sin() * 0.6 * amp_env;
        
        // Sidechain-style ducking effect (slight volume modulation)
        let duck = 1.0 - (time * 8.0).sin() * 0.1;
        sample *= duck;
        
        samples.push(sample * amplitude * 1.2);  // Very loud
    }
    
    samples
}

/// Generate a DnB snare: sharp, snappy, layered with reverb tail
pub fn generate_dnb_snare(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.15..0.25);  // Short and snappy
    let decay_speed = rng.gen_range(20.0..25.0);
    let freq1 = rng.gen_range(200.0..250.0);   // Higher, sharper
    let freq2 = rng.gen_range(400.0..500.0);
    
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Fast, sharp envelope
        let amp_env = (-time * decay_speed).exp();
        
        // Sharp body frequencies
        let phase1 = 2.0 * std::f32::consts::PI * freq1 * time;
        let phase2 = 2.0 * std::f32::consts::PI * freq2 * time;
        let body = (phase1.sin() * 0.6 + phase2.sin() * 0.4) * amp_env;
        
        // Prominent noise for snare character
        let noise = noise_osc.next_sample() * 0.7 * amp_env;
        
        // Sharp attack transient
        let transient = if time < 0.002 {
            noise_osc.next_sample() * 0.5 * (1.0 - time / 0.002)
        } else {
            0.0
        };
        
        // Reverb tail simulation (exponential decay)
        let reverb_tail = if time > 0.05 {
            noise_osc.next_sample() * 0.2 * (-(time - 0.05) * 10.0).exp()
        } else {
            0.0
        };
        
        let sample = (body + noise + transient + reverb_tail) * amplitude;
        samples.push(sample);
    }
    
    samples
}

/// Generate a rock snare: powerful, compressed, with room reverb
pub fn generate_rock_snare(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.25..0.35);
    let decay_speed = rng.gen_range(12.0..16.0);
    let freq1 = rng.gen_range(150.0..180.0);  // Lower, more body
    let freq2 = rng.gen_range(250.0..300.0);
    
    let num_samples = (duration * SAMPLE_RATE() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE() as f32;
        
        // Envelope with compression feel (slower initial decay)
        let amp_env = if time < 0.01 {
            1.0  // Sustained attack
        } else {
            (-(time - 0.01) * decay_speed).exp()
        };
        
        // Body frequencies
        let phase1 = 2.0 * std::f32::consts::PI * freq1 * time;
        let phase2 = 2.0 * std::f32::consts::PI * freq2 * time;
        let body = (phase1.sin() * 0.7 + phase2.sin() * 0.3) * amp_env;
        
        // Moderate noise
        let noise = noise_osc.next_sample() * 0.5 * amp_env;
        
        // Powerful attack transient
        let transient = if time < 0.003 {
            noise_osc.next_sample() * 0.4 * (1.0 - time / 0.003)
        } else {
            0.0
        };
        
        // Room reverb simulation
        let reverb = if time > 0.02 {
            noise_osc.next_sample() * 0.15 * (-(time - 0.02) * 8.0).exp()
        } else {
            0.0
        };
        
        // Compression effect (tanh saturation)
        let mut sample = body + noise + transient + reverb;
        sample = sample.tanh() * 1.1;
        
        samples.push(sample * amplitude * 1.05);
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

