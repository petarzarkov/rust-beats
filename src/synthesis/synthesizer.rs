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
