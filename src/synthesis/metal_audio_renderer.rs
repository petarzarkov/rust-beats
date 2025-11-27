use crate::composition::{
    metal_song_generator::{MetalSong, MetalRiff, MetalSection, MetalSubgenre, ChordType, SectionIntensity, RhythmPattern},
    rhythm_generator,
    bass_generator::BassMode,
};
use crate::synthesis::{
    karplus_strong::{generate_metal_guitar_note, generate_metal_bass_string, PlayingTechnique},
    metal_dsp::{TubeDistortion, NoiseGate},
    cabinet::CabinetSimulator,
    drums::MetalDrums,
};
use crate::synthesis::synthesizer::get_sample_rate;

/// Audio renderer for metal songs
/// Converts symbolic representation (MetalSong) into audio samples
pub struct MetalAudioRenderer {
    drums: MetalDrums,
    distortion: TubeDistortion,
    cabinet: CabinetSimulator,
    noise_gate: NoiseGate,
    sample_rate: u32,
}

impl MetalAudioRenderer {
    /// Create a new metal audio renderer
    pub fn new() -> Self {
        Self {
            drums: MetalDrums::new(),
            distortion: TubeDistortion::new(0.8, 0.7), // High gain
            cabinet: CabinetSimulator::metal_4x12(),
            noise_gate: NoiseGate::new(0.001),
            sample_rate: get_sample_rate(),
        }
    }

    /// Render a complete metal song to audio
    pub fn render_song(&mut self, song: &MetalSong, duration_per_section: f32) -> Vec<f32> {
        let mut full_audio = Vec::new();
        
        // Render each section
        for (section_type, riff) in &song.sections {
            let section_audio = self.render_section(*section_type, riff, duration_per_section, song.tempo, song.subgenre);
            full_audio.extend(section_audio);
        }
        
        // Normalize
        Self::normalize(&mut full_audio);
        
        full_audio
    }

    /// Render a single section with dynamic intensity and bass locking
    pub fn render_section(
        &mut self,
        section_type: MetalSection,
        riff: &MetalRiff,
        duration: f32,
        tempo: u16,
        subgenre: MetalSubgenre,
    ) -> Vec<f32> {
        // Calculate note duration based on tempo
        let beat_duration = 60.0 / tempo as f32;
        let intensity = section_type.intensity();

        let mut section_audio = Vec::new();

        // Render guitar riff with variable durations
        let guitar_audio = self.render_guitar_riff(riff, beat_duration);
        
        // Generate drum pattern first (needed for bass locking)
        let (kick_pattern, _, _) = self.generate_drum_patterns(section_type, duration, tempo, subgenre);
        
        // Render bass with locking mode
        let bass_mode = crate::composition::bass_generator::MetalBassGenerator::mode_for_subgenre(subgenre);
        // For breakdowns, use 8th note subdivision for bass too
        let bass_note_duration = if matches!(section_type, MetalSection::Breakdown) {
            beat_duration / 2.0
        } else {
            beat_duration / 4.0
        };
        let bass_audio = self.render_bass_riff_locked(
            &riff.notes,
            &kick_pattern,
            bass_note_duration,
            bass_mode,
            &riff,
        );
        
        // Render drums with subgenre-specific patterns
        let drum_audio = self.render_drums(section_type, duration, tempo, subgenre);

        // Dynamic mixing based on section intensity
        let (guitar_level, bass_level, drum_level) = match intensity {
            SectionIntensity::Low => (0.3, 0.25, 0.2),      // Intro: Quieter
            SectionIntensity::Medium => (0.4, 0.35, 0.25),  // Verse: Standard
            SectionIntensity::High => (0.45, 0.4, 0.3),    // Chorus: Louder
            SectionIntensity::Extreme => (0.5, 0.45, 0.35), // Breakdown: Maximum
        };

        // Mix guitar, bass, and drums with dynamic levels
        let max_len = guitar_audio.len().max(bass_audio.len()).max(drum_audio.len());
        section_audio.resize(max_len, 0.0);

        for i in 0..max_len {
            let guitar = if i < guitar_audio.len() { guitar_audio[i] } else { 0.0 };
            let bass = if i < bass_audio.len() { bass_audio[i] } else { 0.0 };
            let drums = if i < drum_audio.len() { drum_audio[i] } else { 0.0 };
            
            // Apply dynamic mixing
            section_audio[i] = guitar * guitar_level + bass * bass_level + drums * drum_level;
        }

        section_audio
    }

