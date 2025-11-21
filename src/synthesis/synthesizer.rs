use std::f32::consts::PI;
use std::sync::OnceLock;

static SAMPLE_RATE_STORAGE: OnceLock<u32> = OnceLock::new();

/// Initialize the sample rate from config (must be called before any synthesis)
pub fn init_sample_rate(sample_rate: u32) {
    SAMPLE_RATE_STORAGE
        .set(sample_rate)
        .expect("Sample rate already initialized");
}

/// Get the current sample rate
pub fn get_sample_rate() -> u32 {
    *SAMPLE_RATE_STORAGE.get().unwrap_or(&44100) // Fallback to 44100 if not initialized
}

/// Oscillator waveform types
#[derive(Debug, Clone, Copy)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
    Noise,
}

/// ADSR Envelope generator
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    pub attack: f32,  // seconds
    pub decay: f32,   // seconds
    pub sustain: f32, // level 0.0-1.0
    pub release: f32, // seconds
}

impl Envelope {
    /// Get the envelope value at a specific time
    pub fn get_amplitude(&self, time: f32, note_off_time: Option<f32>) -> f32 {
        if let Some(off_time) = note_off_time {
            if time >= off_time {
                // Release phase
                let release_time = time - off_time;
                if release_time >= self.release {
                    return 0.0;
                }
                let sustain_level = self.get_amplitude(off_time, None);
                return sustain_level * (1.0 - (release_time / self.release));
            }
        }

        if time < self.attack {
            // Attack phase
            time / self.attack
        } else if time < self.attack + self.decay {
            // Decay phase
            let decay_time = time - self.attack;
            1.0 - ((1.0 - self.sustain) * (decay_time / self.decay))
        } else {
            // Sustain phase
            self.sustain
        }
    }

    /// Envelope for leads
    pub fn lead() -> Self {
        Envelope {
            attack: 0.05,
            decay: 0.2,
            sustain: 0.8,
            release: 0.2,
        }
    }

    /// Envelope for drums/percussion (fast attack, quick decay)
    pub fn drum() -> Self {
        Envelope {
            attack: 0.001,
            decay: 0.05,
            sustain: 0.0,
            release: 0.1,
        }
    }
}

/// Oscillator that generates waveforms
pub struct Oscillator {
    pub waveform: Waveform,
    pub frequency: f32,
    pub phase: f32,
    sample_rate: u32,
}

impl Oscillator {
    pub fn new(waveform: Waveform, frequency: f32) -> Self {
        Oscillator {
            waveform,
            frequency,
            phase: 0.0,
            sample_rate: get_sample_rate(),
        }
    }

    /// Generate n samples
    pub fn generate(&mut self, n: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(n);
        for _ in 0..n {
            samples.push(self.next_sample());
        }
        samples
    }

    /// Generate the next sample
    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
            Waveform::Sine => (self.phase * 2.0 * PI).sin(),
            Waveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Saw => 2.0 * self.phase - 1.0,
            Waveform::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            Waveform::Noise => rand::random::<f32>() * 2.0 - 1.0,
        };

        // Advance phase
        self.phase += self.frequency / self.sample_rate as f32;
        self.phase = self.phase.fract();

        sample
    }

}



/// Simple low-pass filter
pub struct LowPassFilter {
    pub cutoff: f32,
    pub resonance: f32,
    prev_output: f32,
}

impl LowPassFilter {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        LowPassFilter {
            cutoff,
            resonance,
            prev_output: 0.0,
        }
    }

    /// Apply filter to a sample
    pub fn process(&mut self, input: f32) -> f32 {
        // Simple one-pole lowpass
        let resonance_adjust = (1.0 + self.resonance).max(0.1);
        let alpha = (self.cutoff / (self.cutoff + resonance_adjust)).clamp(0.0, 1.0);
        self.prev_output = alpha * input + (1.0 - alpha) * self.prev_output;
        self.prev_output
    }

    /// Apply filter to buffer
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// Resonant 2-pole filter for dubstep wobble bass
/// Uses a simple biquad-style filter with resonance
pub struct ResonantFilter {
    pub cutoff: f32,
    pub resonance: f32, // 0.0 to 1.0, higher = more peak/boost
    sample_rate: f32,
    // Two-pole filter state
    x1: f32, // Previous input
    x2: f32, // Input two samples ago
    y1: f32, // Previous output
    y2: f32, // Output two samples ago
}

