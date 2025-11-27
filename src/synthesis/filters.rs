/// Simple low-pass filter for damping
#[derive(Debug, Clone)]
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

    pub fn process(&mut self, input: f32) -> f32 {
        let resonance_adjust = (1.0 + self.resonance).max(0.1);
        let alpha = (self.cutoff / (self.cutoff + resonance_adjust)).clamp(0.0, 1.0);
        self.prev_output = alpha * input + (1.0 - alpha) * self.prev_output;
        self.prev_output
    }
}
