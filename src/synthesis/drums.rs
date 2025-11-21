use super::synthesizer::*;
use rand::Rng;

/// Per-song drum sound variation parameters
/// These are generated once per song to give each song distinct drum character
#[derive(Clone, Copy)]
pub struct DrumSoundParams {
    // Kick variations
    pub kick_pitch_offset: f32,      // Added to base pitch range
    pub kick_decay_offset: f32,      // Added to decay rate
    pub kick_click_amount: f32,      // Multiplier for click transient
    
    // Snare variations
    pub snare_freq_offset: f32,      // Added to body frequencies
    pub snare_decay_offset: f32,     // Added to decay speed
    pub snare_noise_amount: f32,     // Multiplier for noise component
    
    // HiHat variations
    pub hihat_brightness: f32,       // Multiplier for brightness/filter cutoff
    pub hihat_decay_offset: f32,     // Added to decay speed
}

impl DrumSoundParams {
    /// Generate random drum sound parameters for a song
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        DrumSoundParams {
            // Kick: vary pitch by ±20Hz, decay by ±1.5, click by 0.7-1.3x
            kick_pitch_offset: rng.gen_range(-20.0..20.0),
            kick_decay_offset: rng.gen_range(-1.5..1.5),
            kick_click_amount: rng.gen_range(0.7..1.3),
            
            // Snare: vary frequencies by ±30Hz, decay by ±2.0, noise by 0.8-1.2x
            snare_freq_offset: rng.gen_range(-30.0..30.0),
            snare_decay_offset: rng.gen_range(-2.0..2.0),
            snare_noise_amount: rng.gen_range(0.8..1.2),
            
            // HiHat: brightness 0.8-1.2x, decay by ±3.0
            hihat_brightness: rng.gen_range(0.8..1.2),
            hihat_decay_offset: rng.gen_range(-3.0..3.0),
        }
    }
}

/// Generate a soft, sub-heavy lofi kick drum with variance
pub fn generate_kick(amplitude: f32) -> Vec<f32> {
    generate_kick_with_params(amplitude, None)
}

/// Generate kick with optional per-song variation parameters
pub fn generate_kick_with_params(amplitude: f32, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let mut rng = rand::thread_rng();

    // Base variance: duration, pitch, decay
    let duration = rng.gen_range(0.6..0.75);
    let base_pitch_range = 110.0..135.0;
    let base_decay_range = 6.0..8.0;
    
    // Apply per-song variation if provided
    let start_pitch = if let Some(p) = params {
        rng.gen_range(base_pitch_range.start + p.kick_pitch_offset..base_pitch_range.end + p.kick_pitch_offset)
    } else {
        rng.gen_range(base_pitch_range)
    };
    let decay_rate = if let Some(p) = params {
        rng.gen_range(base_decay_range.start + p.kick_decay_offset..base_decay_range.end + p.kick_decay_offset)
    } else {
        rng.gen_range(base_decay_range)
    };

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Kick is a pitched sine wave with pitch bend to deep sub
    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Pitch envelope: varied per song
        let pitch = start_pitch * (1.0 - time * decay_rate).max(0.29);

        // Amplitude envelope: punchier, tighter
        let amp_env = (-time * 6.5).exp(); // Faster decay for punch

        // Generate the tone - more sub, less click
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let mut sample = phase.sin() * amp_env * 0.85;

        // Add more sub-harmonic for deep lofi bass
        let sub = (phase * 0.5).sin() * 0.5 * amp_env;
        sample += sub;

        // Minimal click transient for softer attack
        let click_amount = params.map(|p| p.kick_click_amount).unwrap_or(1.0);
        if time < 0.005 {
            let click = (rand::random::<f32>() * 2.0 - 1.0) * 0.15 * click_amount * (1.0 - time / 0.005);
            sample += click;
        }

        samples.push(sample * amplitude * 1.0); // Fuller volume
    }

    samples
}

/// Generate a muted, soft lofi snare drum with variance
pub fn generate_snare(amplitude: f32) -> Vec<f32> {
    generate_snare_with_params(amplitude, None)
}

