use crate::composition::{
    metal_song_generator::{MetalSong, MetalRiff, MetalSection, MetalSubgenre, ChordType, SectionIntensity, RhythmPattern, RhythmicFeel},
    rhythm_generator,
    bass_generator::BassMode,
};
use crate::synthesis::{
    karplus_strong::{generate_metal_guitar_note, generate_metal_bass_string, PlayingTechnique},
    metal_dsp::{MetalDSPChain, TubeDistortion},
    cabinet::CabinetSimulator,
    drums::MetalDrums,
    fx::generate_drop_kick,
};
use crate::utils::get_sample_rate;

pub struct MetalAudioRenderer {
    drums: MetalDrums,
    dsp_chain: MetalDSPChain,
    bass_dsp: TubeDistortion,
    cabinet: CabinetSimulator,
    sample_rate: u32,
}

impl MetalAudioRenderer {
    pub fn new() -> Self {
        Self {
            drums: MetalDrums::new(),
            // REDUCED DRIVE to prevent noise wall (was higher default)
            dsp_chain: MetalDSPChain::new(6.0), 
            bass_dsp: TubeDistortion::new(5.0, 1.0),
            cabinet: CabinetSimulator::metal_4x12(),
            sample_rate: get_sample_rate(),
        }
    }

    pub fn render_song(&mut self, song: &MetalSong, duration_per_section: f32) -> Vec<f32> {
        let mut full_audio = Vec::new();
        
        for (section_type, riff) in &song.sections {
            let section_audio = self.render_section(*section_type, riff, duration_per_section, song.tempo, song.subgenre);
            full_audio.extend(section_audio);
        }
        
        // Final Limiter instead of Normalize
        // Normalize just finds peak, Limiter compresses peaks
        Self::apply_limiter(&mut full_audio, 0.95);
        
        full_audio
    }

