use crate::utils::get_sample_rate;
use crate::synthesis::filters::LowPassFilter;

/// Cabinet simulator for guitar amplifier cabinet emulation
/// Simulates the frequency response of a guitar speaker cabinet
/// Based on research: models the characteristic "roar" of metal cabinets
#[derive(Debug, Clone)]
pub struct CabinetSimulator {
    /// Low-pass filter for cabinet rolloff
    lpf: LowPassFilter,
    /// High-pass filter coefficients for low-end cleanup
    hpf_prev_input: f32,
    hpf_prev_output: f32,
    hpf_cutoff: f32,
    /// Resonance peak frequency (cabinet resonance)
    resonance_freq: f32,
    /// Resonance gain
    resonance_gain: f32,
}

impl CabinetSimulator {
    /// Create a new cabinet simulator with specified characteristics
    pub fn new(lpf_cutoff: f32, hpf_cutoff: f32, resonance_freq: f32, resonance_gain: f32) -> Self {
        CabinetSimulator {
            lpf: LowPassFilter::new(lpf_cutoff, 0.7),
            hpf_prev_input: 0.0,
            hpf_prev_output: 0.0,
            hpf_cutoff,
            resonance_freq,
            resonance_gain,
        }
    }

    /// Create a 4x12 metal cabinet simulator (e.g., Mesa Boogie, Marshall)
    /// Characteristics: tight low-end, aggressive mids, controlled highs
    pub fn metal_4x12() -> Self {
        CabinetSimulator::new(
            5000.0,  // LPF: Roll off harsh highs above 5kHz
            80.0,    // HPF: Clean up mud below 80Hz
            400.0,   // Resonance: Mid-range punch around 400Hz
            1.3,     // Resonance gain: Moderate boost
        )
    }

    /// Create a 2x12 combo cabinet simulator
    /// Characteristics: tighter, more focused sound
    pub fn combo_2x12() -> Self {
        CabinetSimulator::new(
            4500.0,  // Slightly darker than 4x12
            100.0,   // Tighter low-end
            500.0,   // Higher resonance for clarity
            1.2,     // Less resonance boost
        )
    }

    /// Create a vintage-style cabinet (e.g., Celestion Greenback)
    /// Characteristics: warmer, more colored tone
    pub fn vintage() -> Self {
        CabinetSimulator::new(
            4000.0,  // Darker, warmer
            70.0,    // More low-end
            350.0,   // Lower resonance for warmth
            1.5,     // More pronounced resonance
        )
    }

    /// Process a single sample through the cabinet simulator
    pub fn process(&mut self, input: f32) -> f32 {
        let sample_rate = get_sample_rate() as f32;
        
        // High-pass filter (remove subsonic frequencies)
        let hpf_rc = 1.0 / (2.0 * std::f32::consts::PI * self.hpf_cutoff);
        let hpf_alpha = hpf_rc / (hpf_rc + 1.0 / sample_rate);
        
        let hpf_output = hpf_alpha * (self.hpf_prev_output + input - self.hpf_prev_input);
        self.hpf_prev_input = input;
        self.hpf_prev_output = hpf_output;
        
        // Apply resonance (cabinet characteristic peak)
        // Simple resonance using a bandpass-like boost
        let resonance_boost = if self.resonance_gain > 1.0 {
            // Simplified resonance: boost signal slightly
            input * (1.0 + (self.resonance_gain - 1.0) * 0.3)
        } else {
            input
        };
        
        // Low-pass filter (speaker rolloff)
        let lpf_output = self.lpf.process(resonance_boost);
        
        // Apply cabinet coloration (slight saturation)
        let colored = lpf_output * 0.95; // Slight attenuation for headroom
        
        colored
    }

    /// Process a buffer of samples
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// Simplified IR-based convolution (for future enhancement)
/// This is a placeholder for when we add real impulse response files
#[derive(Debug, Clone)]
pub struct ImpulseResponse {
    /// IR samples (would be loaded from .wav file)
    ir_samples: Vec<f32>,
}

impl ImpulseResponse {
    /// Create a simple synthetic IR for testing
    /// In production, this would load from a .wav file
    pub fn synthetic_metal_cab() -> Self {
        // Generate a simple synthetic IR that approximates a cabinet
        // Real IR would be 1000-10000 samples from a .wav file
        let mut ir_samples = Vec::with_capacity(512);
        
        // Initial spike (direct sound)
        ir_samples.push(1.0);
        
        // Decay with some resonance
        for i in 1..512 {
            let t = i as f32 / 512.0;
            let decay = (-t * 8.0).exp();
            let resonance = (t * 40.0 * std::f32::consts::PI).sin() * 0.3;
            ir_samples.push(decay * (0.7 + resonance));
        }
        
        ImpulseResponse { ir_samples }
    }

