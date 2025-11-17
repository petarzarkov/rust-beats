/// Lofi effects for vintage, warm character
use rand::Rng;
use super::synthesizer::{LowPassFilter, SAMPLE_RATE};

/// Add vinyl crackle and pops to audio
pub fn add_vinyl_crackle(samples: &mut [f32], intensity: f32) {
    let mut rng = rand::thread_rng();
    
    for sample in samples.iter_mut() {
        // Random crackle (continuous low-level noise)
        let crackle = (rng.gen_range(0.0..1.0) - 0.5) * 0.003 * intensity;
        
        // Random pops (occasional louder clicks)
        let pop = if rng.gen_range(0.0..1.0) < 0.0001 * intensity {
            (rng.gen_range(0.0..1.0) - 0.5) * 0.05
        } else {
            0.0
        };
        
        *sample += crackle + pop;
    }
}

/// Tape saturation - adds warmth through soft clipping
pub fn apply_tape_saturation(samples: &mut [f32], drive: f32) {
    for sample in samples.iter_mut() {
        // Soft clipping using tanh for analog warmth
        let driven = *sample * (1.0 + drive * 2.0);
        *sample = driven.tanh() * 0.9; // Scale down to prevent clipping
    }
}

/// Bit crusher - reduce bit depth for lo-fi grit
pub fn apply_bit_crusher(samples: &mut [f32], target_bits: u8) {
    let levels = (1 << target_bits) as f32;
    
    for sample in samples.iter_mut() {
        // Quantize to fewer bits
        let quantized = (*sample * levels).round() / levels;
        *sample = quantized;
    }
}

/// Sample rate reducer - downsample for retro digital sound
pub fn apply_sample_rate_reduction(samples: &mut [f32], target_rate: u32) {
    let reduction_factor = (SAMPLE_RATE / target_rate) as usize;
    
    if reduction_factor <= 1 {
        return;
    }
    
    for i in (0..samples.len()).step_by(reduction_factor) {
        let value = samples[i];
        // Hold the same value for the reduction period
        for j in 0..reduction_factor {
            if i + j < samples.len() {
                samples[i + j] = value;
            }
        }
    }
}

/// Tape wow and flutter - subtle pitch wobble
pub struct TapeWowFlutter {
    phase: f32,
    wow_frequency: f32,      // Very slow pitch wobble (0.5-2 Hz)
    flutter_frequency: f32,   // Faster flutter (4-8 Hz)
}

impl TapeWowFlutter {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        TapeWowFlutter {
            phase: 0.0,
            wow_frequency: rng.gen_range(0.5..1.5),
            flutter_frequency: rng.gen_range(4.0..7.0),
        }
    }
    
    /// Get the current pitch modulation amount (-0.02 to 0.02 semitones)
    pub fn get_modulation(&mut self, time: f32) -> f32 {
        // Combine slow wow with faster flutter
        let wow = (time * self.wow_frequency * 2.0 * std::f32::consts::PI).sin() * 0.015;
        let flutter = (time * self.flutter_frequency * 2.0 * std::f32::consts::PI).sin() * 0.005;
        
        wow + flutter
    }
}

/// Apply tape wow/flutter to buffer (requires resampling)
/// This is a simplified version - adds subtle pitch variation
pub fn apply_wow_flutter(samples: &mut [f32], intensity: f32) {
    let mut wow_flutter = TapeWowFlutter::new();
    let sample_rate_f = SAMPLE_RATE as f32;
    
    // Create a copy for interpolation
    let original = samples.to_vec();
    
    for i in 0..samples.len() {
        let time = i as f32 / sample_rate_f;
        let pitch_shift = wow_flutter.get_modulation(time) * intensity;
        
        // Simple pitch shift by reading at slightly different position
        let read_pos = i as f32 * (1.0 + pitch_shift);
        let read_idx = read_pos as usize;
        
        if read_idx < original.len() - 1 {
            // Linear interpolation
            let frac = read_pos - read_idx as f32;
            samples[i] = original[read_idx] * (1.0 - frac) 
                       + original[read_idx + 1] * frac;
        }
    }
}