/// Generate snare with optional per-song variation parameters
pub fn generate_snare_with_params(amplitude: f32, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let mut rng = rand::thread_rng();

    // Base variance: duration, tuning, decay
    let duration = rng.gen_range(0.30..0.40);
    let base_decay_range = 14.0..18.0;
    let base_freq1_range = 170.0..200.0;
    let base_freq2_range = 280.0..330.0;
    
    // Apply per-song variation if provided
    let decay_speed = if let Some(p) = params {
        rng.gen_range(base_decay_range.start + p.snare_decay_offset..base_decay_range.end + p.snare_decay_offset)
    } else {
        rng.gen_range(base_decay_range)
    };
    let freq1 = if let Some(p) = params {
        rng.gen_range(base_freq1_range.start + p.snare_freq_offset..base_freq1_range.end + p.snare_freq_offset)
    } else {
        rng.gen_range(base_freq1_range)
    };
    let freq2 = if let Some(p) = params {
        rng.gen_range(base_freq2_range.start + p.snare_freq_offset..base_freq2_range.end + p.snare_freq_offset)
    } else {
        rng.gen_range(base_freq2_range)
    };

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Envelope for the snare - varied decay
        let amp_env = (-time * decay_speed).exp();

        // Body: Varied frequencies per song
        let phase1 = 2.0 * std::f32::consts::PI * freq1 * time;
        let phase2 = 2.0 * std::f32::consts::PI * freq2 * time;
        let body = (phase1.sin() * 0.5 + phase2.sin() * 0.3) * amp_env;

        // Snare rattle: Less prominent noise for muted character
        let noise_amount = params.map(|p| p.snare_noise_amount).unwrap_or(1.0);
        let noise = noise_osc.next_sample() * 0.45 * noise_amount * amp_env;

        // Softer attack transient
        let transient = if time < 0.004 {
            noise_osc.next_sample() * 0.25 * (1.0 - time / 0.004)
        } else {
            0.0
        };

        let sample = (body + noise + transient) * amplitude * 0.95; // More present
        samples.push(sample);
    }

    samples
}

/// Generate a dark, muted lofi hi-hat sound with variance
pub fn generate_hihat(amplitude: f32, open: bool) -> Vec<f32> {
    generate_hihat_with_params(amplitude, open, None)
}

/// Generate hi-hat with optional per-song variation parameters
pub fn generate_hihat_with_params(amplitude: f32, open: bool, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let mut rng = rand::thread_rng();

    // Add variance: duration, brightness
    let duration_base = if open { 0.45 } else { 0.06 };
    let duration = duration_base * rng.gen_range(0.9..1.15);
    let base_brightness = rng.gen_range(0.85..1.15);
    let brightness = params.map(|p| base_brightness * p.hihat_brightness).unwrap_or(base_brightness);

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Hi-hat with more noise, less metallic for darker lofi character
    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let cutoff_base = 8500.0 * brightness; // Apply brightness variance
    let mut filter = LowPassFilter::new(cutoff_base, 0.3);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Softer envelopes for lofi feel
        let decay_base = if open { 4.5 } else { 30.0 };
        let decay_rate = decay_base + params.map(|p| p.hihat_decay_offset).unwrap_or(0.0);
        let amp_env = (-time * decay_rate).exp();

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
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(2000.0, 0.3);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Multiple short bursts for clap effect
        let burst1 = if time < 0.01 { 1.0 } else { 0.0 };
        let burst2 = if time > 0.015 && time < 0.025 {
            0.8
        } else {
            0.0
        };
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
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

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
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);
    let mut filter = LowPassFilter::new(10000.0, 0.2);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

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
    let start_pitch = rng.gen_range(80.0..100.0); // Lower, more powerful
    let decay_rate = rng.gen_range(4.0..5.0); // Slower decay for sustain

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Pitch envelope: slower decay for more body
        let pitch = start_pitch * (1.0 - time * decay_rate).max(0.3);

        // Amplitude envelope: longer attack, more sustain
        let amp_env = if time < 0.01 {
            time / 0.01 // Gradual attack
        } else {
            (-(time - 0.01) * 5.0).exp() // Slower decay
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

/// Generate a dubstep kick: sub-bass heavy, sidechain-style ducking (more aggressive)
pub fn generate_dubstep_kick(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.5..0.7); // Longer for sub-bass
    let start_pitch = rng.gen_range(50.0..70.0); // Very low sub-bass
    let decay_rate = rng.gen_range(2.0..3.0); // Very slow decay

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Pitch envelope: very slow decay for sustained sub
        let pitch = start_pitch * (1.0 - time * decay_rate).max(0.2);

        // Faster, sharper attack for more punch
        let amp_env = if time < 0.002 {
            time / 0.002 // Faster attack (was 0.005)
        } else if time < 0.1 {
            1.0 // Sustained
        } else {
            (-(time - 0.1) * 3.0).exp() // Slow release
        };

        // Pure sine wave for clean sub-bass
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let mut sample = phase.sin() * amp_env;

        // Add sub-harmonic (octave down) for extra weight
        sample += (phase * 0.5).sin() * 0.7 * amp_env; // Increased from 0.6

        // Add punchy click transient at attack
        if time < 0.003 {
            let click_freq = 200.0 + (1.0 - time / 0.003) * 300.0; // Sweep down
            let click_phase = 2.0 * std::f32::consts::PI * click_freq * time;
            sample += click_phase.sin() * 0.3 * (1.0 - time / 0.003); // Sharp click
        }

        // Compression/saturation for aggression
        sample = (sample * 1.3).tanh() * 1.1;

        // Sidechain-style ducking effect (slight volume modulation)
        let duck = 1.0 - (time * 8.0).sin() * 0.1;
        sample *= duck;

        samples.push(sample * amplitude * 1.3); // Increased from 1.2
    }

    samples
}

