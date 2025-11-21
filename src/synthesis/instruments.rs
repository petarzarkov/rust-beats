/// Realistic instrument synthesis for lofi vibes
use super::synthesizer::*;

/// Rhodes Electric Piano - warm, bell-like tone
pub fn generate_rhodes_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Rhodes envelope: medium attack, long sustain, gentle release
    let envelope = Envelope {
        attack: 0.005, // Faster attack but still soft
        decay: 2.0,    // Much longer decay for resonance
        sustain: 0.5,  // Lower sustain level
        release: 0.8,  // Smooth release
    };

    let note_off_time = duration * 0.9; // Hold note longer

    // Rhodes consists of multiple detuned harmonics (bell-like)
    // Fundamental and harmonics with slight FM modulation
    let mut osc_fund = Oscillator::new(Waveform::Sine, frequency);
    let mut osc_2nd = Oscillator::new(Waveform::Sine, frequency * 2.01); // Slightly detuned 2nd harmonic
    let mut osc_3rd = Oscillator::new(Waveform::Sine, frequency * 3.02);
    let mut osc_4th = Oscillator::new(Waveform::Sine, frequency * 4.98);

    // FM modulator for character (tine noise / metallic hit)
    // Lower ratio for cleaner bell sound, higher for metallic tine
    let mut fm_mod = Oscillator::new(Waveform::Sine, frequency * 2.0); // Was 14.0 which was too harsh

    // Subtle vibrato LFO
    let mut vibrato = LFO::new(4.0, 0.002);

    // Brighter filter for happier, more present sound
    let mut filter = LowPassFilter::new(2500.0, 0.3); // Slightly lowered cutoff and resonance

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        // Apply vibrato
        let vib = vibrato.next_value();
        let freq_mod = frequency * (1.0 + vib);

        // FM modulation (subtle)
        let fm_amount = fm_mod.next_sample() * 0.05 * env_amp; // Reduced from 0.3 to 0.05 for subtlety

        osc_fund.frequency = freq_mod * (1.0 + fm_amount);
        osc_2nd.frequency = freq_mod * 2.01 * (1.0 + fm_amount * 0.5);
        osc_3rd.frequency = freq_mod * 3.02;
        osc_4th.frequency = freq_mod * 4.98;

        // Mix harmonics with decreasing amplitude (bell-like spectrum)
        let mut sample = osc_fund.next_sample() * 0.5      // Fundamental
                       + osc_2nd.next_sample() * 0.25      // 2nd harmonic
                       + osc_3rd.next_sample() * 0.15      // 3rd harmonic
                       + osc_4th.next_sample() * 0.08; // 4th harmonic

        // Filter for brightness and presence
        filter.cutoff = 2200.0 + env_amp * 2000.0; // Max 4200Hz - brighter!
        sample = filter.process(sample);

        samples[i] = sample * env_amp * velocity * 0.6; // Gentle overall level
    }

    samples
}

/// Warm organ-like pad (for backing)
pub fn generate_warm_organ(frequency: f32, duration: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Very slow envelope
    let envelope = Envelope {
        attack: duration * 0.25,
        decay: 0.0,
        sustain: 1.0,
        release: duration * 0.35,
    };

    let note_off_time = duration * 0.75;

    // Organ drawbars (multiple pure harmonics)
    let mut osc1 = Oscillator::new(Waveform::Sine, frequency); // 16' (sub)
    let mut osc2 = Oscillator::new(Waveform::Sine, frequency * 1.0); // 8' (fundamental)
    let mut osc3 = Oscillator::new(Waveform::Sine, frequency * 2.0); // 4'
    let mut osc4 = Oscillator::new(Waveform::Sine, frequency * 3.0); // 2 2/3'
    let mut osc5 = Oscillator::new(Waveform::Sine, frequency * 4.0); // 2'

    // Slow chorus/vibrato
    let mut vibrato = LFO::new(0.5, 0.008);

    let mut filter = LowPassFilter::new(2500.0, 0.3);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let vib = vibrato.next_value();
        let freq_mod = frequency * (1.0 + vib);

        osc1.frequency = freq_mod * 0.5;
        osc2.frequency = freq_mod;
        osc3.frequency = freq_mod * 2.0;
        osc4.frequency = freq_mod * 3.0;
        osc5.frequency = freq_mod * 4.0;

        let mut sample = osc1.next_sample() * 0.2
            + osc2.next_sample() * 0.4
            + osc3.next_sample() * 0.2
            + osc4.next_sample() * 0.15
            + osc5.next_sample() * 0.05;

        sample = filter.process(sample);
        samples[i] = sample * env_amp * amplitude * 0.5;
    }

    samples
}

/// Soft mallet sound (like vibraphone or marimba)
pub fn generate_mallet(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Quick attack, long decay
    let envelope = Envelope {
        attack: 0.003,
        decay: 0.8,
        sustain: 0.1,
        release: 0.5,
    };

    let note_off_time = duration * 0.9;

    // Bell-like overtone series
    let mut osc1 = Oscillator::new(Waveform::Sine, frequency);
    let mut osc2 = Oscillator::new(Waveform::Sine, frequency * 2.76); // Slightly inharmonic
    let mut osc3 = Oscillator::new(Waveform::Sine, frequency * 5.40);
    let mut osc4 = Oscillator::new(Waveform::Sine, frequency * 8.93);

    // Tremolo (amplitude modulation)
    let mut tremolo = LFO::new(5.8, 0.15);

    let mut filter = LowPassFilter::new(4000.0, 0.5);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let trem = 1.0 + tremolo.next_value();

        let mut sample = osc1.next_sample() * 0.6
            + osc2.next_sample() * 0.25
            + osc3.next_sample() * 0.1
            + osc4.next_sample() * 0.05;

        sample = filter.process(sample);
        samples[i] = sample * env_amp * velocity * trem * 0.5;
    }

    samples
}