    pub fn render_section(
        &mut self,
        section_type: MetalSection,
        riff: &MetalRiff,
        duration: f32,
        tempo: u16,
        subgenre: MetalSubgenre,
    ) -> Vec<f32> {
        let beat_duration = 60.0 / tempo as f32;
        let intensity = section_type.intensity();
        // CRITICAL: Get the rhythmic feel (HalfTime/Normal/Blast) from the section
        let rhythmic_feel = section_type.rhythmic_feel();

        let mut section_audio = Vec::new();

        // 1. THE DROP: Add an aggressive kick drop for breakdowns
        if matches!(section_type, MetalSection::Breakdown) {
            let silence_duration = 0.5; // Shorter silence before the drop
            let silence_samples = (silence_duration * self.sample_rate as f32) as usize;
            
            // Generate Heavy Drop Kick
            let drop_kick = generate_drop_kick();
            let mut transition = vec![0.0; silence_samples];
            
            // Add the drop kick after the silence
            transition.extend(drop_kick);
            
            section_audio.extend(transition);
            println!("💥 THE DROP: Heavy kick drop triggered");
        }

        // 2. Render Guitar (Keeps Song Tempo - Guitars still chug on grid)
        let guitar_audio = self.render_guitar_riff(riff, beat_duration);
        
        // 3. Render Drums (Decoupled Tempo based on RhythmicFeel)
        let (kick_pattern, _, _) = self.generate_drum_patterns(section_type, duration, tempo, subgenre, rhythmic_feel);
        
        let drum_audio = self.render_drums(section_type, duration, tempo, subgenre, rhythmic_feel);

        // 4. Render Bass (Locks to Kick OR Guitar depending on density)
        // If it's a breakdown, bass matches the sparse kick (Lock mode)
        let bass_mode = if section_type == MetalSection::Breakdown {
            BassMode::Lock 
        } else {
            crate::composition::bass_generator::MetalBassGenerator::mode_for_subgenre(subgenre)
        };

        // For breakdowns, bass notes are loooong (Quarter notes)
        let bass_note_duration = if section_type == MetalSection::Breakdown {
            beat_duration 
        } else {
            beat_duration / 4.0 // 16th note bass
        };

        let bass_audio = self.render_bass_riff_locked(
            &riff.notes,
            &kick_pattern,
            bass_note_duration,
            bass_mode,
            &riff,
        );

        // 5. Dynamic Mixing (Turn down instruments to avoid clipping/noise)
        let (guitar_level, bass_level, drum_level) = match intensity {
            SectionIntensity::Low => (0.35, 0.40, 0.50),
            SectionIntensity::Medium => (0.40, 0.45, 0.60),
            SectionIntensity::High => (0.45, 0.50, 0.65),
            SectionIntensity::Extreme => (0.50, 0.55, 0.70), // Louder, but safe
        };

        let max_len = guitar_audio.len().max(bass_audio.len()).max(drum_audio.len());
        section_audio.resize(section_audio.len() + max_len, 0.0);
        let offset = section_audio.len() - max_len;

        for i in 0..max_len {
            let guitar = if i < guitar_audio.len() { guitar_audio[i] } else { 0.0 };
            let bass = if i < bass_audio.len() { bass_audio[i] } else { 0.0 };
            let drums = if i < drum_audio.len() { drum_audio[i] } else { 0.0 };
            
            section_audio[offset + i] = guitar * guitar_level + bass * bass_level + drums * drum_level;
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

    /// Generate drum patterns based on RhythmicFeel (Tempo Decoupling)
    fn generate_drum_patterns(
        &self,
        section: MetalSection,
        duration: f32,
        tempo: u16,
        subgenre: MetalSubgenre,
        feel: RhythmicFeel,
    ) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
        let _ = section; 
        let beat_duration = 60.0 / tempo as f32;
        let sixteenth_duration = beat_duration / 4.0; 
        
        let steps = (duration / sixteenth_duration).ceil() as usize;
        
        let mut kick = vec![false; steps];
        let mut snare = vec![false; steps];
        let mut cymbal = vec![false; steps];

        match feel {
            RhythmicFeel::HalfTime => {
                // HALF TIME LOGIC (Breakdowns)
                // Snare on beat 3 (Step 8 in a 0-15 grid)
                // Kick is sparse, Euclidean pulses reduced
                let pulses = 3; 
                kick = rhythm_generator::generate_euclidean_pattern(steps, pulses);
                
                for i in 0..steps {
                    // Snare on beat 3 (every 16 steps, offset 8)
                    if i % 16 == 8 { 
                        snare[i] = true; 
                        kick[i] = false; // Don't kick on snare
                    }
                    // China/Crash on beat 1
                    if i % 16 == 0 { cymbal[i] = true; kick[i] = true; }
                }
            },
            RhythmicFeel::DoubleTime | RhythmicFeel::Blast => {
                // BLAST LOGIC
                // Every 2nd step (8th note at high tempo)
                for i in 0..steps {
                    if i % 2 == 0 {
                        kick[i] = true;
                        snare[i] = true; // Unison blast
                        cymbal[i] = true;
                    }
                }
            },
            RhythmicFeel::Normal => {
                // STANDARD METAL
                // Snare on 2 and 4 (Steps 4 and 12)
                let pulses = if matches!(subgenre, MetalSubgenre::ProgressiveMetal) { 7 } else { 5 };
                kick = rhythm_generator::generate_euclidean_pattern(steps, pulses);
                
                for i in 0..steps {
                    if i % 16 == 4 || i % 16 == 12 {
                        snare[i] = true;
                        kick[i] = false; // Clear kick for snare
                    }
                    // Sparse cymbals: only on beat 1 of each bar (every 16th step)
                    if i % 16 == 0 {
                        cymbal[i] = true;
                        kick[i] = true;
                    }
                }
            },
        }
        
        (kick, snare, cymbal)
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
                    // Render Minor chord (Root, 3rd, 5th)
                    let root_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordRoot);
                    
                    // Render 3rd (+3 semitones)
                    let freq_3rd = 440.0 * 2.0_f32.powf(((note + 3) as f32 - 69.0) / 12.0);
                    let third_samples = generate_metal_guitar_note(freq_3rd, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordThird);
                    
                    // Render 5th (+7 semitones)
                    let freq_5th = 440.0 * 2.0_f32.powf(((note + 7) as f32 - 69.0) / 12.0);
                    let fifth_samples = generate_metal_guitar_note(freq_5th, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordFifth);
                    
                    // Mix
                    let max_len = root_samples.len().max(third_samples.len()).max(fifth_samples.len());
                    note_samples.resize(max_len, 0.0);
                    
                    for j in 0..max_len {
                        let s1 = if j < root_samples.len() { root_samples[j] } else { 0.0 };
                        let s2 = if j < third_samples.len() { third_samples[j] } else { 0.0 };
                        let s3 = if j < fifth_samples.len() { fifth_samples[j] } else { 0.0 };
                        note_samples[j] = s1 * 0.4 + s2 * 0.3 + s3 * 0.3;
                    }
                },
                ChordType::Diminished | ChordType::Octave => {
                    // Fallback to single note for unsupported chord types
                    note_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::SingleNote);
                },
                ChordType::Single => {
                    // Render single note
                    note_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::SingleNote);
                },
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
            // Process through DSP chain
            let processed_sample = self.dsp_chain.process(sample);
            
            // Cabinet Simulation (speaker coloration)
            let final_sample = self.cabinet.process(processed_sample);
            
            processed.push(final_sample);
        }

        processed
    }

    fn render_drums(&self, section: MetalSection, duration: f32, tempo: u16, subgenre: MetalSubgenre, feel: RhythmicFeel) -> Vec<f32> {
        let sample_rate = self.sample_rate as f32;
        let num_samples = (duration * sample_rate) as usize;
        let mut drum_audio = vec![0.0; num_samples];
        
        let beat_duration = 60.0 / tempo as f32;
        let sixteenth_duration = beat_duration / 4.0;
        
        // Pass 'feel' to pattern generator
        let (kick_pattern, snare_pattern, cymbal_pattern) = self.generate_drum_patterns(section, duration, tempo, subgenre, feel);

        // Render loop
        for i in 0..kick_pattern.len() {
            let base_time = i as f32 * sixteenth_duration;
            let sample_idx = (base_time * sample_rate) as usize;
            
            if sample_idx >= num_samples { break; }

            // Velocity logic...
            let velocity = 0.9; // Less than 1.0 to prevent clipping

            if kick_pattern[i] {
                let kick_sound = self.drums.generate_kick(velocity);
                self.mix_drum_hit(&mut drum_audio, &kick_sound, sample_idx);
            }
            if snare_pattern[i] {
                let snare_sound = self.drums.generate_snare(velocity);
                self.mix_drum_hit(&mut drum_audio, &snare_sound, sample_idx);
            }
            if cymbal_pattern[i] {
                let crash_sound = self.drums.generate_crash(velocity * 0.8);
                self.mix_drum_hit(&mut drum_audio, &crash_sound, sample_idx);
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

    /// Normalize audio buffer using soft clipping limiter
    fn apply_limiter(samples: &mut [f32], threshold: f32) {
        for sample in samples.iter_mut() {
            if *sample > threshold {
                *sample = threshold + (*sample - threshold).tanh() * 0.1;
            } else if *sample < -threshold {
                *sample = -threshold + (*sample + threshold).tanh() * 0.1;
            }
        }
    }
}

impl Default for MetalAudioRenderer {
    fn default() -> Self {
        Self::new()
    }
}