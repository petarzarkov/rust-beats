/// Aggressive Mix Pipeline for Metal
/// Implements parallel compression, tape saturation, stereo widening, and other aggressive techniques

use std::f32::consts::PI;

/// Parallel compressor - blends compressed and dry signals
pub struct ParallelCompressor {
    threshold: f32,
    ratio: f32,
    wet_mix: f32,
    envelope: f32,
    attack: f32,
    release: f32,
}

impl ParallelCompressor {
    pub fn metal() -> Self {
        ParallelCompressor {
            threshold: -12.0,
            ratio: 4.0,
            wet_mix: 0.4,
            envelope: 0.0,
            attack: 0.001,
            release: 0.1,
        }
    }

    pub fn process(&mut self, input: f32, sample_rate: f32) -> f32 {
        let input_db = 20.0 * input.abs().max(0.0001).log10();
        
        // Envelope follower
        let attack_coef = (-1.0 / (self.attack * sample_rate)).exp();
        let release_coef = (-1.0 / (self.release * sample_rate)).exp();
        
        if input_db > self.envelope {
            self.envelope = attack_coef * self.envelope + (1.0 - attack_coef) * input_db;
        } else {
            self.envelope = release_coef * self.envelope + (1.0 - release_coef) * input_db;
        }
        
        // Compression
        let gain_reduction = if self.envelope > self.threshold {
            (self.envelope - self.threshold) * (1.0 - 1.0 / self.ratio)
        } else {
            0.0
        };
        
        let compressed = input * 10.0_f32.powf(-gain_reduction / 20.0);
        
        // Parallel blend
        input * (1.0 - self.wet_mix) + compressed * self.wet_mix
    }
}

/// Tape saturation simulator
pub struct TapeSaturation {
    drive: f32,
    warmth: f32,
}

impl TapeSaturation {
    pub fn light() -> Self {
        TapeSaturation {
            drive: 1.5,
            warmth: 0.3,
        }
    }

    pub fn process(&self, input: f32) -> f32 {
        let driven = input * self.drive;
        
        // Soft clipping with warmth
        let saturated = if driven.abs() < 1.0 {
            driven
        } else {
            driven.signum() * (1.0 - (-driven.abs()).exp())
        };
        
        // Add warmth (even harmonics)
        saturated + self.warmth * (saturated * saturated).signum() * 0.1
    }
}

/// Stereo widener using Haas effect
pub struct StereoWidener {
    delay_samples: usize,
    width: f32,
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos: usize,
}

impl StereoWidener {
    pub fn new(sample_rate: u32, width: f32) -> Self {
        let delay_samples = (sample_rate as f32 * 0.015) as usize; // 15ms max delay
        StereoWidener {
            delay_samples,
            width: width.clamp(0.0, 1.0),
            buffer_l: vec![0.0; delay_samples],
            buffer_r: vec![0.0; delay_samples],
            write_pos: 0,
        }
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Write to buffers
        self.buffer_l[self.write_pos] = left;
        self.buffer_r[self.write_pos] = right;
        
        // Calculate delay amount based on width
        let delay = (self.delay_samples as f32 * self.width * 0.5) as usize;
        
        // Read delayed samples
        let read_pos_l = (self.write_pos + self.delay_samples - delay) % self.delay_samples;
        let read_pos_r = (self.write_pos + self.delay_samples - delay) % self.delay_samples;
        
        let delayed_l = self.buffer_l[read_pos_l];
        let delayed_r = self.buffer_r[read_pos_r];
        
        self.write_pos = (self.write_pos + 1) % self.delay_samples;
        
        // Mix: left gets right's delay, right gets left's delay
        let out_l = left * (1.0 - self.width * 0.5) + delayed_r * self.width * 0.5;
        let out_r = right * (1.0 - self.width * 0.5) + delayed_l * self.width * 0.5;
        
        (out_l, out_r)
    }
}