/// Soft acoustic-like guitar pluck
pub fn generate_soft_pluck(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Very fast attack, exponential decay
    let envelope = Envelope {
        attack: 0.001,
        decay: 0.4,
        sustain: 0.05,
        release: 0.3,
    };

    let note_off_time = duration * 0.85;

    // String-like harmonics
    let mut osc_fund = Oscillator::new(Waveform::Sine, frequency);
    let mut osc_2 = Oscillator::new(Waveform::Sine, frequency * 2.0);
    let mut osc_3 = Oscillator::new(Waveform::Sine, frequency * 3.0);
    let mut osc_5 = Oscillator::new(Waveform::Sine, frequency * 5.0);

    // Add slight triangle for body
    let mut body = Oscillator::new(Waveform::Triangle, frequency * 0.5);

    let mut filter = LowPassFilter::new(3000.0, 0.6);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let mut sample = osc_fund.next_sample() * 0.5
            + osc_2.next_sample() * 0.2
            + osc_3.next_sample() * 0.15
            + osc_5.next_sample() * 0.08
            + body.next_sample() * 0.1;

        // Dynamic filter
        filter.cutoff = 1500.0 + env_amp * 2000.0;
        sample = filter.process(sample);

        samples[i] = sample * env_amp * velocity * 0.55;
    }

    samples
}

/// Acoustic guitar - plucked string with body resonance
pub fn generate_acoustic_guitar(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Fast attack, exponential decay
    let envelope = Envelope {
        attack: 0.002,
        decay: 0.15,
        sustain: 0.3,
        release: 0.25,
    };

    let note_off_time = duration * 0.85;

    // String harmonics (emphasize odd harmonics for warmth)
    let mut fund = Oscillator::new(Waveform::Sine, freq);
    let mut h2 = Oscillator::new(Waveform::Sine, freq * 2.0);
    let mut h3 = Oscillator::new(Waveform::Sine, freq * 3.0);
    let mut h5 = Oscillator::new(Waveform::Sine, freq * 5.0);

    // Body resonance (low-pass with slight peak)
    let mut filter = LowPassFilter::new(2500.0, 0.6);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let mut sample = fund.next_sample() * 0.5
            + h2.next_sample() * 0.15
            + h3.next_sample() * 0.25
            + h5.next_sample() * 0.1;

        // Dynamic filter (body resonance)
        filter.cutoff = 2000.0 + env_amp * 1500.0;
        sample = filter.process(sample);

        samples[i] = sample * env_amp * velocity * 0.65;
    }

    samples
}

/// Ukulele - higher pitched, softer pluck
pub fn generate_ukulele(freq: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    // Very fast attack, quick decay
    let envelope = Envelope {
        attack: 0.001,
        decay: 0.08,
        sustain: 0.2,
        release: 0.15,
    };

    let note_off_time = duration * 0.8;

    // Bright, nylon-like harmonics
    let mut fund = Oscillator::new(Waveform::Sine, freq);
    let mut h2 = Oscillator::new(Waveform::Sine, freq * 2.0);
    let mut h3 = Oscillator::new(Waveform::Sine, freq * 3.0);
    let mut h4 = Oscillator::new(Waveform::Sine, freq * 4.0);

    let mut filter = LowPassFilter::new(3500.0, 0.4);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let mut sample = fund.next_sample() * 0.45
            + h2.next_sample() * 0.25
            + h3.next_sample() * 0.2
            + h4.next_sample() * 0.1;

        sample = filter.process(sample);
        samples[i] = sample * env_amp * velocity * 0.6;
    }

    samples
}

/// Electric guitar - square wave with slight distortion
pub fn generate_electric_guitar(
    freq: f32,
    duration: f32,
    velocity: f32,
    distortion: f32,
) -> Vec<f32> {
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = vec![0.0; num_samples];

    let envelope = Envelope {
        attack: 0.003,
        decay: 0.2,
        sustain: 0.6,
        release: 0.3,
    };

    let note_off_time = duration * 0.85;

    // Square wave with harmonics for electric guitar tone
    let mut sq1 = Oscillator::new(Waveform::Square, freq);
    let mut sq2 = Oscillator::new(Waveform::Square, freq * 2.0);

    let mut filter = LowPassFilter::new(2800.0, 0.7);

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));

        let mut sample = sq1.next_sample() * 0.6 + sq2.next_sample() * 0.3;

        // Apply distortion (soft clipping)
        if distortion > 0.0 {
            let driven = sample * (1.0 + distortion * 3.0);
            sample = driven.tanh();
        }

        filter.cutoff = 2500.0 + env_amp * 1500.0;
        sample = filter.process(sample);

        samples[i] = sample * env_amp * velocity * 0.55;
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhodes() {
        let note = generate_rhodes_note(440.0, 1.0, 0.8);
        assert!(!note.is_empty());
        assert!(note.len() > 40000);
    }

    #[test]
    fn test_warm_organ() {
        let organ = generate_warm_organ(220.0, 2.0, 0.5);
        assert!(!organ.is_empty());
    }

    #[test]
    fn test_mallet() {
        let mallet = generate_mallet(523.25, 1.5, 0.7);
        assert!(!mallet.is_empty());
    }

    #[test]
    fn test_acoustic_guitar() {
        let guitar = generate_acoustic_guitar(440.0, 1.0, 0.7);
        assert!(!guitar.is_empty());
    }

    #[test]
    fn test_ukulele() {
        let uke = generate_ukulele(523.25, 0.8, 0.7);
        assert!(!uke.is_empty());
    }
}
