use crate::utils::get_sample_rate;
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
            kick_click_amount: rng.gen_range(0.8..1.2), // Reduced from 1.2..1.5 to reduce DnB noise
            snare_freq_offset: rng.gen_range(-10.0..10.0),
            snare_decay_offset: rng.gen_range(-1.0..1.0),
            snare_noise_amount: rng.gen_range(1.0..1.3),
            hihat_brightness: rng.gen_range(0.8..1.0), // Darker hats
            hihat_decay_offset: rng.gen_range(-1.0..1.0),
        }
    }
}

/// Generate a Metal Kick: Massive click, sub weight, aggressive compression
/// PHASE 3: ADVANCED DRUM SYNTHESIS - Layered samples, pitch envelope, transient shaping
pub fn generate_kick(amplitude: f32) -> Vec<f32> {
    generate_kick_with_params(amplitude, None)
}

pub fn generate_kick_with_params(amplitude: f32, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = 0.5; // Extended for more sub weight
    let base_pitch = 60.0;
    
    let _start_pitch = if let Some(p) = params { base_pitch + p.kick_pitch_offset } else { base_pitch };
    
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // ===== PHASE 3.1: ENHANCED PITCH ENVELOPE =====
        // Start at 220Hz (beater attack), drop to 35Hz (deep sub)
        let pitch_drop = (-time * 60.0).exp(); // Faster, more aggressive drop
        let pitch = 35.0 + (185.0 * pitch_drop); // Lower sub, higher attack

        // ===== PHASE 3.2: LAYERED SAMPLES =====
        
        // LAYER 1: SUB BODY (Pure sine for deep low end)
        let sub_phase = 2.0 * std::f32::consts::PI * pitch * time;
        let sub_env = (-time * 6.0).exp(); // Slower decay for sub weight
        let sub_layer = sub_phase.sin() * sub_env * 0.6;
        
        // LAYER 2: BEATER ATTACK (Triangle wave for mid punch)
        let beater_phase = 2.0 * std::f32::consts::PI * (pitch * 2.0) * time;
        let beater_env = (-time * 12.0).exp(); // Faster decay
        let beater_layer = (beater_phase * 0.5).sin().signum() * beater_env * 0.3;
        
        // LAYER 3: CLICK (Noise burst for transient)
        let click_amp = params.map(|p| p.kick_click_amount).unwrap_or(1.3); 
        let click_env = (-time * 200.0).exp(); // Very fast decay
        let click_layer = (rng.gen_range(-1.0..1.0)) * click_amp * click_env * 0.4;

        // ===== PHASE 3.3: TRANSIENT SHAPER =====
        // Boost the first 10ms for extreme punch
        let transient_boost = if time < 0.01 {
            1.0 + (1.0 - time / 0.01) * 0.5 // 50% boost in first 10ms
        } else {
            1.0
        };

        // Mix all layers
        let mut sample = (sub_layer + beater_layer + click_layer) * transient_boost;

        // ===== PHASE 3.4: PARALLEL COMPRESSION =====
        // Compress a copy and blend for punch
        let compressed = (sample * 6.0).tanh() * 0.4;
        let dry = sample * 0.6;
        sample = dry + compressed;

        // Final saturation for that "basketball" thud
        sample = (sample * 3.5).tanh(); 

        samples.push(sample * amplitude);
    }

    samples
}

/// Generate a Metal Snare: Gunshot quality
/// PHASE 3: ADVANCED DRUM SYNTHESIS - Layered samples for realistic snare
pub fn generate_snare(amplitude: f32) -> Vec<f32> {
    generate_snare_with_params(amplitude, None)
}

pub fn generate_snare_with_params(amplitude: f32, params: Option<&DrumSoundParams>) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let duration = 0.35; // Extended for more tail
    let base_freq = 190.0;
    
    let freq = if let Some(p) = params { base_freq + p.snare_freq_offset } else { base_freq };

    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;

        // ===== PHASE 3.2: LAYERED SAMPLES =====
        
        // LAYER 1: TONAL BODY (Pitch dive for shell resonance)
        let pitch_mod = 1.0 - (-time * 30.0).exp() * 0.4; // More aggressive pitch dive
        let phase = 2.0 * std::f32::consts::PI * freq * pitch_mod * time;
        let body_env = (-time * 15.0).exp(); // Faster decay
        let body_layer = phase.sin() * body_env * 0.25;

        // LAYER 2: NOISE RATTLE (Snare wires)
        let noise_amp = params.map(|p| p.snare_noise_amount).unwrap_or(1.3);
        let noise_env = (-time * 10.0).exp(); // Snare wire decay
        let noise_layer = rng.gen_range(-1.0..1.0) * noise_env * 0.7 * noise_amp;

        // LAYER 3: CRACK TRANSIENT (Stick attack)
        let crack_env = (-time * 100.0).exp(); // Very fast decay
        let crack_layer = rng.gen_range(-1.0..1.0) * crack_env * 0.5;

        // ===== PHASE 3.3: TRANSIENT SHAPER =====
        // Boost the first 5ms for extreme crack
        let transient_boost = if time < 0.005 {
            1.0 + (1.0 - time / 0.005) * 0.8 // 80% boost in first 5ms
        } else {
            1.0
        };

        // Mix all layers
        let mut sample = (body_layer + noise_layer + crack_layer) * transient_boost;
        
        // ===== PHASE 3.4: PARALLEL COMPRESSION =====
        // Compress a copy and blend for punch
        let compressed = (sample * 5.0).tanh() * 0.5;
        let dry = sample * 0.5;
        sample = dry + compressed;

        // Final hard clipping for that "gunshot" quality
        sample = (sample * 2.5).clamp(-0.95, 0.95);

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

    pub fn generate_crash(&self, amplitude: f32) -> Vec<f32> {
        generate_crash(amplitude)
    }

    pub fn generate_ride(&self, amplitude: f32) -> Vec<f32> {
        generate_ride(amplitude)
    }

    pub fn generate_tom(&self, amplitude: f32) -> Vec<f32> {
        generate_tom(amplitude)
    }

    pub fn generate_china(&self, amplitude: f32) -> Vec<f32> {
        generate_china(amplitude)
    }
}