    /// Render bass guitar riff with locking support
    fn render_bass_riff_locked(
        &mut self,
        guitar_notes: &[u8],
        kick_pattern: &[bool],
        note_duration: f32,
        mode: BassMode,
        _riff: &MetalRiff,
    ) -> Vec<f32> {
        let mut bass_audio = Vec::new();
        let _sample_rate = self.sample_rate as f32;
        let sixteenth_samples = (note_duration * _sample_rate) as usize;
        
        match mode {
            BassMode::Lock => {
                // Lock Mode: Bass plays exactly when kick drum plays
                for (i, &kick_hit) in kick_pattern.iter().enumerate() {
                    if kick_hit {
                        // Get corresponding guitar note
                        let guitar_idx = (i * guitar_notes.len()) / kick_pattern.len().max(1);
                        let guitar_note = if guitar_idx < guitar_notes.len() {
                            guitar_notes[guitar_idx]
                        } else {
                            guitar_notes[0]
                        };
                        
                        // Bass plays root of power chord or guitar note root
                        let bass_note = guitar_note.saturating_sub(12);
                        let frequency = 440.0 * 2.0_f32.powf((bass_note as f32 - 69.0) / 12.0);
                        
                        // Generate bass note with heavy tone
                        let bass_sample = generate_metal_bass_string(frequency, note_duration, 0.9);
                        bass_audio.extend(bass_sample);
                    } else {
                        // No kick = sustain or silence
                        // Sustain previous note briefly, then silence
                        bass_audio.extend(vec![0.0; sixteenth_samples / 2]);
                    }
                }
            },
            BassMode::Counterpoint => {
                // Counterpoint Mode: Distinct bass lines
                // For now, use simplified version - can be enhanced
                for &note in guitar_notes {
                    let bass_note = note.saturating_sub(12);
                    let frequency = 440.0 * 2.0_f32.powf((bass_note as f32 - 69.0) / 12.0);
                    let bass_sample = generate_metal_bass_string(frequency, note_duration, 0.8);
                    bass_audio.extend(bass_sample);
                }
            },
            BassMode::Follow => {
                // Follow Mode: Traditional bass following guitar
                for &note in guitar_notes {
                    let bass_note = note.saturating_sub(12);
                    let frequency = 440.0 * 2.0_f32.powf((bass_note as f32 - 69.0) / 12.0);
                    let bass_sample = generate_metal_bass_string(frequency, note_duration, 0.8);
                    bass_audio.extend(bass_sample);
                }
            },
        }
        
        bass_audio
    }

    /// Generate drum patterns (returns patterns for bass locking)
    fn generate_drum_patterns(
        &self,
        section: MetalSection,
        duration: f32,
        tempo: u16,
        subgenre: MetalSubgenre,
    ) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
        let sample_rate = self.sample_rate as f32;
        let beat_duration = 60.0 / tempo as f32;
        let sixteenth_duration = beat_duration / 4.0;
        let steps = (duration / sixteenth_duration).ceil() as usize;
        
