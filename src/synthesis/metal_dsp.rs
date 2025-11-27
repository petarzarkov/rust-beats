use crate::utils::get_sample_rate;

/// Advanced distortion with tube-style waveshaping and oversampling
/// Based on research: tanh for tube emulation, oversampling to prevent aliasing
#[derive(Debug, Clone)]
pub struct TubeDistortion {
    pub drive: f32,           // Input gain (1.0 - 20.0+)
    pub mix: f32,             // Dry/wet mix (0.0 - 1.0)
    pub output_level: f32,    // Output compensation
    pub asymmetry: f32,       // Asymmetric clipping (0.0 = symmetric, 1.0 = full asymmetry)
    pub oversample_factor: usize, // 1, 2, 4, or 8
}

impl TubeDistortion {
    pub fn new(drive: f32, mix: f32) -> Self {
        TubeDistortion {
            drive: drive.max(1.0),
            mix: mix.clamp(0.0, 1.0),
            output_level: 1.0 / (1.0 + drive * 0.1), // Auto-compensate for gain
            asymmetry: 0.2, // Slight asymmetry by default (like real tubes)
            oversample_factor: 4, // 4x oversampling by default
        }
    }

    /// Create a metal-optimized distortion preset
    pub fn metal() -> Self {
        TubeDistortion {
            drive: 8.0,
            mix: 1.0,
            output_level: 0.6,
            asymmetry: 0.3,
            oversample_factor: 4,
        }
    }

    /// Create a high-gain distortion for extreme metal
    pub fn high_gain() -> Self {
        TubeDistortion {
            drive: 15.0,
            mix: 1.0,
            output_level: 0.5,
            asymmetry: 0.4,
            oversample_factor: 8, // More oversampling for extreme gain
        }
    }

    /// Process a single sample with tube-style waveshaping
    fn waveshape(&self, input: f32) -> f32 {
        let driven = input * self.drive;
        
        // Asymmetric clipping: positive and negative cycles clip differently
        let shaped = if driven >= 0.0 {
            // Positive cycle
            (driven * (1.0 + self.asymmetry)).tanh()
        } else {
            // Negative cycle (less compression)
            (driven * (1.0 - self.asymmetry * 0.5)).tanh()
        };
        
        shaped
    }

    /// Process a sample with oversampling to reduce aliasing
    pub fn process(&self, input: f32) -> f32 {
        if self.oversample_factor == 1 {
            // No oversampling
            let wet = self.waveshape(input);
            return input * (1.0 - self.mix) + wet * self.mix * self.output_level;
        }

        // Upsample: insert zeros between samples
        let mut upsampled = vec![0.0; self.oversample_factor];
        upsampled[0] = input;

        // Apply simple low-pass filter for interpolation
        for i in 1..self.oversample_factor {
            upsampled[i] = upsampled[i - 1] * 0.5;
        }

        // Process each upsampled sample
        let processed: Vec<f32> = upsampled.iter()
            .map(|&sample| self.waveshape(sample))
            .collect();

        // Downsample: simple averaging
        let wet = processed.iter().sum::<f32>() / self.oversample_factor as f32;

        // Mix dry/wet
        input * (1.0 - self.mix) + wet * self.mix * self.output_level
    }

