use crate::utils::get_sample_rate;

/// Professional mixing effects for metal music
/// Includes reverb, EQ, and compression

/// Simple reverb effect using comb filters and all-pass filters
#[derive(Debug, Clone)]
pub struct Reverb {
    sample_rate: f32,
    // Comb filter delays (in samples)
    comb_delays: Vec<usize>,
    comb_buffers: Vec<Vec<f32>>,
    comb_indices: Vec<usize>,
    // All-pass filter delays
    allpass_delays: Vec<usize>,
    allpass_buffers: Vec<Vec<f32>>,
    allpass_indices: Vec<usize>,
    // Parameters
    decay: f32,
    wet_dry_mix: f32,
}

impl Reverb {
    /// Create a new reverb effect
    pub fn new(room_size: f32, decay: f32, wet_dry_mix: f32) -> Self {
        let sample_rate = get_sample_rate();
        
        // Comb filter delays (prime numbers for natural sound)
        let base_delays = vec![1557, 1617, 1491, 1422, 1277, 1356, 1188, 1116];
        let comb_delays: Vec<usize> = base_delays.iter()
            .map(|&d| ((d as f32) * room_size) as usize)
            .collect();
        
        let comb_buffers: Vec<Vec<f32>> = comb_delays.iter()
            .map(|&size| vec![0.0; size])
            .collect();
        
        // All-pass filter delays
        let allpass_base = vec![225, 556, 441, 341];
        let allpass_delays: Vec<usize> = allpass_base.iter()
            .map(|&d| ((d as f32) * room_size) as usize)
            .collect();
        
        let allpass_buffers: Vec<Vec<f32>> = allpass_delays.iter()
            .map(|&size| vec![0.0; size])
            .collect();
        
        Reverb {
            sample_rate: sample_rate as f32,
            comb_delays,
            comb_buffers,
            comb_indices: vec![0; 8],
            allpass_delays,
            allpass_buffers,
            allpass_indices: vec![0; 4],
            decay,
            wet_dry_mix,
        }
    }
    
    /// Metal reverb preset (short, tight reverb)
    pub fn metal() -> Self {
        Reverb::new(0.6, 0.3, 0.15) // Small room, short decay, subtle mix
    }
    
    /// Hall reverb preset (larger space)
    pub fn hall() -> Self {
        Reverb::new(1.2, 0.6, 0.25) // Large room, longer decay
    }
    
    /// Process a single sample
    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = 0.0;
        
        // Comb filters (parallel)
        for i in 0..self.comb_buffers.len() {
            let delay_idx = self.comb_indices[i];
            let delayed = self.comb_buffers[i][delay_idx];
            
            // Feedback
            self.comb_buffers[i][delay_idx] = input + delayed * self.decay;
            
            output += delayed;
            
            // Increment index
            self.comb_indices[i] = (delay_idx + 1) % self.comb_delays[i];
        }
        
        output /= self.comb_buffers.len() as f32;
        
        // All-pass filters (series)
        for i in 0..self.allpass_buffers.len() {
            let delay_idx = self.allpass_indices[i];
            let delayed = self.allpass_buffers[i][delay_idx];
            
            let temp = output - delayed * 0.5;
            self.allpass_buffers[i][delay_idx] = temp;
            output = delayed + temp * 0.5;
            
            self.allpass_indices[i] = (delay_idx + 1) % self.allpass_delays[i];
        }
        
        // Wet/dry mix
        input * (1.0 - self.wet_dry_mix) + output * self.wet_dry_mix
    }
    
    /// Process a buffer of samples
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// 3-band parametric EQ for metal mixing
#[derive(Debug, Clone)]
pub struct ParametricEQ {
    sample_rate: f32,
    // Low shelf
    low_freq: f32,
    low_gain: f32,
    // Mid peak
    mid_freq: f32,
    mid_gain: f32,
    mid_q: f32,
    // High shelf
    high_freq: f32,
    high_gain: f32,
}

impl ParametricEQ {
    /// Create a new parametric EQ
    pub fn new(
        low_freq: f32, low_gain: f32,
        mid_freq: f32, mid_gain: f32, mid_q: f32,
        high_freq: f32, high_gain: f32
    ) -> Self {
        ParametricEQ {
            sample_rate: get_sample_rate() as f32,
            low_freq,
            low_gain,
            mid_freq,
            mid_gain,
            mid_q,
            high_freq,
            high_gain,
        }
    }
    
    /// Metal EQ preset (scooped mids, boosted lows and highs)
    pub fn metal() -> Self {
        ParametricEQ::new(
            100.0, 1.3,    // Boost lows at 100Hz
            800.0, 0.7, 1.0, // Cut mids at 800Hz (scooped sound)
            4000.0, 1.4    // Boost highs at 4kHz (presence)
        )
    }
    
    /// Modern metal EQ (tighter low end)
    pub fn modern_metal() -> Self {
        ParametricEQ::new(
            80.0, 1.2,     // Moderate low boost
            600.0, 0.6, 1.5, // Aggressive mid scoop
            6000.0, 1.5    // Strong high boost (clarity)
        )
    }
    
    /// Process a single sample (simplified - applies gain directly)
    pub fn process(&self, input: f32) -> f32 {
        // Simplified EQ: just apply frequency-dependent gain
        // In a real implementation, this would use biquad filters
        // For now, we'll just apply the gains as a rough approximation
        input * ((self.low_gain + self.mid_gain + self.high_gain) / 3.0)
    }
    