        match section {
            MetalSection::Breakdown => {
                rhythm_generator::generate_breakdown_pattern(steps, 0.9)
            },
            MetalSection::Verse | MetalSection::Chorus | MetalSection::Solo => {
                if matches!(subgenre, MetalSubgenre::DeathMetal) && matches!(section, MetalSection::Verse) {
                    rhythm_generator::generate_blast_beat(subgenre, steps)
                } else {
                    let pulses = if matches!(subgenre, MetalSubgenre::ProgressiveMetal) { 7 } else { 5 };
                    let kick = rhythm_generator::generate_euclidean_pattern(steps, pulses);
                    
                    let mut snare = vec![false; steps];
                    let mut cymbal = vec![false; steps];
                    
                    for i in 0..steps {
                        if i % 16 == 4 || i % 16 == 12 {
                            snare[i] = true;
                        }
                        // Sparse cymbals: only on beat 1 of each bar (every 16th step)
                        if i % 16 == 0 {
                            cymbal[i] = true;
                        }
                    }
                    (kick, snare, cymbal)
                }
            },
            _ => {
                let mut kick = vec![false; steps];
                let mut snare = vec![false; steps];
                let mut cymbal = vec![false; steps];
                
                for i in 0..steps {
                    if i % 16 == 0 { kick[i] = true; cymbal[i] = true; }
                    if i % 16 == 8 { snare[i] = true; }
                }
                (kick, snare, cymbal)
            }
        }
    }

    /// Render guitar riff with chords support and variable durations
    fn render_guitar_riff(&mut self, riff: &MetalRiff, beat_duration: f32) -> Vec<f32> {
        let mut guitar_audio = Vec::new();
        
        for (i, &note) in riff.notes.iter().enumerate() {
            let palm_muted = riff.palm_muted[i];
            let chord_type = riff.chord_types.get(i).copied().unwrap_or(ChordType::Single);
            let rhythm = riff.rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            // Handle rests
            if rhythm == RhythmPattern::Rest {
                let rest_duration = beat_duration / 4.0; // Default to sixteenth rest
                let rest_samples = (rest_duration * self.sample_rate as f32) as usize;
                guitar_audio.extend(vec![0.0; rest_samples]);
                continue;
            }
            
            // Calculate note duration from rhythm pattern
            let note_duration = match rhythm {
                RhythmPattern::QuarterNote => beat_duration,
                RhythmPattern::EighthNote => beat_duration / 2.0,
                RhythmPattern::SixteenthNote => beat_duration / 4.0,
                RhythmPattern::ThirtySecondNote => beat_duration / 8.0,
                RhythmPattern::Gallop => {
                    // Gallop is handled specially - render 3 notes
                    if let Some(gallop_samples) = self.render_gallop_pattern(riff, i, beat_duration, palm_muted, chord_type) {
                        guitar_audio.extend(gallop_samples);
                    }
                    continue; // Skip normal note rendering for gallop
                },
                RhythmPattern::Rest => beat_duration / 4.0, // Shouldn't reach here
            };
            
            // Convert MIDI note to frequency: f = 440 * 2.0_f32.powf((n-69)/12)
            let freq_root = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
            
            let is_palm_muted = palm_muted;
            let velocity = 0.8;

            let mut note_samples = Vec::new();

            match chord_type {
                ChordType::Power => {
                    // Render Root
                    let root_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::PowerChordRoot);
                    
                    // Render 5th (+7 semitones)
                    let freq_5th = 440.0 * 2.0_f32.powf(((note + 7) as f32 - 69.0) / 12.0);
                    let fifth_samples = generate_metal_guitar_note(freq_5th, note_duration, velocity, is_palm_muted, PlayingTechnique::PowerChordFifth);
                    
                    // Render Octave (+12 semitones)
                    let freq_oct = 440.0 * 2.0_f32.powf(((note + 12) as f32 - 69.0) / 12.0);
                    let oct_samples = generate_metal_guitar_note(freq_oct, note_duration, velocity, is_palm_muted, PlayingTechnique::PowerChordOctave);
                    
                    // Mix voices (Root loudest, 5th and Octave slightly quieter)
                    let max_len = root_samples.len().max(fifth_samples.len()).max(oct_samples.len());
                    note_samples.resize(max_len, 0.0);
                    
                    for j in 0..max_len {
                        let s1 = if j < root_samples.len() { root_samples[j] } else { 0.0 };
                        let s2 = if j < fifth_samples.len() { fifth_samples[j] } else { 0.0 };
                        let s3 = if j < oct_samples.len() { oct_samples[j] } else { 0.0 };
                        
                        note_samples[j] = s1 * 0.5 + s2 * 0.3 + s3 * 0.2;
                    }
                },
                ChordType::Minor => {
                     // Render Root
                    let root_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordRoot);
                    
                    // Render Minor 3rd (+3 semitones)
                    let freq_m3 = 440.0 * 2.0_f32.powf(((note + 3) as f32 - 69.0) / 12.0);
                    let m3_samples = generate_metal_guitar_note(freq_m3, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordThird);
                    
                    // Render 5th (+7 semitones)
                    let freq_5th = 440.0 * 2.0_f32.powf(((note + 7) as f32 - 69.0) / 12.0);
                    let fifth_samples = generate_metal_guitar_note(freq_5th, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordFifth);
                    
                    // Mix
                    let max_len = root_samples.len().max(m3_samples.len()).max(fifth_samples.len());
                    note_samples.resize(max_len, 0.0);
                    
                    for j in 0..max_len {
                        let s1 = if j < root_samples.len() { root_samples[j] } else { 0.0 };
                        let s2 = if j < m3_samples.len() { m3_samples[j] } else { 0.0 };
                        let s3 = if j < fifth_samples.len() { fifth_samples[j] } else { 0.0 };
                        
                        note_samples[j] = s1 * 0.4 + s2 * 0.3 + s3 * 0.3;
                    }
                },
                _ => {
                    // Single note
                    note_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::SingleNote);
                }
            }
            
            guitar_audio.extend(note_samples);
        }
        
        // Apply distortion and cabinet simulation to the whole riff
        self.process_guitar_chain(&guitar_audio)
    }

    /// Render a gallop pattern (eighth + two sixteenths)
    fn render_gallop_pattern(
        &mut self,
        riff: &MetalRiff,
        start_idx: usize,
        beat_duration: f32,
        palm_muted: bool,
        chord_type: ChordType,
    ) -> Option<Vec<f32>> {
        if start_idx >= riff.notes.len() {
            return None;
        }
        
        let note = riff.notes[start_idx];
        let freq_root = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        let velocity = 0.8;
        
        // Gallop durations: [eighth, sixteenth, sixteenth]
        let durations = vec![
            beat_duration / 2.0,  // Eighth note
            beat_duration / 4.0,  // Sixteenth note
            beat_duration / 4.0,  // Sixteenth note
        ];
        
        let mut gallop_samples = Vec::new();
        for duration in durations {
            let note_samples = match chord_type {
                ChordType::Power => {
                    let root_samples = generate_metal_guitar_note(freq_root, duration, velocity, palm_muted, PlayingTechnique::PowerChordRoot);
                    let freq_5th = 440.0 * 2.0_f32.powf(((note + 7) as f32 - 69.0) / 12.0);
                    let fifth_samples = generate_metal_guitar_note(freq_5th, duration, velocity, palm_muted, PlayingTechnique::PowerChordFifth);
                    let freq_oct = 440.0 * 2.0_f32.powf(((note + 12) as f32 - 69.0) / 12.0);
                    let oct_samples = generate_metal_guitar_note(freq_oct, duration, velocity, palm_muted, PlayingTechnique::PowerChordOctave);
                    
                    let max_len = root_samples.len().max(fifth_samples.len()).max(oct_samples.len());
                    let mut mixed = vec![0.0; max_len];
                    for j in 0..max_len {
                        let s1 = if j < root_samples.len() { root_samples[j] } else { 0.0 };
                        let s2 = if j < fifth_samples.len() { fifth_samples[j] } else { 0.0 };
                        let s3 = if j < oct_samples.len() { oct_samples[j] } else { 0.0 };
                        mixed[j] = s1 * 0.5 + s2 * 0.3 + s3 * 0.2;
                    }
                    mixed
                },
                _ => {
                    generate_metal_guitar_note(freq_root, duration, velocity, palm_muted, PlayingTechnique::SingleNote)
                },
            };
            gallop_samples.extend(note_samples);
        }
        
        Some(gallop_samples)
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

    /// Render drums for a section with subgenre-specific patterns
    fn render_drums(&self, section: MetalSection, duration: f32, tempo: u16, subgenre: MetalSubgenre) -> Vec<f32> {
        let sample_rate = self.sample_rate as f32;
        let num_samples = (duration * sample_rate) as usize;
        let mut drum_audio = vec![0.0; num_samples];
        
        let beat_duration = 60.0 / tempo as f32;
        let sixteenth_duration = beat_duration / 4.0;
        
        // Determine number of 16th note steps in the section
        let steps = (duration / sixteenth_duration).ceil() as usize;
        
        // Generate patterns using rhythm generator (reuse the helper)
        let (kick_pattern, snare_pattern, cymbal_pattern) = self.generate_drum_patterns(section, duration, tempo, subgenre);

        // Render the patterns with humanization
        for i in 0..steps {
            let time = i as f32 * sixteenth_duration;
            let sample_idx = (time * sample_rate) as usize;
            
            if sample_idx >= num_samples { break; }
            
            if kick_pattern.get(i).copied().unwrap_or(false) {
                let kick_sound = self.drums.generate_kick(0.8);
                self.mix_drum_hit(&mut drum_audio, &kick_sound, sample_idx);
            }
            
            if snare_pattern.get(i).copied().unwrap_or(false) {
                let snare_sound = self.drums.generate_snare(0.7);
                self.mix_drum_hit(&mut drum_audio, &snare_sound, sample_idx);
            }
            
            if cymbal_pattern.get(i).copied().unwrap_or(false) {
                let hihat_sound = self.drums.generate_hihat(0.6, false);
                self.mix_drum_hit(&mut drum_audio, &hihat_sound, sample_idx);
            }
        }

        drum_audio
    }

    /// Mix a drum hit into the main buffer
    fn mix_drum_hit(&self, buffer: &mut [f32], hit: &[f32], start_idx: usize) {
        for (i, &sample) in hit.iter().enumerate() {
            if start_idx + i < buffer.len() {
                buffer[start_idx + i] += sample;
            }
        }
    }


    
    /// Normalize audio buffer
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
    use crate::composition::metal_song_generator::MetalSongGenerator;

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
            chord_types: vec![ChordType::Single; 4],
            palm_muted: vec![true, false, false, true],
            rhythms: vec![RhythmPattern::SixteenthNote; 4],
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
        let drums = renderer.render_drums(MetalSection::Verse, 2.0, 120, MetalSubgenre::HeavyMetal);
        
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
            chord_types: vec![ChordType::Single; 4],
            palm_muted: vec![true, true, false, true],
            rhythms: vec![RhythmPattern::SixteenthNote; 4],
            playability_score: 0.95,
        };

        let audio = renderer.render_section(
            MetalSection::Verse,
            &riff,
            2.0, // 2 seconds
            120,
            MetalSubgenre::HeavyMetal,
        );

        assert_eq!(audio.len(), 2 * 44100);
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
            notes: vec![40, 40, 40, 40],
            chord_types: vec![ChordType::Single; 4],
            palm_muted: vec![true, true, true, true],
            rhythms: vec![RhythmPattern::SixteenthNote; 4],
            playability_score: 1.0,
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
            let audio = renderer.render_section(section, &riff, 1.0, 120, MetalSubgenre::HeavyMetal);
            assert!(!audio.is_empty(), "Section {:?} should produce audio", section);
        }
    }
}