    /// Process a buffer of samples
    pub fn process_buffer(&self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// Noise gate for metal - essential to stop hum between staccato riffs
#[derive(Debug, Clone)]
pub struct NoiseGate {
    pub threshold: f32,       // Amplitude threshold (0.0 - 1.0)
    pub ratio: f32,           // Attenuation ratio (0.0 = full cut, 1.0 = no cut)
    pub attack: f32,          // Attack time in seconds
    pub release: f32,         // Release time in seconds
    envelope: f32,            // Current envelope value
}

impl NoiseGate {
    pub fn new(threshold: f32) -> Self {
        NoiseGate {
            threshold: threshold.clamp(0.0, 1.0),
            ratio: 0.0, // Full cut by default
            attack: 0.001, // 1ms attack
            release: 0.05, // 50ms release
            envelope: 0.0,
        }
    }

    /// Metal preset: aggressive gating
    pub fn metal() -> Self {
        NoiseGate {
            threshold: 0.02,
            ratio: 0.0,
            attack: 0.0005,
            release: 0.03,
            envelope: 0.0,
        }
    }

    /// Process a sample
    pub fn process(&mut self, input: f32) -> f32 {
        let sample_rate = get_sample_rate() as f32;
        let input_level = input.abs();

        // Calculate envelope coefficients
        let attack_coeff = (-1.0 / (self.attack * sample_rate)).exp();
        let release_coeff = (-1.0 / (self.release * sample_rate)).exp();

        // Update envelope
        if input_level > self.envelope {
            // Attack
            self.envelope = attack_coeff * self.envelope + (1.0 - attack_coeff) * input_level;
        } else {
            // Release
            self.envelope = release_coeff * self.envelope + (1.0 - release_coeff) * input_level;
        }

        // Apply gate
        if self.envelope > self.threshold {
            input // Gate open
        } else {
            input * self.ratio // Gate closed (attenuate)
        }
    }

    /// Process a buffer
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

// ============================================================================
// EQ and Filter Components
// ============================================================================

/// Simple high-pass filter (1-pole)
#[derive(Debug, Clone)]
struct SimpleHighPass {
    prev_input: f32,
    prev_output: f32,
    alpha: f32,
}

impl SimpleHighPass {
    fn new(cutoff_hz: f32) -> Self {
        let sample_rate = get_sample_rate() as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);

        SimpleHighPass {
            prev_input: 0.0,
            prev_output: 0.0,
            alpha,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.alpha * (self.prev_output + input - self.prev_input);
        self.prev_input = input;
        self.prev_output = output;
        output
    }
}

/// Simple low-pass filter (1-pole)
#[derive(Debug, Clone)]
struct SimpleLowPass {
    prev_output: f32,
    alpha: f32,
}

impl SimpleLowPass {
    fn new(cutoff_hz: f32) -> Self {
        let sample_rate = get_sample_rate() as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);

        SimpleLowPass {
            prev_output: 0.0,
            alpha,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.prev_output + self.alpha * (input - self.prev_output);
        self.prev_output = output;
        output
    }
}

/// Simple peaking EQ (boost/cut at frequency)
#[derive(Debug, Clone)]
struct SimplePeaking {
    gain: f32,
}

impl SimplePeaking {
    fn new(_freq_hz: f32, gain: f32) -> Self {
        // Simplified: just apply gain (proper implementation would use biquad)
        SimplePeaking { gain }
    }

    fn process(&self, input: f32) -> f32 {
        input * self.gain
    }
}

/// Simple high-shelf filter
#[derive(Debug, Clone)]
struct SimpleHighShelf {
    gain: f32,
}

impl SimpleHighShelf {
    fn new(_freq_hz: f32, gain: f32) -> Self {
        // Simplified: just apply gain (proper implementation would use biquad)
        SimpleHighShelf { gain }
    }

    fn process(&self, input: f32) -> f32 {
        input * self.gain
    }
}

/// Pre-gain EQ for shaping tone before distortion
/// Boosts mids, cuts mud
#[derive(Debug, Clone)]
pub struct PreGainEQ {
    high_pass: SimpleHighPass,
    mid_boost: SimplePeaking,
}

impl PreGainEQ {
    pub fn new() -> Self {
        PreGainEQ {
            high_pass: SimpleHighPass::new(80.0), // Remove mud
            mid_boost: SimplePeaking::new(1400.0, 1.5), // Djent "quack"
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let sample = self.high_pass.process(sample);
        self.mid_boost.process(sample)
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// Post-distortion EQ for presence and air
#[derive(Debug, Clone)]
pub struct PostDistortionEQ {
    presence: SimplePeaking,
    air: SimpleHighShelf,
    low_pass: SimpleLowPass,
}

impl PostDistortionEQ {
    pub fn new() -> Self {
        PostDistortionEQ {
            presence: SimplePeaking::new(4000.0, 1.3), // Clarity
            air: SimpleHighShelf::new(8000.0, 1.2),     // Brightness
            low_pass: SimpleLowPass::new(12000.0),      // Smooth harshness
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let sample = self.presence.process(sample);
        let sample = self.air.process(sample);
        self.low_pass.process(sample)
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

// ============================================================================
// Unified Metal DSP Chain
// ============================================================================

/// Complete metal DSP chain combining all processing stages
/// Signal flow: Noise Gate → Pre-EQ → Distortion (with oversampling) → Post-EQ
#[derive(Debug, Clone)]
pub struct MetalDSPChain {
    noise_gate: NoiseGate,
    pre_eq: PreGainEQ,
    distortion: TubeDistortion,
    post_eq: PostDistortionEQ,
}

impl MetalDSPChain {
    /// Create a new metal DSP chain with specified drive
    pub fn new(drive: f32) -> Self {
        MetalDSPChain {
            noise_gate: NoiseGate::metal(),
            pre_eq: PreGainEQ::new(),
            distortion: TubeDistortion::new(drive, 1.0),
            post_eq: PostDistortionEQ::new(),
        }
    }

    /// Metal preset: balanced heavy tone
    pub fn metal() -> Self {
        MetalDSPChain {
            noise_gate: NoiseGate::metal(),
            pre_eq: PreGainEQ::new(),
            distortion: TubeDistortion::metal(),
            post_eq: PostDistortionEQ::new(),
        }
    }

    /// High-gain preset: extreme distortion
    pub fn high_gain() -> Self {
        MetalDSPChain {
            noise_gate: NoiseGate::metal(),
            pre_eq: PreGainEQ::new(),
            distortion: TubeDistortion::high_gain(),
            post_eq: PostDistortionEQ::new(),
        }
    }

    /// Process a single sample through the complete DSP chain
    pub fn process(&mut self, sample: f32) -> f32 {
        let sample = self.noise_gate.process(sample);
        let sample = self.pre_eq.process(sample);
        let sample = self.distortion.process(sample);
        self.post_eq.process(sample)
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
    fn test_tube_distortion_basic() {
        let dist = TubeDistortion::new(5.0, 1.0);
        let input = 0.5;
        let output = dist.process(input);
        
        // Output should be distorted (not equal to input)
        assert_ne!(output, input);
        // Output should be within valid range
        assert!(output.abs() <= 1.0);
    }

    #[test]
    fn test_tube_distortion_metal_preset() {
        let dist = TubeDistortion::metal();
        assert_eq!(dist.drive, 8.0);
        assert_eq!(dist.oversample_factor, 4);
    }

    #[test]
    fn test_noise_gate() {
        let mut gate = NoiseGate::new(0.1);
        
        // Process several quiet samples - should be attenuated
        for _ in 0..10 {
            let quiet = gate.process(0.05);
            assert!(quiet.abs() < 0.05);
        }
        
        // Process several loud samples - envelope should build up and pass through
        let mut last_loud = 0.0;
        for _ in 0..100 {
            last_loud = gate.process(0.5);
        }
        assert!(last_loud.abs() > 0.1, "Gate should open for loud signal after envelope builds up");
    }

    #[test]
    fn test_asymmetric_clipping() {
        let dist = TubeDistortion {
            drive: 10.0,
            mix: 1.0,
            output_level: 1.0,
            asymmetry: 0.5,
            oversample_factor: 1,
        };

        let positive = dist.waveshape(0.5);
        let negative = dist.waveshape(-0.5);
        
        // Asymmetry means positive and negative should clip differently
        assert_ne!(positive.abs(), negative.abs());
    }

    #[test]
    fn test_metal_dsp_chain() {
        let mut chain = MetalDSPChain::metal();
        let mut buffer = vec![0.5; 100];
        
        chain.process_buffer(&mut buffer);
        
        // All samples should be processed and within valid range
        assert!(buffer.iter().all(|&s| s.abs() < 1.0));
    }

    #[test]
    fn test_dsp_chain_presets() {
        let metal = MetalDSPChain::metal();
        let high_gain = MetalDSPChain::high_gain();
        
        // High gain should have higher drive
        assert!(high_gain.distortion.drive > metal.distortion.drive);
    }
}