/// Low-shelf filter for kick drum bump
pub struct LowShelfFilter {
    freq: f32,
    gain: f32,
    q: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl LowShelfFilter {
    pub fn new(sample_rate: f32, freq: f32, gain_db: f32, q: f32) -> Self {
        let mut filter = LowShelfFilter {
            freq,
            gain: gain_db,
            q,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        filter.calculate_coefficients(sample_rate);
        filter
    }

    fn calculate_coefficients(&mut self, sample_rate: f32) {
        let a = 10.0_f32.powf(self.gain / 40.0);
        let w0 = 2.0 * PI * self.freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * self.q);
        
        let a_plus = (a + 1.0);
        let a_minus = (a - 1.0);
        
        self.b0 = a * (a_plus - a_minus * cos_w0 + 2.0 * a.sqrt() * alpha);
        self.b1 = 2.0 * a * (a_minus - a_plus * cos_w0);
        self.b2 = a * (a_plus - a_minus * cos_w0 - 2.0 * a.sqrt() * alpha);
        let a0 = a_plus + a_minus * cos_w0 + 2.0 * a.sqrt() * alpha;
        self.a1 = -2.0 * (a_minus + a_plus * cos_w0);
        self.a2 = a_plus + a_minus * cos_w0 - 2.0 * a.sqrt() * alpha;
        
        // Normalize
        self.b0 /= a0;
        self.b1 /= a0;
        self.b2 /= a0;
        self.a1 /= a0;
        self.a2 /= a0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
                     - self.a1 * self.y1 - self.a2 * self.y2;
        
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        
        output
    }
}

/// Formant shaper for guitar distortion character
pub struct FormantShaper {
    peak_freq: f32,
    resonance: f32,
    filter: LowShelfFilter,
}

impl FormantShaper {
    pub fn guitar_growl(sample_rate: f32) -> Self {
        FormantShaper {
            peak_freq: 2500.0,
            resonance: 3.0,
            filter: LowShelfFilter::new(sample_rate, 2500.0, 6.0, 3.0),
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.filter.process(input)
    }
}

/// Transient smearing for drum thickness
pub struct TransientSmear {
    attack_time: f32,
    envelope: f32,
    prev_sample: f32,
}

impl TransientSmear {
    pub fn new(attack_ms: f32) -> Self {
        TransientSmear {
            attack_time: attack_ms / 1000.0,
            envelope: 0.0,
            prev_sample: 0.0,
        }
    }

    pub fn process(&mut self, input: f32, sample_rate: f32) -> f32 {
        // Detect transient
        let delta = (input - self.prev_sample).abs();
        self.prev_sample = input;
        
        // Smooth envelope
        let attack_coef = (-1.0 / (self.attack_time * sample_rate)).exp();
        self.envelope = attack_coef * self.envelope + (1.0 - attack_coef) * delta;
        
        // Smear transients
        input * (1.0 - self.envelope * 0.3)
    }
}

/// Complete aggressive mix pipeline
pub struct AggressiveMixPipeline {
    parallel_comp: ParallelCompressor,
    tape_sat: TapeSaturation,
    stereo_widener: StereoWidener,
    kick_shelf: LowShelfFilter,
    formant: FormantShaper,
    transient_smear: TransientSmear,
    sample_rate: f32,
}

impl AggressiveMixPipeline {
    pub fn new(sample_rate: u32) -> Self {
        AggressiveMixPipeline {
            parallel_comp: ParallelCompressor::metal(),
            tape_sat: TapeSaturation::light(),
            stereo_widener: StereoWidener::new(sample_rate, 0.6),
            kick_shelf: LowShelfFilter::new(sample_rate as f32, 80.0, 4.0, 0.7),
            formant: FormantShaper::guitar_growl(sample_rate as f32),
            transient_smear: TransientSmear::new(5.0),
            sample_rate: sample_rate as f32,
        }
    }

    /// Process stereo buffer with full aggressive pipeline
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            // Parallel compression
            *l = self.parallel_comp.process(*l, self.sample_rate);
            *r = self.parallel_comp.process(*r, self.sample_rate);
            
            // Tape saturation
            *l = self.tape_sat.process(*l);
            *r = self.tape_sat.process(*r);
            
            // Formant shaping (guitar character)
            *l = self.formant.process(*l);
            *r = self.formant.process(*r);
            
            // Kick low-shelf boost
            *l = self.kick_shelf.process(*l);
            *r = self.kick_shelf.process(*r);
            
            // Transient smearing
            *l = self.transient_smear.process(*l, self.sample_rate);
            *r = self.transient_smear.process(*r, self.sample_rate);
            
            // Stereo widening (last)
            let (wide_l, wide_r) = self.stereo_widener.process(*l, *r);
            *l = wide_l;
            *r = wide_r;
        }
    }

    /// Process mono buffer
    pub fn process_mono(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.parallel_comp.process(*sample, self.sample_rate);
            *sample = self.tape_sat.process(*sample);
            *sample = self.formant.process(*sample);
            *sample = self.kick_shelf.process(*sample);
            *sample = self.transient_smear.process(*sample, self.sample_rate);
        }
    }
}
