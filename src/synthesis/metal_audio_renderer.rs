use crate::composition::{
    metal_song_generator::{MetalSong, MetalRiff, MetalSection},
};
use crate::synthesis::{
    karplus_strong::generate_metal_guitar_note,
    metal_dsp::{TubeDistortion, NoiseGate},
    cabinet::CabinetSimulator,
    synthesizer::get_sample_rate,
    drums::{generate_kick, generate_snare, generate_hihat},
};

/// Audio renderer for metal songs
/// Converts symbolic representation (MetalSong) into audio samples
pub struct MetalAudioRenderer {
    sample_rate: u32,
    noise_gate: NoiseGate,
    distortion: TubeDistortion,
    cabinet: CabinetSimulator,
}

impl MetalAudioRenderer {
    /// Create a new metal audio renderer
    pub fn new() -> Self {
        MetalAudioRenderer {
            sample_rate: get_sample_rate(),
            noise_gate: NoiseGate::metal(),
            distortion: TubeDistortion::metal(),
            cabinet: CabinetSimulator::metal_4x12(),
        }
    }

    /// Render a complete metal song to audio
    pub fn render_song(&mut self, song: &MetalSong, duration_per_section: f32) -> Vec<f32> {
        let mut output = Vec::new();

        for (section, riff) in &song.sections {
            let section_audio = self.render_section(*section, riff, duration_per_section, song.tempo);
            output.extend(section_audio);
        }

        output
    }

    /// Render a single section
    fn render_section(
        &mut self,
        section: MetalSection,
        riff: &MetalRiff,
        duration: f32,
        tempo: u16,
    ) -> Vec<f32> {
        // Calculate note duration based on tempo
        let beat_duration = 60.0 / tempo as f32;
        let note_duration = beat_duration / 4.0; // 16th notes

        let mut section_audio = Vec::new();

        // Render guitar riff
        let guitar_audio = self.render_guitar_riff(riff, note_duration);
        
        // Render drums
        let drum_audio = self.render_drums(section, duration, tempo);

        // Mix guitar and drums (simple mixing for now)
        let max_len = guitar_audio.len().max(drum_audio.len());
        section_audio.resize(max_len, 0.0);

        for i in 0..max_len {
            let guitar = if i < guitar_audio.len() { guitar_audio[i] } else { 0.0 };
            let drums = if i < drum_audio.len() { drum_audio[i] } else { 0.0 };
            
            // Mix with appropriate levels (guitar louder, drums for support)
            section_audio[i] = guitar * 0.6 + drums * 0.4;
        }

        section_audio
    }

    /// Render a guitar riff through the full DSP chain
    fn render_guitar_riff(&mut self, riff: &MetalRiff, note_duration: f32) -> Vec<f32> {
        let mut output = Vec::new();

        for (i, &note) in riff.notes.iter().enumerate() {
            let is_palm_muted = riff.palm_muted.get(i).copied().unwrap_or(false);
            
            // Convert MIDI note to frequency
            let frequency = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
            let velocity = 0.8; // Standard velocity

            // Generate note using Karplus-Strong synthesis
            let note_samples = generate_metal_guitar_note(frequency, note_duration, velocity, is_palm_muted);

            // Apply DSP chain: Noise Gate → Distortion → Cabinet
            let processed = self.process_guitar_chain(&note_samples);

            output.extend(processed);
        }

        output
    }

    /// Process audio through the guitar DSP chain
    fn process_guitar_chain(&mut self, samples: &[f32]) -> Vec<f32> {
        let mut processed = Vec::with_capacity(samples.len());

        for &sample in samples {
            // 1. Noise Gate (remove hum between notes)
            let gated = self.noise_gate.process(sample);
            
            // 2. Tube Distortion (add gain and harmonics)
            let distorted = self.distortion.process(gated);
            
            // 3. Cabinet Simulation (speaker coloration)
            let final_sample = self.cabinet.process(distorted);
            
            processed.push(final_sample);
        }

        processed
    }

    /// Render drums for a section
    fn render_drums(&self, section: MetalSection, duration: f32, tempo: u16) -> Vec<f32> {
        let sample_count = (duration * self.sample_rate as f32) as usize;
        let mut drum_audio = vec![0.0; sample_count];

        let beat_duration = 60.0 / tempo as f32;
        let samples_per_beat = (beat_duration * self.sample_rate as f32) as usize;

        // Simple drum pattern based on section type
        match section {
            MetalSection::Breakdown => {
                // Heavy, slow pattern
                self.add_drum_pattern(&mut drum_audio, samples_per_beat, 2, true, false);
            }
            MetalSection::Verse | MetalSection::Chorus => {
                // Standard rock beat
                self.add_drum_pattern(&mut drum_audio, samples_per_beat, 1, true, true);
            }
            MetalSection::Solo => {
                // Fast, driving pattern
                self.add_drum_pattern(&mut drum_audio, samples_per_beat / 2, 1, true, true);
            }
            _ => {
                // Simple pattern for intro/outro
                self.add_drum_pattern(&mut drum_audio, samples_per_beat, 2, false, false);
            }
        }

        drum_audio
    }

