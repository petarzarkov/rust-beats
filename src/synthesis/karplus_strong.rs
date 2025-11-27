use crate::synthesis::synthesizer::{get_sample_rate, LowPassFilter};
use rand::Rng;

/// Playing technique for guitar strings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayingTechnique {
    Open,               // Normal open string
    PalmMute,           // Palm muted (damped)
    Harmonic,           // Natural harmonic
    PinchHarmonic,      // Artificial harmonic (squeal)
    TremoloPick,        // Fast alternate picking
    PowerChordRoot,     // Root note of a power chord
    PowerChordFifth,    // Fifth of a power chord
    PowerChordOctave,   // Octave of a power chord
    MinorChordRoot,     // Root of a minor chord
    MinorChordThird,    // Minor third
    MinorChordFifth,    // Fifth of a minor chord
    SingleNote,         // Standard single note
}

/// Karplus-Strong string synthesizer for realistic guitar/bass sounds
/// Based on research: physical modeling of plucked strings
#[derive(Debug, Clone)]
pub struct KarplusStrong {
    buffer: Vec<f32>,
    buffer_index: usize,
    decay_factor: f32,
    damping_filter: LowPassFilter,
    technique: PlayingTechnique,
}

impl KarplusStrong {
    /// Create a new Karplus-Strong synthesizer for a given frequency
    pub fn new(frequency: f32, technique: PlayingTechnique) -> Self {
        let sample_rate = get_sample_rate() as f32;
        let buffer_length = (sample_rate / frequency).round() as usize;
        
        // Initialize buffer with white noise (the "pluck")
        let mut rng = rand::thread_rng();
        let buffer: Vec<f32> = (0..buffer_length)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();

        // Set decay factor and filter based on technique
        // Set decay factor and filter based on technique
        let (decay_factor, filter_cutoff) = match technique {
            PlayingTechnique::Open | PlayingTechnique::SingleNote => (0.996, 8000.0),
            PlayingTechnique::PalmMute => (0.90, 1000.0),
            PlayingTechnique::Harmonic | PlayingTechnique::PinchHarmonic => (0.999, 12000.0),
            PlayingTechnique::TremoloPick => (0.95, 6000.0),
            PlayingTechnique::PowerChordRoot | PlayingTechnique::MinorChordRoot => (0.98, 5000.0),
            PlayingTechnique::PowerChordFifth | PlayingTechnique::MinorChordFifth => (0.98, 5500.0),
            PlayingTechnique::PowerChordOctave | PlayingTechnique::MinorChordThird => (0.97, 6000.0),
        };

        KarplusStrong {
            buffer,
            buffer_index: 0,
            decay_factor,
            damping_filter: LowPassFilter::new(filter_cutoff, 0.7),
            technique,
        }
    }

    /// Generate the next sample
    pub fn next_sample(&mut self) -> f32 {
        // Get current sample
        let current = self.buffer[self.buffer_index];
        
        // Get next sample (with wraparound)
        let next_index = (self.buffer_index + 1) % self.buffer.len();
        let next = self.buffer[next_index];
        
        // Karplus-Strong algorithm: average of current and next, with decay
        let mut new_sample = (current + next) * 0.5 * self.decay_factor;
        
        // Apply damping filter (essential for palm mute)
        new_sample = self.damping_filter.process(new_sample);
        
        // Store the new sample back in the buffer
        self.buffer[self.buffer_index] = new_sample;
        
        // Advance buffer index
        self.buffer_index = next_index;
        
        current
    }

    /// Generate a complete note with envelope
    pub fn generate_note(frequency: f32, duration: f32, technique: PlayingTechnique) -> Vec<f32> {
        let sample_rate = get_sample_rate() as f32;
        let num_samples = (duration * sample_rate) as usize;
        let mut synth = KarplusStrong::new(frequency, technique);
        
        let mut buffer = Vec::with_capacity(num_samples);
        
        // Simple envelope for amplitude shaping
        for i in 0..num_samples {
            let t = i as f32 / num_samples as f32;
            
            // Envelope depends on technique
            let envelope = match technique {
                PlayingTechnique::Open | PlayingTechnique::SingleNote | 
                PlayingTechnique::PowerChordRoot | PlayingTechnique::PowerChordFifth | 
                PlayingTechnique::PowerChordOctave | PlayingTechnique::MinorChordRoot |
                PlayingTechnique::MinorChordThird | PlayingTechnique::MinorChordFifth => {
                    // Quick attack, long sustain
                    if t < 0.01 {
                        t / 0.01
                    } else {
                        1.0 - (t - 0.01) * 0.3
                    }
                }
                PlayingTechnique::PalmMute => {
                    // Quick attack, fast decay (percussive)
                    if t < 0.005 {
                        t / 0.005
                    } else {
                        (1.0 - t).powf(2.0)
                    }
                }
                PlayingTechnique::Harmonic | PlayingTechnique::PinchHarmonic => {
                    // Very smooth, sustained
                    if t < 0.02 {
                        t / 0.02
                    } else {
                        1.0 - (t - 0.02) * 0.2
                    }
                }
                PlayingTechnique::TremoloPick => {
                    // Sharp attack, medium sustain
                    if t < 0.005 {
                        t / 0.005
                    } else {
                        1.0 - (t - 0.005) * 0.5
                    }
                }
            };
            
            let sample = synth.next_sample() * envelope;
            buffer.push(sample);
        }
        
        buffer
    }
}