/// Generate a DnB snare: sharp, snappy, layered with reverb tail (more aggressive)
pub fn generate_dnb_snare(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.15..0.25); // Short and snappy
    let decay_speed = rng.gen_range(22.0..28.0); // Faster decay for sharper sound
    let freq1 = rng.gen_range(200.0..250.0); // Higher, sharper
    let freq2 = rng.gen_range(400.0..500.0);

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Faster, sharper envelope
        let amp_env = (-time * decay_speed).exp();

        // Sharp body frequencies
        let phase1 = 2.0 * std::f32::consts::PI * freq1 * time;
        let phase2 = 2.0 * std::f32::consts::PI * freq2 * time;
        let body = (phase1.sin() * 0.65 + phase2.sin() * 0.35) * amp_env;

        // More prominent noise for snare character
        let noise = noise_osc.next_sample() * 0.8 * amp_env; // Increased from 0.7

        // Sharper attack transient
        let transient = if time < 0.0015 {
            noise_osc.next_sample() * 0.6 * (1.0 - time / 0.0015) // Faster, sharper
        } else {
            0.0
        };

        // Reverb tail simulation (exponential decay)
        let reverb_tail = if time > 0.05 {
            noise_osc.next_sample() * 0.25 * (-(time - 0.05) * 10.0).exp() // Slightly more
        } else {
            0.0
        };

        let mut sample = body + noise + transient + reverb_tail;
        
        // Compression/saturation for aggression
        sample = (sample * 1.4).tanh() * 1.15;
        
        sample = sample * amplitude;
        samples.push(sample);
    }

    samples
}

/// Generate a rock snare: powerful, compressed, with room reverb
pub fn generate_rock_snare(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.25..0.35);
    let decay_speed = rng.gen_range(12.0..16.0);
    let freq1 = rng.gen_range(150.0..180.0); // Lower, more body
    let freq2 = rng.gen_range(250.0..300.0);

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Envelope with compression feel (slower initial decay)
        let amp_env = if time < 0.01 {
            1.0 // Sustained attack
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

/// Generate a crash cymbal sound
pub fn generate_crash(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(1.5..2.5); // Longer sustain
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Long decay envelope
        let amp_env = (-time * 1.5).exp();

        // High-frequency metallic content
        let mut sample = noise_osc.next_sample() * amp_env;

        // Add metallic harmonics (high frequencies)
        for freq in [4000.0, 6000.0, 8000.0, 10000.0] {
            let phase = 2.0 * std::f32::consts::PI * freq * time;
            sample += phase.sin() * 0.15 * amp_env;
        }

        // Initial transient
        if time < 0.01 {
            sample += noise_osc.next_sample() * 0.5 * (1.0 - time / 0.01);
        }

        samples.push(sample * amplitude * 0.5);
    }

    samples
}

/// Generate a rim shot sound
pub fn generate_rimshot(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = 0.08;
    let freq = rng.gen_range(2000.0..3000.0); // High, sharp pitch
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Very fast decay
        let amp_env = (-time * 40.0).exp();

        // Sharp click tone
        let phase = 2.0 * std::f32::consts::PI * freq * time;
        let tone = phase.sin() * amp_env * 0.7;

        // Noise component for click
        let noise = noise_osc.next_sample() * amp_env * 0.3;

        samples.push((tone + noise) * amplitude);
    }

    samples
}

/// Generate a tom drum sound
pub fn generate_tom(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = 0.5;
    let pitch = rng.gen_range(80.0..150.0); // Mid-range pitch
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Pitch envelope (slight downward bend)
        let pitch_env = pitch * (1.0 - time * 0.3);

        // Amplitude envelope
        let amp_env = (-time * 8.0).exp();

        // Body tone with harmonics
        let phase = 2.0 * std::f32::consts::PI * pitch_env * time;
        let mut sample = phase.sin() * amp_env * 0.8;
        sample += (phase * 2.0).sin() * amp_env * 0.2;

        // Attack transient
        if time < 0.01 {
            sample += (rand::random::<f32>() * 2.0 - 1.0) * 0.3 * (1.0 - time / 0.01);
        }

        samples.push(sample * amplitude);
    }

    samples
}

/// Generate a ride cymbal sound
pub fn generate_ride(amplitude: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(0.4..0.6);
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut noise_osc = Oscillator::new(Waveform::Noise, 0.0);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Medium decay
        let amp_env = (-time * 5.0).exp();

        // Filtered noise for ride character
        let noise = noise_osc.next_sample() * amp_env * 0.6;

        // Bell tone (metallic harmonics)
        let mut bell_tone = 0.0;
        for freq in [3200.0, 4800.0, 6400.0] {
            let phase = 2.0 * std::f32::consts::PI * freq * time;
            bell_tone += phase.sin() * 0.2 * amp_env;
        }

        // Ping transient
        let ping = if time < 0.005 {
            noise_osc.next_sample() * 0.4 * (1.0 - time / 0.005)
        } else {
            0.0
        };

        samples.push((noise + bell_tone + ping) * amplitude * 0.7);
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