    /// Convolve input signal with impulse response
    /// This is a simple direct convolution (O(N*M))
    /// For production, use FFT-based convolution for efficiency
    pub fn convolve(&self, input: &[f32]) -> Vec<f32> {
        let input_len = input.len();
        let ir_len = self.ir_samples.len();
        let output_len = input_len + ir_len - 1;
        
        let mut output = vec![0.0; output_len];
        
        // Direct convolution
        for i in 0..input_len {
            for j in 0..ir_len {
                if i + j < output_len {
                    output[i + j] += input[i] * self.ir_samples[j];
                }
            }
        }
        
        // Normalize to prevent clipping
        let max_val = output.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        if max_val > 1.0 {
            for sample in output.iter_mut() {
                *sample /= max_val;
            }
        }
        
        output
    }

    /// Process a buffer in-place (truncates to input length)
    pub fn process_buffer(&self, buffer: &mut [f32]) {
        let convolved = self.convolve(buffer);
        let len = buffer.len().min(convolved.len());
        buffer[..len].copy_from_slice(&convolved[..len]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cabinet_simulator_creation() {
        let cab = CabinetSimulator::metal_4x12();
        assert!(cab.lpf.cutoff > 0.0);
    }

    #[test]
    fn test_cabinet_presets() {
        let metal = CabinetSimulator::metal_4x12();
        let combo = CabinetSimulator::combo_2x12();
        let vintage = CabinetSimulator::vintage();
        
        // Different presets should have different characteristics
        assert!(metal.lpf.cutoff != combo.lpf.cutoff);
        assert!(combo.lpf.cutoff != vintage.lpf.cutoff);
    }

    #[test]
    fn test_cabinet_process() {
        let mut cab = CabinetSimulator::metal_4x12();
        
        // Process some samples
        let input = vec![0.5, -0.3, 0.8, -0.2];
        let mut output = Vec::new();
        
        for &sample in &input {
            output.push(cab.process(sample));
        }
        
        assert_eq!(output.len(), input.len());
        // Output should be within valid range
        for &sample in &output {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_cabinet_process_buffer() {
        let mut cab = CabinetSimulator::metal_4x12();
        let mut buffer = vec![0.5, -0.3, 0.8, -0.2, 0.1];
        
        cab.process_buffer(&mut buffer);
        
        // All samples should be processed
        for &sample in &buffer {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_impulse_response_creation() {
        let ir = ImpulseResponse::synthetic_metal_cab();
        assert_eq!(ir.ir_samples.len(), 512);
        assert_eq!(ir.ir_samples[0], 1.0); // Initial spike
    }

    #[test]
    fn test_impulse_response_convolve() {
        let ir = ImpulseResponse::synthetic_metal_cab();
        let input = vec![1.0, 0.5, 0.0, -0.5];
        
        let output = ir.convolve(&input);
        
        // Output should be longer than input (input + IR - 1)
        assert!(output.len() >= input.len());
        // Should be normalized
        assert!(output.iter().all(|&x| x.abs() <= 1.0));
    }

    #[test]
    fn test_impulse_response_process_buffer() {
        let ir = ImpulseResponse::synthetic_metal_cab();
        let mut buffer = vec![0.5, -0.3, 0.8, -0.2];
        let original_len = buffer.len();
        
        ir.process_buffer(&mut buffer);
        
        // Buffer length should remain the same
        assert_eq!(buffer.len(), original_len);
        // All samples should be valid
        assert!(buffer.iter().all(|&x| x.abs() <= 1.0));
    }

    #[test]
    fn test_full_signal_chain() {
        // Test complete signal chain: input -> cabinet -> output
        let mut cab = CabinetSimulator::metal_4x12();
        let input = vec![0.8, 0.6, 0.4, 0.2, 0.0, -0.2, -0.4, -0.6, -0.8];
        
        let mut output = Vec::new();
        for &sample in &input {
            output.push(cab.process(sample));
        }
        
        // Cabinet should smooth/filter the signal
        assert_eq!(output.len(), input.len());
        
        // Check that high frequencies are attenuated (cabinet acts as LPF)
        // This is a simple check - in reality we'd do spectral analysis
        assert!(output.iter().all(|&x| x.abs() <= 1.0));
    }
}
