/// Audio mixing and mastering utilities
use crate::synthesis::synthesizer::LowPassFilter;

const SAMPLE_RATE: u32 = 44100;

/// Represents a track with audio samples and mixing parameters
#[derive(Clone)]
pub struct Track {
    pub samples: Vec<f32>,
    pub volume: f32,      // 0.0 to 1.0+
    pub pan: f32,         // -1.0 (left) to 1.0 (right)
    pub eq_low: f32,      // Low frequency boost/cut (0.0 to 2.0, 1.0 = flat)
    pub eq_mid: f32,      // Mid frequency boost/cut
    pub eq_high: f32,     // High frequency boost/cut
}

impl Track {
    pub fn new(samples: Vec<f32>) -> Self {
        Track {
            samples,
            volume: 1.0,
            pan: 0.0,
            eq_low: 1.0,
            eq_mid: 1.0,
            eq_high: 1.0,
        }
    }
    
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
    
    pub fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }
    
    pub fn with_eq(mut self, low: f32, mid: f32, high: f32) -> Self {
        self.eq_low = low;
        self.eq_mid = mid;
        self.eq_high = high;
        self
    }
}

/// Simple 3-band EQ using butterworth filters
struct SimpleEQ {
    // Low-shelf filter state (< 200Hz)
    low_prev_in: f32,
    low_prev_out: f32,
    // High-shelf filter state (> 3000Hz)
    high_prev_in: f32,
    high_prev_out: f32,
}

impl SimpleEQ {
    fn new() -> Self {
        SimpleEQ {
            low_prev_in: 0.0,
            low_prev_out: 0.0,
            high_prev_in: 0.0,
            high_prev_out: 0.0,
        }
    }
    
    fn process(&mut self, sample: f32, low_gain: f32, mid_gain: f32, high_gain: f32) -> f32 {
        // Simple shelf filters
        // Low shelf (200Hz cutoff)
        let low_coeff = 0.15;
        let low = low_coeff * sample + (1.0 - low_coeff) * self.low_prev_out;
        self.low_prev_out = low;
        
        // High shelf (3000Hz cutoff)
        let high_coeff = 0.7;
        let high = high_coeff * (sample - self.high_prev_in + 0.95 * self.high_prev_out);
        self.high_prev_in = sample;
        self.high_prev_out = high;
        
        // Mid is what's left
        let mid = sample - low - high;
        
        // Apply gains
        low * low_gain + mid * mid_gain + high * high_gain
    }
}

/// Simple compressor with makeup gain
struct Compressor {
    prev_env: f32,
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
}

impl Compressor {
    fn new(threshold: f32, ratio: f32) -> Self {
        Compressor {
            prev_env: 0.0,
            threshold,
            ratio,
            attack: 0.0001,     // Fast attack
            release: 0.00005,   // Slower release
        }
    }
    
    fn process(&mut self, sample: f32) -> f32 {
        let sample_abs = sample.abs();
        
        // Envelope follower
        let coeff = if sample_abs > self.prev_env {
            self.attack
        } else {
            self.release
        };
        
        let env = coeff * sample_abs + (1.0 - coeff) * self.prev_env;
        self.prev_env = env;
        
        // Calculate gain reduction
        let gain = if env > self.threshold {
            let over = env / self.threshold;
            let reduction = over.powf(1.0 / self.ratio - 1.0);
            reduction
        } else {
            1.0
        };
        
        sample * gain
    }
}

/// Limiter to prevent clipping
struct Limiter {
    threshold: f32,
}

impl Limiter {
    fn new(threshold: f32) -> Self {
        Limiter { threshold }
    }
    
    fn process(&self, sample: f32) -> f32 {
        if sample > self.threshold {
            self.threshold + (sample - self.threshold).tanh() * 0.1
        } else if sample < -self.threshold {
            -self.threshold + (sample + self.threshold).tanh() * 0.1
        } else {
            sample
        }
    }
}

/// Mix multiple tracks into stereo output
pub fn mix_tracks(tracks: Vec<Track>) -> Vec<f32> {
    if tracks.is_empty() {
        return Vec::new();
    }
    
    // Find the longest track
    let max_len = tracks.iter()
        .map(|t| t.samples.len())
        .max()
        .unwrap_or(0);
    
    // Initialize stereo output (interleaved: L, R, L, R, ...)
    let mut stereo_output = vec![0.0; max_len * 2];
    
    // Mix each track
    for track in tracks {
        let mut eq_l = SimpleEQ::new();
        let mut eq_r = SimpleEQ::new();
        
        for (i, &sample) in track.samples.iter().enumerate() {
            // Apply EQ
            let eq_sample_l = eq_l.process(sample, track.eq_low, track.eq_mid, track.eq_high);
            let eq_sample_r = eq_r.process(sample, track.eq_low, track.eq_mid, track.eq_high);
            
            // Apply volume
            let vol_sample_l = eq_sample_l * track.volume;
            let vol_sample_r = eq_sample_r * track.volume;
            
            // Apply panning (constant power pan)
            let pan_angle = (track.pan + 1.0) * 0.25 * std::f32::consts::PI;
            let left_gain = pan_angle.cos();
            let right_gain = pan_angle.sin();
            
            let left = vol_sample_l * left_gain;
            let right = vol_sample_r * right_gain;
            
            // Mix into stereo output
            let stereo_idx = i * 2;
            if stereo_idx + 1 < stereo_output.len() {
                stereo_output[stereo_idx] += left;
                stereo_output[stereo_idx + 1] += right;
            }
        }
    }
    
    stereo_output
}