impl ResonantFilter {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        ResonantFilter {
            cutoff,
            resonance: resonance.clamp(0.0, 0.99),
            sample_rate: get_sample_rate() as f32,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Apply resonant filter to a sample
    /// Returns filtered output with resonance peak
    /// Uses a simple 2-pole biquad filter
    pub fn process(&mut self, input: f32) -> f32 {
        let sample_rate = self.sample_rate;
        let cutoff = self.cutoff.clamp(20.0, sample_rate * 0.45);
        let resonance = self.resonance.clamp(0.0, 0.99);
        
        // Calculate filter coefficients for 2-pole low-pass with resonance
        let w = 2.0 * PI * cutoff / sample_rate;
        let cos_w = w.cos();
        let sin_w = w.sin();
        
        // Q factor from resonance
        let q = 0.5 + resonance * 4.5; // Q from 0.5 to 5.0
        let alpha = sin_w / (2.0 * q);
        
        // Biquad coefficients
        let b0 = (1.0 - cos_w) / 2.0;
        let b1 = 1.0 - cos_w;
        let b2 = (1.0 - cos_w) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;
        
        // Normalize coefficients
        let b0_norm = b0 / a0;
        let b1_norm = b1 / a0;
        let b2_norm = b2 / a0;
        let a1_norm = a1 / a0;
        let a2_norm = a2 / a0;
        
        // Apply filter
        let output = b0_norm * input 
                   + b1_norm * self.x1 
                   + b2_norm * self.x2
                   - a1_norm * self.y1
                   - a2_norm * self.y2;
        
        // Update state
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        
        // Apply resonance boost
        let resonance_boost = 1.0 + resonance * 1.5;
        output * resonance_boost
    }

    /// Apply filter to buffer
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

/// LFO (Low Frequency Oscillator) for modulation
pub struct LFO {
    oscillator: Oscillator,
    pub depth: f32,
}

impl LFO {
    pub fn new(frequency: f32, depth: f32) -> Self {
        LFO {
            oscillator: Oscillator::new(Waveform::Sine, frequency),
            depth,
        }
    }

    /// Get the current modulation value (0.0 to 1.0)
    pub fn next_value(&mut self) -> f32 {
        (self.oscillator.next_sample() + 1.0) * 0.5 * self.depth
    }
}

/// Generate samples for a note with envelope
pub fn generate_note(
    waveform: Waveform,
    frequency: f32,
    duration: f32,
    envelope: &Envelope,
    amplitude: f32,
) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut oscillator = Oscillator::new(waveform, frequency);
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(duration * 0.8));
        let sample = oscillator.next_sample() * env_amp * amplitude;
        samples.push(sample);
    }

    samples
}

/// Mix multiple audio buffers together
pub fn mix_buffers(buffers: &[Vec<f32>]) -> Vec<f32> {
    if buffers.is_empty() {
        return Vec::new();
    }

    let max_len = buffers.iter().map(|b| b.len()).max().unwrap();
    let mut mixed = vec![0.0; max_len];

    for buffer in buffers {
        for (i, &sample) in buffer.iter().enumerate() {
            mixed[i] += sample;
        }
    }

    // Normalize to prevent clipping
    let max_amplitude = mixed.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if max_amplitude > 1.0 {
        for sample in &mut mixed {
            *sample /= max_amplitude;
        }
    }

    mixed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator() {
        let mut osc = Oscillator::new(Waveform::Sine, 440.0);
        let samples = osc.generate(100);
        assert_eq!(samples.len(), 100);
    }

    #[test]
    fn test_envelope() {
        let env = Envelope::drum();
        assert!(env.get_amplitude(0.0, None) < 0.1);
        assert!(env.get_amplitude(0.001, None) > 0.9);
    }

    #[test]
    fn test_note_generation() {
        let samples = generate_note(Waveform::Sine, 440.0, 0.5, &Envelope::drum(), 0.5);
        assert!(!samples.is_empty());
    }
}