/// Generate a metal guitar riff using Karplus-Strong synthesis
pub fn generate_metal_guitar_note(
    frequency: f32,
    duration: f32,
    velocity: f32,
    _palm_mute: bool,
    technique: PlayingTechnique,
) -> Vec<f32> {
    let mut buffer = KarplusStrong::generate_note(frequency, duration, technique);
    
    // Apply velocity scaling
    for sample in buffer.iter_mut() {
        *sample *= velocity;
    }
    
    buffer
}

/// Generate a bass note using Karplus-Strong with extra low-end
/// Enhanced with lower cutoff and sub-bass boost for heaviness
pub fn generate_metal_bass_string(
    frequency: f32,
    duration: f32,
    velocity: f32,
) -> Vec<f32> {
    // Bass uses slightly different parameters for more weight
    let sample_rate = get_sample_rate() as f32;
    let num_samples = (duration * sample_rate) as usize;
    
    let mut synth = KarplusStrong::new(frequency, PlayingTechnique::Open);
    // Increase decay for bass sustain
    synth.decay_factor = 0.998;
    
    // Lower cutoff filter for bass damping (separate from guitar)
    // Bass should be darker and heavier
    let mut bass_filter = LowPassFilter::new(2000.0, 0.7); // Lower cutoff than guitar
    
    let mut buffer = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let t = i as f32 / num_samples as f32;
        
        // Bass envelope: punchy attack, long sustain
        let envelope = if t < 0.005 {
            t / 0.005
        } else if t < 0.8 {
            0.9 + 0.1 * (1.0 - (t - 0.005) / 0.795)
        } else {
            0.9 * (1.0 - (t - 0.8) / 0.2)
        };
        
        let mut sample = synth.next_sample() * envelope * velocity;
        
        // Apply bass filter (darker tone)
        sample = bass_filter.process(sample);
        
        // Sub-bass boost: Add harmonics in 60-100Hz range
        // Simple approach: boost frequencies below 100Hz
        if frequency < 100.0 {
            // Boost sub-bass frequencies
            sample *= 1.3;
        } else if frequency < 200.0 {
            // Slight boost for low bass
            sample *= 1.15;
        }
        
        buffer.push(sample);
    }
    
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_karplus_strong_creation() {
        let synth = KarplusStrong::new(440.0, PlayingTechnique::Open);
        assert!(synth.buffer.len() > 0);
        assert_eq!(synth.buffer_index, 0);
    }

    #[test]
    fn test_karplus_strong_generates_samples() {
        let mut synth = KarplusStrong::new(440.0, PlayingTechnique::Open);
        
        // Generate some samples
        for _ in 0..100 {
            let sample = synth.next_sample();
            assert!(sample.abs() <= 1.0, "Sample should be within valid range");
        }
    }

    #[test]
    fn test_palm_mute_vs_open() {
        let open = KarplusStrong::new(220.0, PlayingTechnique::Open);
        let muted = KarplusStrong::new(220.0, PlayingTechnique::PalmMute);
        
        // Palm mute should have lower decay factor
        assert!(muted.decay_factor < open.decay_factor);
    }

    #[test]
    fn test_generate_note() {
        let buffer = KarplusStrong::generate_note(
            330.0,
            0.5,
            PlayingTechnique::Open
        );
        
        assert!(buffer.len() > 0);
        // Check that samples are within valid range
        for sample in buffer.iter() {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_metal_guitar_note() {
        let buffer = generate_metal_guitar_note(
            220.0,
            0.3,
            0.8,
            true, // palm mute
            PlayingTechnique::PalmMute
        );
        
        assert!(buffer.len() > 0);
        assert!(buffer.iter().all(|&s| s.abs() <= 1.0));
    }

    #[test]
    fn test_metal_bass_string() {
        let buffer = generate_metal_bass_string(
            110.0,
            0.5,
            0.9
        );
        
        assert!(buffer.len() > 0);
        assert!(buffer.iter().all(|&s| s.abs() <= 1.0));
    }

    #[test]
    fn test_playing_techniques() {
        let open = KarplusStrong::generate_note(440.0, 0.1, PlayingTechnique::Open);
        let muted = KarplusStrong::generate_note(440.0, 0.1, PlayingTechnique::PalmMute);
        let harmonic = KarplusStrong::generate_note(440.0, 0.1, PlayingTechnique::Harmonic);
        
        // All should generate valid buffers
        assert!(open.len() > 0);
        assert!(muted.len() > 0);
        assert!(harmonic.len() > 0);
    }
}
