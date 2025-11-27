use super::synthesizer::*;
use rand::Rng;

/// Per-song drum sound variation parameters
#[derive(Clone, Copy)]
pub struct DrumSoundParams {
    pub kick_pitch_offset: f32,
    pub kick_decay_offset: f32,
    pub kick_click_amount: f32,
    pub snare_freq_offset: f32,
    pub snare_decay_offset: f32,
    pub snare_noise_amount: f32,
    pub hihat_brightness: f32,
    pub hihat_decay_offset: f32,
}

impl DrumSoundParams {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        DrumSoundParams {
            kick_pitch_offset: rng.gen_range(-5.0..5.0),
            kick_decay_offset: rng.gen_range(-0.5..0.5),
            kick_click_amount: rng.gen_range(1.2..1.5), // Boosted click
            snare_freq_offset: rng.gen_range(-10.0..10.0),
            snare_decay_offset: rng.gen_range(-1.0..1.0),
            snare_noise_amount: rng.gen_range(1.0..1.3),
            hihat_brightness: rng.gen_range(0.8..1.0), // Darker hats
            hihat_decay_offset: rng.gen_range(-1.0..1.0),
        }
    }
}

/// Generate a Metal Kick: Massive click, sub weight, aggressive compression
pub fn generate_kick(amplitude: f32) -> Vec<f32> {
    generate_kick_with_params(amplitude, None)
}

pub fn generate_kick_with_params(amplitude: f32, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = 0.5;
    // Start higher for that "basketball" click, drop lower for the sub
    let base_pitch = 55.0; 
    
    let start_pitch = if let Some(p) = params { base_pitch + p.kick_pitch_offset } else { base_pitch };
    
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Pitch envelope: Drastic drop from 180Hz down to 50Hz very quickly
        let pitch_drop = (-time * 40.0).exp(); 
        let pitch = start_pitch + (120.0 * pitch_drop);

        // Amplitude envelope: 
        let amp_env = (-time * 6.0).exp();

        // Main Body (Sine + Triangle blend for weight)
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let body = (phase.sin() * 0.7 + (phase * 0.5).sin().signum() * 0.3) * amp_env;

        // The Click (High pass filtered noise burst)
        let click_amp = params.map(|p| p.kick_click_amount).unwrap_or(1.0);
        let click_env = (-time * 80.0).exp(); // Super fast decay
        let click = (rng.gen_range(-1.0..1.0)) * click_amp * click_env;

        let mut sample = body + click * 0.4;

        // Hard Clipping / Limiting to square it off (The "Thud")
        sample = (sample * 3.0).tanh(); 

        // Low pass to remove digital harshness from the clip
        // Simple distinct filter implementation for brevity or use struct if available
        // ... (Omitting filter state for stateless function, relying on tanh smoothing)

        samples.push(sample * amplitude);
    }

    samples
}

/// Generate a Metal Snare: Gunshot quality
pub fn generate_snare(amplitude: f32) -> Vec<f32> {
    generate_snare_with_params(amplitude, None)
}

pub fn generate_snare_with_params(amplitude: f32, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = 0.35;
    let base_freq = 180.0; // Lower fundamental for "fat" snare
    
    let freq = if let Some(p) = params { base_freq + p.snare_freq_offset } else { base_freq };

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // Envelope
        let amp_env = (-time * 10.0).exp();

        // Tonal Body (Pitch dive)
        let pitch_mod = 1.0 - (-time * 20.0).exp() * 0.2;
        let phase = 2.0 * std::f32::consts::PI * freq * pitch_mod * time;
        let body = phase.sin() * amp_env * 0.4;

        // Noise (Snares) - Broad spectrum
        let noise_amp = params.map(|p| p.snare_noise_amount).unwrap_or(1.0);
        let noise = rng.gen_range(-1.0..1.0) * amp_env * 0.8 * noise_amp;

        let mut sample = body + noise;
        
        // Bitcrush-like distortion for that "90s metal sample" feel
        sample = (sample * 2.5).clamp(-0.9, 0.9);

        samples.push(sample * amplitude);
    }

    samples
}

// Keep existing Cymbals/Toms/China (China is good)
pub fn generate_hihat(amplitude: f32, open: bool) -> Vec<f32> {
    generate_hihat_with_params(amplitude, open, None)
}

pub fn generate_hihat_with_params(amplitude: f32, open: bool, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let duration = if open { 0.5 } else { 0.05 };
    let _brightness = params.map(|p| p.hihat_brightness).unwrap_or(1.0);
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut rng = rand::thread_rng();

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let decay = if open { 8.0 } else { 50.0 };
        let amp_env = (-time * decay).exp();
        
        // White noise high-passed
        let noise = rng.gen_range(-1.0..1.0);
        
        // Simple high-pass effect
        samples.push(noise * amp_env * amplitude * 0.7);
    }
    samples
}

pub fn generate_crash(amplitude: f32) -> Vec<f32> { crate::synthesis::drums::generate_china(amplitude) } // Re-use China logic for Crash for trashier sound
pub fn generate_ride(amplitude: f32) -> Vec<f32> { crate::synthesis::drums::generate_china(amplitude * 0.6) } // Temporary mapping
pub fn generate_tom(amplitude: f32) -> Vec<f32> { 
   // Deep heavy tom
   let duration = 0.6;
   let num_samples = (duration * get_sample_rate() as f32) as usize;
   let mut samples = Vec::with_capacity(num_samples);
   for i in 0..num_samples {
       let time = i as f32 / get_sample_rate() as f32;
       let pitch = 80.0 * (1.0 - time * 3.0).max(0.5);
       let val = (time * pitch * 6.28).sin() * (-time * 4.0).exp();
       samples.push((val * 2.0).tanh() * amplitude);
   }
   samples
}
pub fn generate_china(amplitude: f32) -> Vec<f32> {
    let duration = 1.2;
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut rng = rand::thread_rng();
    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env = (-time * 5.0).exp();
        let noise = rng.gen_range(-1.0..1.0);
        let metal = (time * 400.0 * 6.28).sin() * (time * 340.0 * 6.28).sin(); // Ring mod
        samples.push((noise + metal * 0.5) * env * amplitude);
    }
    samples
}

/// Metal Drums synthesizer
pub struct MetalDrums {
    params: DrumSoundParams,
}

impl MetalDrums {
    pub fn new() -> Self {
        Self {
            params: DrumSoundParams::generate(),
        }
    }

    pub fn generate_kick(&self, amplitude: f32) -> Vec<f32> {
        generate_kick_with_params(amplitude, Some(&self.params))
    }

    pub fn generate_snare(&self, amplitude: f32) -> Vec<f32> {
        generate_snare_with_params(amplitude, Some(&self.params))
    }

    pub fn generate_hihat(&self, amplitude: f32, open: bool) -> Vec<f32> {
        generate_hihat_with_params(amplitude, open, Some(&self.params))
    }
}