/// Roll off high frequencies for warm, muffled vintage sound
pub fn apply_vintage_lowpass(samples: &mut [f32], cutoff: f32) {
    let mut filter = LowPassFilter::new(cutoff, 0.3); // Low resonance for smooth roll-off
    
    for sample in samples.iter_mut() {
        *sample = filter.process(*sample);
    }
}

/// Combined lofi effect chain
pub struct LofiProcessor {
    pub vinyl_intensity: f32,
    pub tape_drive: f32,
    pub bit_depth: u8,
    pub downsample_rate: u32,
    pub wow_flutter_intensity: f32,
    pub vintage_cutoff: f32,
}

impl LofiProcessor {
    /// Preset for subtle lofi character (VERY minimal)
    pub fn subtle() -> Self {
        LofiProcessor {
            vinyl_intensity: 0.05,      // Barely any crackle
            tape_drive: 0.08,           // Minimal saturation
            bit_depth: 16,              // No bit crushing
            downsample_rate: 44100,     // No downsampling
            wow_flutter_intensity: 0.0, // NO pitch wobble!
            vintage_cutoff: 12000.0,    // Gentle high roll-off only
        }
    }
    
    /// Preset for medium lofi character
    pub fn medium() -> Self {
        LofiProcessor {
            vinyl_intensity: 0.6,
            tape_drive: 0.4,
            bit_depth: 12,
            downsample_rate: 22050,
            wow_flutter_intensity: 0.5,
            vintage_cutoff: 8000.0,
        }
    }
    
    /// Preset for heavy lofi character
    pub fn heavy() -> Self {
        LofiProcessor {
            vinyl_intensity: 1.0,
            tape_drive: 0.7,
            bit_depth: 10,
            downsample_rate: 16000,
            wow_flutter_intensity: 0.8,
            vintage_cutoff: 6000.0,
        }
    }
    
    /// Apply all lofi effects in optimal order
    pub fn process(&self, samples: &mut [f32]) {
        // 1. Tape saturation first (adds harmonics)
        if self.tape_drive > 0.0 {
            apply_tape_saturation(samples, self.tape_drive);
        }
        
        // 2. High frequency roll-off (vintage character)
        if self.vintage_cutoff < 15000.0 {
            apply_vintage_lowpass(samples, self.vintage_cutoff);
        }
        
        // 3. Sample rate reduction (retro digital)
        if self.downsample_rate < SAMPLE_RATE {
            apply_sample_rate_reduction(samples, self.downsample_rate);
        }
        
        // 4. Bit crushing (grit)
        if self.bit_depth < 16 {
            apply_bit_crusher(samples, self.bit_depth);
        }
        
        // 5. Wow and flutter (tape character)
        if self.wow_flutter_intensity > 0.0 {
            apply_wow_flutter(samples, self.wow_flutter_intensity);
        }
        
        // 6. Vinyl crackle on top (final texture)
        if self.vinyl_intensity > 0.0 {
            add_vinyl_crackle(samples, self.vinyl_intensity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vinyl_crackle() {
        let mut samples = vec![0.5; 1000];
        add_vinyl_crackle(&mut samples, 1.0);
        // Should have added noise
        assert!(samples.iter().any(|&s| s != 0.5));
    }
    
    #[test]
    fn test_tape_saturation() {
        let mut samples = vec![0.9; 100];
        apply_tape_saturation(&mut samples, 0.5);
        // Should be compressed/saturated
        assert!(samples[0] < 0.9);
    }
    
    #[test]
    fn test_lofi_processor() {
        let mut samples = vec![0.3; 1000];
        let processor = LofiProcessor::subtle();
        processor.process(&mut samples);
        // Should have been processed
        assert_ne!(samples[0], 0.3);
    }
}