    /// Add a drum pattern to the audio buffer
    fn add_drum_pattern(
        &self,
        buffer: &mut [f32],
        interval: usize,
        kick_interval: usize,
        add_snare: bool,
        add_hihat: bool,
    ) {
        let kick_samples = generate_kick(1.0);
        let snare_samples = if add_snare { Some(generate_snare(1.0)) } else { None };
        let hihat_samples = if add_hihat { Some(generate_hihat(0.6, false)) } else { None };

        let mut position = 0;
        let mut beat_count = 0;

        while position < buffer.len() {
            // Add kick
            if beat_count % kick_interval == 0 {
                self.mix_drum_sample(buffer, position, &kick_samples, 0.8);
            }

            // Add snare on backbeats
            if add_snare && beat_count % 2 == 1 {
                if let Some(ref snare) = snare_samples {
                    self.mix_drum_sample(buffer, position, snare, 0.7);
                }
            }

            // Add hi-hat
            if add_hihat {
                if let Some(ref hihat) = hihat_samples {
                    self.mix_drum_sample(buffer, position, hihat, 0.3);
                }
            }

            position += interval;
            beat_count += 1;
        }
    }

    /// Mix a drum sample into the buffer at the specified position
    fn mix_drum_sample(&self, buffer: &mut [f32], position: usize, sample: &[f32], level: f32) {
        for (i, &s) in sample.iter().enumerate() {
            let idx = position + i;
            if idx >= buffer.len() {
                break;
            }
            buffer[idx] += s * level;
        }
    }

    /// Normalize audio to prevent clipping
    pub fn normalize(samples: &mut [f32]) {
        let max_amplitude = samples.iter()
            .map(|&s| s.abs())
            .fold(0.0f32, f32::max);

        if max_amplitude > 0.0 {
            let scale = 0.95 / max_amplitude; // Leave headroom
            for sample in samples.iter_mut() {
                *sample *= scale;
            }
        }
    }
}

impl Default for MetalAudioRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::{
        metal_song_generator::{MetalSongGenerator, MetalSubgenre},
    };

    #[test]
    fn test_renderer_creation() {
        let renderer = MetalAudioRenderer::new();
        assert_eq!(renderer.sample_rate, get_sample_rate());
    }

    #[test]
    fn test_render_guitar_riff() {
        let mut renderer = MetalAudioRenderer::new();
        
        let riff = MetalRiff {
            notes: vec![40, 42, 43, 40], // E, F#, G, E
            palm_muted: vec![true, false, false, true],
            playability_score: 0.9,
        };

        let audio = renderer.render_guitar_riff(&riff, 0.25);
        assert!(!audio.is_empty());
        
        // Check that audio was generated
        let has_signal = audio.iter().any(|&s| s.abs() > 0.01);
        assert!(has_signal);
    }

    #[test]
    fn test_render_drums() {
        let renderer = MetalAudioRenderer::new();
        let drums = renderer.render_drums(MetalSection::Verse, 2.0, 120);
        
        assert!(!drums.is_empty());
        
        // Check that drums were generated
        let has_signal = drums.iter().any(|&s| s.abs() > 0.01);
        assert!(has_signal);
    }

    #[test]
    fn test_render_section() {
        let mut renderer = MetalAudioRenderer::new();
        
        let riff = MetalRiff {
            notes: vec![40, 40, 43, 40],
            palm_muted: vec![true, true, false, true],
            playability_score: 0.95,
        };

        let audio = renderer.render_section(MetalSection::Verse, &riff, 2.0, 140);
        assert!(!audio.is_empty());
    }

    #[test]
    fn test_render_complete_song() {
        let generator = MetalSongGenerator::new(MetalSubgenre::HeavyMetal);
        let song = generator.generate_song();
        
        let mut renderer = MetalAudioRenderer::new();
        let audio = renderer.render_song(&song, 2.0); // 2 seconds per section
        
        assert!(!audio.is_empty());
        
        // Should have audio for all sections
        let expected_min_samples = (2.0 * song.sections.len() as f32 * get_sample_rate() as f32) as usize;
        assert!(audio.len() >= expected_min_samples / 2); // Allow some flexibility
    }

    #[test]
    fn test_normalize() {
        let mut samples = vec![0.5, -1.5, 0.8, -0.3];
        MetalAudioRenderer::normalize(&mut samples);
        
        // Check that max amplitude is <= 0.95
        let max = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        assert!(max <= 0.95);
        assert!(max > 0.9); // Should be close to 0.95
    }

    #[test]
    fn test_dsp_chain_processing() {
        let mut renderer = MetalAudioRenderer::new();
        
        // Generate some test samples
        let input = vec![0.1, 0.2, 0.3, 0.2, 0.1];
        let output = renderer.process_guitar_chain(&input);
        
        assert_eq!(output.len(), input.len());
        
        // Output should be different from input (processed)
        let is_different = output.iter().zip(input.iter())
            .any(|(a, b)| (a - b).abs() > 0.001);
        assert!(is_different);
    }

    #[test]
    fn test_all_sections_render() {
        let mut renderer = MetalAudioRenderer::new();
        
        let riff = MetalRiff {
            notes: vec![40, 42, 43],
            palm_muted: vec![true, false, true],
            playability_score: 0.9,
        };

        let sections = vec![
            MetalSection::Intro,
            MetalSection::Verse,
            MetalSection::Chorus,
            MetalSection::Breakdown,
            MetalSection::Solo,
            MetalSection::Outro,
        ];

        for section in sections {
            let audio = renderer.render_section(section, &riff, 1.0, 120);
            assert!(!audio.is_empty(), "Section {:?} should produce audio", section);
        }
    }
}