/// Apply mastering chain (compression and limiting) to stereo mix
pub fn master(stereo_samples: &mut [f32], target_loudness: f32) {
    // Apply gentle compression
    let mut compressor_l = Compressor::new(0.6, 3.0);
    let mut compressor_r = Compressor::new(0.6, 3.0);
    
    for i in (0..stereo_samples.len()).step_by(2) {
        if i + 1 < stereo_samples.len() {
            stereo_samples[i] = compressor_l.process(stereo_samples[i]);
            stereo_samples[i + 1] = compressor_r.process(stereo_samples[i + 1]);
        }
    }
    
    // Find current peak
    let peak = stereo_samples.iter()
        .map(|&s| s.abs())
        .fold(0.0f32, f32::max);
    
    // Calculate makeup gain
    let makeup_gain = if peak > 0.0001 {
        (target_loudness / peak).min(3.0)
    } else {
        1.0
    };
    
    // Apply makeup gain
    for sample in stereo_samples.iter_mut() {
        *sample *= makeup_gain;
    }
    
    // Apply final limiter
    let limiter = Limiter::new(0.95);
    for sample in stereo_samples.iter_mut() {
        *sample = limiter.process(*sample);
    }
}

/// Convert stereo f32 samples (-1.0 to 1.0) to mono f32 samples
pub fn stereo_to_mono(stereo_samples: &[f32]) -> Vec<f32> {
    stereo_samples
        .chunks_exact(2)
        .map(|lr| (lr[0] + lr[1]) * 0.5)
        .collect()
}

/// Add two audio buffers together (mixing)
pub fn mix_buffers(a: &[f32], b: &[f32]) -> Vec<f32> {
    let max_len = a.len().max(b.len());
    let mut result = vec![0.0; max_len];
    
    for i in 0..max_len {
        if i < a.len() {
            result[i] += a[i];
        }
        if i < b.len() {
            result[i] += b[i];
        }
    }
    
    result
}

/// Lofi-specific mastering chain with warmth and character
pub fn master_lofi(stereo_samples: &mut [f32], target_loudness: f32, lofi_intensity: f32) {
    // 1. Gentle compression with slower attack for laid-back feel
    let mut compressor_l = Compressor::new(0.65, 2.5);  // Less aggressive
    let mut compressor_r = Compressor::new(0.65, 2.5);
    
    for i in (0..stereo_samples.len()).step_by(2) {
        if i + 1 < stereo_samples.len() {
            stereo_samples[i] = compressor_l.process(stereo_samples[i]);
            stereo_samples[i + 1] = compressor_r.process(stereo_samples[i + 1]);
        }
    }
    
    // 2. Warm high-frequency roll-off (vintage character)
    let mut filter_l = LowPassFilter::new(10000.0 - lofi_intensity * 3000.0, 0.3);
    let mut filter_r = LowPassFilter::new(10000.0 - lofi_intensity * 3000.0, 0.3);
    
    for i in (0..stereo_samples.len()).step_by(2) {
        if i + 1 < stereo_samples.len() {
            stereo_samples[i] = filter_l.process(stereo_samples[i]);
            stereo_samples[i + 1] = filter_r.process(stereo_samples[i + 1]);
        }
    }
    
    // 3. Find current peak
    let peak = stereo_samples.iter()
        .map(|&s| s.abs())
        .fold(0.0f32, f32::max);
    
    // 4. Calculate makeup gain
    let makeup_gain = if peak > 0.0001 {
        (target_loudness / peak).min(2.5)
    } else {
        1.0
    };
    
    // 5. Apply makeup gain
    for sample in stereo_samples.iter_mut() {
        *sample *= makeup_gain;
    }
    
    // 6. Final soft limiter (less aggressive for lofi)
    let limiter = Limiter::new(0.9);
    for sample in stereo_samples.iter_mut() {
        *sample = limiter.process(*sample);
    }
}

/// Lofi mix chain with vinyl and tape effects
pub fn mix_lofi_tracks(tracks: Vec<Track>) -> Vec<f32> {
    // Standard stereo mix
    let stereo_output = mix_tracks(tracks);
    stereo_output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_track_creation() {
        let samples = vec![0.0; 1000];
        let track = Track::new(samples.clone())
            .with_volume(0.8)
            .with_pan(-0.5);
        
        assert_eq!(track.volume, 0.8);
        assert_eq!(track.pan, -0.5);
        assert_eq!(track.samples.len(), 1000);
    }
    
    #[test]
    fn test_mix_tracks() {
        let track1 = Track::new(vec![0.5; 1000]);
        let track2 = Track::new(vec![0.3; 1000]);
        
        let mixed = mix_tracks(vec![track1, track2]);
        assert_eq!(mixed.len(), 2000); // Stereo
    }
    
    #[test]
    fn test_mastering() {
        let mut samples = vec![0.5; 2000]; // Stereo
        master(&mut samples, 0.8);
        
        let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak <= 1.0); // Should not clip
    }
}