    /// Process a buffer of samples
    pub fn process_buffer(&self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// Dynamic range compressor for metal mixing
#[derive(Debug, Clone)]
pub struct Compressor {
    sample_rate: f32,
    threshold: f32,      // dB
    ratio: f32,          // e.g., 4.0 for 4:1
    attack_time: f32,    // seconds
    release_time: f32,   // seconds
    makeup_gain: f32,    // dB
    envelope: f32,
}

impl Compressor {
    /// Create a new compressor
    pub fn new(threshold: f32, ratio: f32, attack_ms: f32, release_ms: f32, makeup_gain: f32) -> Self {
        Compressor {
            sample_rate: get_sample_rate() as f32,
            threshold,
            ratio,
            attack_time: attack_ms / 1000.0,
            release_time: release_ms / 1000.0,
            makeup_gain,
            envelope: 0.0,
        }
    }
    
    /// Metal compressor preset (aggressive, punchy)
    pub fn metal() -> Self {
        Compressor::new(
            -12.0,  // Threshold (dB)
            4.0,    // Ratio (4:1)
            5.0,    // Attack (5ms - fast)
            50.0,   // Release (50ms - moderate)
            3.0     // Makeup gain (3dB)
        )
    }
    
    /// Mastering compressor (gentle, transparent)
    pub fn mastering() -> Self {
        Compressor::new(
            -6.0,   // Threshold
            2.0,    // Ratio (2:1 - gentle)
            10.0,   // Attack (10ms)
            100.0,  // Release (100ms)
            2.0     // Makeup gain
        )
    }
    
    /// Process a single sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Convert to dB
        let input_db = 20.0 * (input.abs() + 1e-10).log10();
        
        // Calculate gain reduction
        let over_threshold = input_db - self.threshold;
        let gain_reduction = if over_threshold > 0.0 {
            over_threshold * (1.0 - 1.0 / self.ratio)
        } else {
            0.0
        };
        
        // Envelope follower
        let attack_coef = (-1.0 / (self.attack_time * self.sample_rate)).exp();
        let release_coef = (-1.0 / (self.release_time * self.sample_rate)).exp();
        
        if gain_reduction > self.envelope {
            self.envelope = attack_coef * self.envelope + (1.0 - attack_coef) * gain_reduction;
        } else {
            self.envelope = release_coef * self.envelope + (1.0 - release_coef) * gain_reduction;
        }
        
        // Apply gain reduction and makeup gain
        let total_gain = -self.envelope + self.makeup_gain;
        let gain_linear = 10.0_f32.powf(total_gain / 20.0);
        
        input * gain_linear
    }
    
    /// Process a buffer of samples
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverb_creation() {
        let reverb = Reverb::metal();
        assert!(reverb.wet_dry_mix > 0.0 && reverb.wet_dry_mix < 1.0);
        assert!(reverb.decay > 0.0 && reverb.decay < 1.0);
    }

    #[test]
    fn test_reverb_process() {
        let mut reverb = Reverb::metal();
        // Use a larger buffer with an impulse to test reverb tail
        let mut output = vec![0.0; 10000];
        output[0] = 1.0; // Impulse at start
        
        reverb.process_buffer(&mut output);
        
        // Reverb should create a tail - check samples after the impulse
        let tail_energy: f32 = output.iter().skip(100).take(1000).map(|s| s.abs()).sum();
        assert!(tail_energy > 0.0, "Reverb should create a tail");
    }

    #[test]
    fn test_eq_creation() {
        let eq = ParametricEQ::metal();
        assert!(eq.low_gain > 1.0); // Boosted lows
        assert!(eq.mid_gain < 1.0); // Cut mids
        assert!(eq.high_gain > 1.0); // Boosted highs
    }

    #[test]
    fn test_eq_process() {
        let eq = ParametricEQ::metal();
        let input = vec![0.5; 100];
        let mut output = input.clone();
        
        eq.process_buffer(&mut output);
        
        // EQ should modify the signal
        assert!(output.iter().any(|&s| (s - 0.5).abs() > 0.01));
    }

    #[test]
    fn test_compressor_creation() {
        let comp = Compressor::metal();
        assert!(comp.threshold < 0.0);
        assert!(comp.ratio > 1.0);
        assert!(comp.makeup_gain > 0.0);
    }

    #[test]
    fn test_compressor_process() {
        let mut comp = Compressor::metal();
        
        // Process several loud samples to let envelope build up
        let loud_samples = vec![0.9; 500]; // More samples for envelope to stabilize
        let mut loud_output = loud_samples.clone();
        comp.process_buffer(&mut loud_output);
        
        // Check the last samples after envelope has built up
        let avg_last_100: f32 = loud_output.iter().skip(400).sum::<f32>() / 100.0;
        
        // With makeup gain, output might be similar or slightly higher, but should show compression
        // The key is that peaks are reduced - check that max is controlled
        let max_output = loud_output.iter().skip(100).map(|&s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_output < 1.0, "Compressor should prevent clipping");
        
        // Envelope should have built up by the end
        assert!(avg_last_100 > 0.0, "Compressor should output signal");
    }

    #[test]
    fn test_reverb_hall() {
        let hall = Reverb::hall();
        let metal = Reverb::metal();
        
        // Hall should have longer decay
        assert!(hall.decay > metal.decay);
        assert!(hall.wet_dry_mix > metal.wet_dry_mix);
    }

    #[test]
    fn test_eq_modern_metal() {
        let modern = ParametricEQ::modern_metal();
        let classic = ParametricEQ::metal();
        
        // Modern should have more aggressive mid scoop
        assert!(modern.mid_gain < classic.mid_gain);
    }

    #[test]
    fn test_compressor_mastering() {
        let mastering = Compressor::mastering();
        let metal = Compressor::metal();
        
        // Mastering should be gentler
        assert!(mastering.ratio < metal.ratio);
        assert!(mastering.attack_time > metal.attack_time);
    }
}
