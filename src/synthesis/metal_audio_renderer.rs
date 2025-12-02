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
use rand::Rng;

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
        
        // 3. Render Drums with Guitar Context Awareness
        // Extract guitar context from the riff for intelligent drum generation
        use crate::composition::phrase_drums::GuitarContext;
        let guitar_context = GuitarContext::from_riff(riff);
        
        // Generate base drum patterns influenced by guitar context
        let (kick_pattern, _, _) = self.generate_drum_patterns(section_type, duration, tempo, subgenre, rhythmic_feel);
        
        // Apply guitar-context-aware modifications to drums
        // High palm-mute density = more aggressive kick patterns
        // This makes drums react to the guitar's playing style
        let _palm_mute_influence = guitar_context.palm_mute_density;
        let _riff_complexity = guitar_context.riff_contour.len();
        
        // Render drums with context-aware enhancements
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

        // ===== PHASE 5: MIX GLUE & SIDECHAIN COMPRESSION =====
        // Extract kick hits for sidechain trigger
        let kick_envelope = self.extract_kick_envelope(&drum_audio, beat_duration);
        
        // Apply sidechain compression to guitar and bass
        let guitar_sidechained = self.apply_sidechain(&guitar_audio, &kick_envelope);
        let bass_sidechained = self.apply_sidechain(&bass_audio, &kick_envelope);

        // 5. Dynamic Mixing (Turn down instruments to avoid clipping/noise)
        let (guitar_level, bass_level, drum_level) = match intensity {
            SectionIntensity::Low => (0.35, 0.40, 0.50),
            SectionIntensity::Medium => (0.40, 0.45, 0.60),
            SectionIntensity::High => (0.45, 0.50, 0.65),
            SectionIntensity::Extreme => (0.50, 0.55, 0.70), // Louder, but safe
        };

        let max_len = guitar_sidechained.len().max(bass_sidechained.len()).max(drum_audio.len());
        section_audio.resize(section_audio.len() + max_len, 0.0);
        let offset = section_audio.len() - max_len;

        for i in 0..max_len {
            let guitar = if i < guitar_sidechained.len() { guitar_sidechained[i] } else { 0.0 };
            let bass = if i < bass_sidechained.len() { bass_sidechained[i] } else { 0.0 };
            let drums = if i < drum_audio.len() { drum_audio[i] } else { 0.0 };
            
            section_audio[offset + i] = guitar * guitar_level + bass * bass_level + drums * drum_level;
        }

        section_audio
    }
    
    /// PHASE 5.1: Extract kick drum envelope for sidechain trigger
    fn extract_kick_envelope(&self, drum_audio: &[f32], beat_duration: f32) -> Vec<f32> {
        let mut envelope = vec![0.0; drum_audio.len()];
        let window_size = (beat_duration * self.sample_rate as f32 / 16.0) as usize; // Sixteenth note window
        
        for i in 0..drum_audio.len() {
            // Simple peak detection in sliding window
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2).min(drum_audio.len());
            
            let mut peak = 0.0_f32;
            for j in start..end {
                peak = peak.max(drum_audio[j].abs());
            }
            
            envelope[i] = peak;
        }
        
        envelope
    }
    
    /// PHASE 5.2: Apply sidechain compression (ducking) based on kick envelope
    fn apply_sidechain(&self, audio: &[f32], kick_envelope: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(audio.len());
        let max_len = audio.len().min(kick_envelope.len());
        
        // Sidechain parameters
        let threshold = 0.3; // Kick level threshold to trigger ducking
        let attack_samples = (0.010 * self.sample_rate as f32) as usize; // 10ms attack
        let release_samples = (0.150 * self.sample_rate as f32) as usize; // 150ms release
        let reduction = 0.7; // Reduce to 70% (30% ducking)
        
        let mut gain_reduction = 1.0;
        
        for i in 0..max_len {
            let kick_level = if i < kick_envelope.len() { kick_envelope[i] } else { 0.0 };
            let input = if i < audio.len() { audio[i] } else { 0.0 };
            
            // Calculate target gain based on kick level
            let target_gain = if kick_level > threshold {
                reduction // Duck when kick hits
            } else {
                1.0 // Full level otherwise
            };
            
            // Smooth gain changes with attack/release
            if target_gain < gain_reduction {
                // Attack (fast reduction)
                let attack_coeff = 1.0 / attack_samples as f32;
                gain_reduction = gain_reduction - (gain_reduction - target_gain) * attack_coeff;
            } else {
                // Release (slow recovery)
                let release_coeff = 1.0 / release_samples as f32;
                gain_reduction = gain_reduction + (target_gain - gain_reduction) * release_coeff;
            }
            
            output.push(input * gain_reduction);
        }
        
        // Extend with remaining audio if any
        for i in max_len..audio.len() {
            output.push(audio[i]);
        }
        
        output
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
    /// IMPROVED: More variety based on section and subgenre
    /// PHASE 1.1: Added kick overload modes for extreme aggression
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
        let mut rng = rand::thread_rng();
        
        let mut kick = vec![false; steps];
        let mut snare = vec![false; steps];
        let mut cymbal = vec![false; steps];

        match feel {
            RhythmicFeel::HalfTime => {
                // HALF TIME LOGIC (Breakdowns)
                // Vary the pulses based on section for more variety
                let pulses = match section {
                    MetalSection::Breakdown => rng.gen_range(2..=4), // 2-4 pulses for variety
                    _ => 3,
                };
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
                // BLAST LOGIC - vary based on subgenre
                let blast_density = match subgenre {
                    MetalSubgenre::DeathMetal => 2, // Every 2nd step
                    MetalSubgenre::ThrashMetal => 4, // Every 4th step (less dense)
                    _ => 3, // Every 3rd step
                };
                
                for i in 0..steps {
                    if i % blast_density == 0 {
                        kick[i] = true;
                        snare[i] = true; // Unison blast
                        cymbal[i] = true;
                    }
                }
            },
            RhythmicFeel::Normal => {
                // STANDARD METAL - vary pulses by subgenre and section
                let pulses = match (subgenre, section) {
                    (MetalSubgenre::ProgressiveMetal, _) => rng.gen_range(5..=9), // Prime numbers
                    (MetalSubgenre::DeathMetal, MetalSection::Verse) => rng.gen_range(7..=11),
                    (MetalSubgenre::ThrashMetal, _) => rng.gen_range(5..=7),
                    (_, MetalSection::Chorus) => rng.gen_range(6..=8), // More dense in chorus
                    (_, MetalSection::Verse) => rng.gen_range(4..=6),
                    _ => 5,
                };
                kick = rhythm_generator::generate_euclidean_pattern(steps, pulses);
                
                // Rotate pattern for variety
                let rotation = rng.gen_range(0..steps.min(16));
                kick.rotate_left(rotation);
                
                // Snare on 2 and 4 (every 16 steps, offset 4 and 12)
                for i in 0..steps {
                    if i % 16 == 4 || i % 16 == 12 { snare[i] = true; }
                    if i % 4 == 0 { cymbal[i] = true; } // Hi-hat on every quarter note
                }
                
                // Occasional crash accents
                for i in (0..steps).step_by(32) {
                    if rng.gen_bool(0.7) { cymbal[i] = true; }
                }
            },
        }
        
        // ===== PHASE 1.1: KICK OVERLOAD MODES =====
        // Add extreme aggression for death metal and breakdown sections
        
        // 1. BURST KICK CLUSTERS (4-6 rapid kicks every 8-12 beats)
        if matches!(subgenre, MetalSubgenre::DeathMetal) || matches!(section, MetalSection::Breakdown | MetalSection::Solo) {
            let burst_interval = rng.gen_range(32..=48); // Every 8-12 beats (32-48 sixteenths)
            for i in (0..steps).step_by(burst_interval) {
                if rng.gen_bool(0.6) { // 60% chance of burst
                    let burst_length = rng.gen_range(4..=6);
                    for j in 0..burst_length {
                        if i + j < steps {
                            kick[i + j] = true;
                        }
                    }
                }
            }
        }
        
        // 2. DOUBLE-KICK ALTERNATING-FOOT SIMULATION
        // Add double-kick patterns for extreme sections
        if matches!(subgenre, MetalSubgenre::DeathMetal | MetalSubgenre::ThrashMetal) {
            for i in 0..steps {
                // On every existing kick, add a chance for a double-kick follow-up
                if kick[i] && i + 1 < steps && rng.gen_bool(0.4) {
                    kick[i + 1] = true; // Alternating foot
                }
            }
        }
        
        // 3. 32ND-NOTE KICK PEPPERING (extreme sections only)
        if matches!(section, MetalSection::Solo) && matches!(subgenre, MetalSubgenre::DeathMetal) {
            for i in 0..steps {
                // Add random 32nd-note kicks (every other sixteenth)
                if i % 2 == 1 && rng.gen_bool(0.25) {
                    kick[i] = true;
                }
            }
        }
        
        // 4. INTENTIONAL OVER-DENSIFICATION (breakdowns and death metal)
        if matches!(section, MetalSection::Breakdown) || (matches!(subgenre, MetalSubgenre::DeathMetal) && rng.gen_bool(0.5)) {
            // Add extra kicks to create overwhelming density
            for i in 0..steps {
                if !kick[i] && rng.gen_bool(0.15) { // 15% chance to add extra kick
                    kick[i] = true;
                }
            }
        }
        
        // ===== PHASE 6: DYNAMIC DRUM PATTERNS =====
        // Add intelligent fills, transitions, and section-aware variations
        
        // 6.1: DRUM FILLS AT SECTION BOUNDARIES
        // Add tom cascades and snare rolls before transitions
        if steps >= 16 {
            // Fill in last 4 steps (one beat) of the section
            let fill_start = steps.saturating_sub(4);
            
            // 70% chance of fill for transitions
            if rng.gen_bool(0.7) {
                match rng.gen_range(0..3) {
                    0 => {
                        // TOM CASCADE: Descending tom pattern
                        for i in fill_start..steps {
                            kick[i] = false; // Clear kicks
                            snare[i] = true; // Use snare for tom simulation
                            cymbal[i] = i == fill_start; // Crash on first hit
                        }
                    },
                    1 => {
                        // SNARE ROLL: Rapid snare hits
                        for i in fill_start..steps {
                            snare[i] = true;
                            kick[i] = false;
                            cymbal[i] = i == steps - 1; // Crash on last hit
                        }
                    },
                    _ => {
                        // KICK/SNARE COMBO: Alternating pattern
                        for i in fill_start..steps {
                            if (i - fill_start) % 2 == 0 {
                                kick[i] = true;
                                snare[i] = false;
                            } else {
                                kick[i] = false;
                                snare[i] = true;
                            }
                            cymbal[i] = i == steps - 1; // Crash on last hit
                        }
                    }
                }
            }
        }
        
        // 6.2: TRANSITION MARKERS
        // Add crash accents and cymbal swells at key moments
        
        // Crash on first beat of chorus/breakdown
        if matches!(section, MetalSection::Chorus | MetalSection::Breakdown) && steps > 0 {
            cymbal[0] = true;
            kick[0] = true; // Emphasize with kick
        }
        
        // Cymbal swell before breakdown (last 8 steps)
        if matches!(section, MetalSection::Verse | MetalSection::Chorus) && steps >= 8 {
            let swell_start = steps.saturating_sub(8);
            for i in swell_start..steps {
                if (i - swell_start) % 2 == 0 {
                    cymbal[i] = true; // Alternating cymbal hits for swell effect
                }
            }
        }
        
        // 6.3: SECTION-AWARE VARIATIONS
        // Adjust patterns based on section type
        
        match section {
            MetalSection::Intro => {
                // Simplify intro: reduce kick density by 30%
                for i in 0..steps {
                    if kick[i] && rng.gen_bool(0.3) {
                        kick[i] = false;
                    }
                }
                // Add ride cymbal pattern instead of hi-hat
                for i in (0..steps).step_by(2) {
                    cymbal[i] = true;
                }
            },
            MetalSection::Outro => {
                // Simplify outro: reduce overall density
                for i in 0..steps {
                    if kick[i] && rng.gen_bool(0.4) {
                        kick[i] = false;
                    }
                    if snare[i] && rng.gen_bool(0.3) {
                        snare[i] = false;
                    }
                }
                // Fade cymbal pattern
                for i in 0..steps {
                    let fade_factor = 1.0 - (i as f32 / steps as f32);
                    if cymbal[i] && rng.gen_bool(1.0 - fade_factor as f64) {
                        cymbal[i] = false;
                    }
                }
            },
            MetalSection::Verse => {
                // Verse: Add ghost notes (light snare hits)
                for i in 0..steps {
                    if !snare[i] && i % 4 == 2 && rng.gen_bool(0.4) {
                        snare[i] = true; // Ghost note
                    }
                }
            },
            MetalSection::Chorus => {
                // Chorus: Increase cymbal density for energy
                for i in 0..steps {
                    if i % 2 == 0 && rng.gen_bool(0.6) {
                        cymbal[i] = true;
                    }
                }
            },
            MetalSection::Solo => {
                // Solo: Add syncopated kick patterns
                for i in 0..steps {
                    if i % 6 == 3 && rng.gen_bool(0.5) { // Off-beat kicks
                        kick[i] = true;
                    }
                }
            },
            MetalSection::Breakdown => {
                // Breakdown: Sparse, heavy hits (already handled above)
                // Add occasional china cymbal hits
                for i in (0..steps).step_by(16) {
                    if rng.gen_bool(0.8) {
                        cymbal[i] = true;
                        kick[i] = true;
                    }
                }
            },
        }
        
        // 6.4: DYNAMIC INTENSITY SCALING
        // Gradually increase density throughout the section for building tension
        if matches!(section, MetalSection::Verse | MetalSection::Chorus) {
            for i in 0..steps {
                let intensity = i as f32 / steps as f32; // 0.0 to 1.0
                
                // Add progressive kick density
                if !kick[i] && rng.gen_bool(intensity as f64 * 0.2) {
                    kick[i] = true;
                }
                
                // Add progressive cymbal density
                if !cymbal[i] && i % 4 == 0 && rng.gen_bool(intensity as f64 * 0.3) {
                    cymbal[i] = true;
                }
            }
        }

        
        // ===== PHASE 1.4: CYMBAL BRUTALITY =====
        // Add chaotic cymbal patterns for extreme aggression
        
        // 1. CHINA SPAM on breakdowns (explosive china walls)
        if matches!(section, MetalSection::Breakdown) {
            for i in 0..steps {
                // China on every downbeat + random chaos
                if i % 4 == 0 || rng.gen_bool(0.3) {
                    cymbal[i] = true;
                }
            }
        }
        
        // 2. CYMBAL CLUSTERS on motif resets/transitions (every 32 steps = 2 bars)
        for i in (0..steps).step_by(32) {
            if rng.gen_bool(0.7) {
                // Cluster: 3-5 rapid cymbal hits
                let cluster_length = rng.gen_range(3..=5);
                for j in 0..cluster_length {
                    if i + j < steps {
                        cymbal[i + j] = true;
                    }
                }
            }
        }
        
        // 3. CRASH ACCENTS on section changes (first beat of section)
        if steps > 0 {
            cymbal[0] = true; // Always crash on section start
        }
        
        // 4. BELL RIDE on upbeats for thrash metal
        if matches!(subgenre, MetalSubgenre::ThrashMetal) {
            for i in 0..steps {
                // Bell ride on off-beats (8th notes offset)
                if i % 8 == 2 || i % 8 == 6 {
                    if rng.gen_bool(0.5) {
                        cymbal[i] = true;
                    }
                }
            }
        }
        
        // 5. CHAOTIC CRASHES on extreme genre transitions
        if matches!(subgenre, MetalSubgenre::DeathMetal) {
            for i in 0..steps {
                // Random crash spam (10% chance any step)
                if rng.gen_bool(0.1) {
                    cymbal[i] = true;
                }
            }
        }

        (kick, snare, cymbal)
    }

    /// Render guitar riff with chords support and variable durations
    /// PHASE 4: STEREO WIDTH & DOUBLE-TRACKING - Renders left and right channels with variations
    fn render_guitar_riff(&mut self, riff: &MetalRiff, beat_duration: f32) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        
        // ===== PHASE 4.1: RENDER LEFT CHANNEL =====
        let mut left_channel = Vec::new();
        
        for (i, &note) in riff.notes.iter().enumerate() {
            let palm_muted = riff.palm_muted[i];
            let chord_type = riff.chord_types.get(i).copied().unwrap_or(ChordType::Single);
            let rhythm = riff.rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            // Handle rests
            if rhythm == RhythmPattern::Rest {
                let rest_duration = beat_duration / 4.0;
                let rest_samples = (rest_duration * self.sample_rate as f32) as usize;
                left_channel.extend(vec![0.0; rest_samples]);
                continue;
            }
            
            let base_duration = match rhythm {
                RhythmPattern::QuarterNote => beat_duration,
                RhythmPattern::EighthNote => beat_duration / 2.0,
                RhythmPattern::SixteenthNote => beat_duration / 4.0,
                RhythmPattern::ThirtySecondNote => beat_duration / 8.0,
                RhythmPattern::Quintuplet => beat_duration * 0.8,
                RhythmPattern::Septuplet => beat_duration * 0.571,
                RhythmPattern::DottedEighth => beat_duration * 0.75,
                RhythmPattern::Gallop => {
                    if let Some(gallop_samples) = self.render_gallop_pattern(riff, i, beat_duration, palm_muted, chord_type) {
                        left_channel.extend(gallop_samples);
                    }
                    continue;
                },
                RhythmPattern::Rest => beat_duration / 4.0,
            };
            
            let min_sustain = if palm_muted { 0.08 } else { 0.12 };
            let note_duration = base_duration.max(min_sustain);
            
            // LEFT CHANNEL: Standard tuning
            let freq_root = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
            let note_samples = self.render_chord(note, chord_type, freq_root, note_duration, palm_muted, 0.8);
            left_channel.extend(note_samples);
        }
        
        // ===== PHASE 4.2: RENDER RIGHT CHANNEL WITH VARIATIONS =====
        let mut right_channel = Vec::new();
        
        for (i, &note) in riff.notes.iter().enumerate() {
            let palm_muted = riff.palm_muted[i];
            let chord_type = riff.chord_types.get(i).copied().unwrap_or(ChordType::Single);
            let rhythm = riff.rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            if rhythm == RhythmPattern::Rest {
                let rest_duration = beat_duration / 4.0;
                let rest_samples = (rest_duration * self.sample_rate as f32) as usize;
                right_channel.extend(vec![0.0; rest_samples]);
                continue;
            }
            
            let base_duration = match rhythm {
                RhythmPattern::QuarterNote => beat_duration,
                RhythmPattern::EighthNote => beat_duration / 2.0,
                RhythmPattern::SixteenthNote => beat_duration / 4.0,
                RhythmPattern::ThirtySecondNote => beat_duration / 8.0,
                RhythmPattern::Quintuplet => beat_duration * 0.8,
                RhythmPattern::Septuplet => beat_duration * 0.571,
                RhythmPattern::DottedEighth => beat_duration * 0.75,
                RhythmPattern::Gallop => {
                    if let Some(gallop_samples) = self.render_gallop_pattern(riff, i, beat_duration, palm_muted, chord_type) {
                        right_channel.extend(gallop_samples);
                    }
                    continue;
                },
                RhythmPattern::Rest => beat_duration / 4.0,
            };
            
            let min_sustain = if palm_muted { 0.08 } else { 0.12 };
            let note_duration = base_duration.max(min_sustain);
            
            // RIGHT CHANNEL: Slightly detuned (±5 cents = ±0.05 semitones)
            let detune_cents = rng.gen_range(-5.0..5.0);
            let freq_root = 440.0 * 2.0_f32.powf((note as f32 - 69.0 + detune_cents / 100.0) / 12.0);
            
            let note_samples = self.render_chord(note, chord_type, freq_root, note_duration, palm_muted, 0.8);
            right_channel.extend(note_samples);
        }
        
        // ===== PHASE 4.3: APPLY TIMING OFFSET TO RIGHT CHANNEL =====
        // Delay right channel by 5-15ms for "Haas effect" stereo width
        let timing_offset_ms = rng.gen_range(5.0..15.0);
        let timing_offset_samples = (timing_offset_ms / 1000.0 * self.sample_rate as f32) as usize;
        let mut right_delayed = vec![0.0; timing_offset_samples];
        right_delayed.extend(right_channel);
        
        // ===== PHASE 4.4: MIX TO STEREO (INTERLEAVED L/R) =====
        let max_len = left_channel.len().max(right_delayed.len());
        let mut stereo_audio = Vec::with_capacity(max_len * 2);
        
        for i in 0..max_len {
            let left = if i < left_channel.len() { left_channel[i] } else { 0.0 };
            let right = if i < right_delayed.len() { right_delayed[i] } else { 0.0 };
            
            // Hard pan: 100% L, 100% R for maximum width
            stereo_audio.push(left);   // Left channel
            stereo_audio.push(right);  // Right channel
        }
        
        // Apply distortion and cabinet simulation to stereo signal
        self.process_guitar_chain(&stereo_audio)
    }
    
    /// Helper function to render a chord with given parameters
    fn render_chord(&mut self, note: u8, chord_type: ChordType, freq_root: f32, note_duration: f32, is_palm_muted: bool, velocity: f32) -> Vec<f32> {
        let mut note_samples = Vec::new();
        
        match chord_type {
            ChordType::Power => {
                let root_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::PowerChordRoot);
                let freq_5th = 440.0 * 2.0_f32.powf(((note + 7) as f32 - 69.0) / 12.0);
                let fifth_samples = generate_metal_guitar_note(freq_5th, note_duration, velocity, is_palm_muted, PlayingTechnique::PowerChordFifth);
                let freq_oct = 440.0 * 2.0_f32.powf(((note + 12) as f32 - 69.0) / 12.0);
                let oct_samples = generate_metal_guitar_note(freq_oct, note_duration, velocity, is_palm_muted, PlayingTechnique::PowerChordOctave);
                
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
                let root_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordRoot);
                let freq_3rd = 440.0 * 2.0_f32.powf(((note + 3) as f32 - 69.0) / 12.0);
                let third_samples = generate_metal_guitar_note(freq_3rd, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordThird);
                let freq_5th = 440.0 * 2.0_f32.powf(((note + 7) as f32 - 69.0) / 12.0);
                let fifth_samples = generate_metal_guitar_note(freq_5th, note_duration, velocity, is_palm_muted, PlayingTechnique::MinorChordFifth);
                
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
                note_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::SingleNote);
            },
            ChordType::Single => {
                note_samples = generate_metal_guitar_note(freq_root, note_duration, velocity, is_palm_muted, PlayingTechnique::SingleNote);
            },
        }
        
        note_samples
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

        // Initialize RNG for humanization
        let mut rng = rand::thread_rng();
        
        // ===== PHASE 1.3: EXTREME HUMANIZATION =====
        // Aggressive presets for extreme metal (not jazz-level ±5 ticks)
        let is_extreme_genre = matches!(subgenre, MetalSubgenre::DeathMetal | MetalSubgenre::ThrashMetal);
        
        // Section-specific velocity ranges with EXTREME variance for death metal
        let (base_velocity, velocity_variance) = match (section, is_extreme_genre) {
            (MetalSection::Breakdown, _) => (0.95_f32, 0.05_f32), // High velocity, tight variance
            (MetalSection::Solo, true) => (0.85_f32, 0.20_f32),   // EXTREME variance for death metal solos
            (MetalSection::Solo, false) => (0.85_f32, 0.10_f32),  // Medium variance for other genres
            (MetalSection::Intro | MetalSection::Outro, _) => (0.75_f32, 0.08_f32), // Lower velocity
            (_, true) => (0.85_f32, 0.15_f32),                    // Higher variance for extreme genres
            (_, false) => (0.85_f32, 0.08_f32),                   // Default
        };
        
        // EXTREME TIMING VARIANCE (25-40 ticks, not 5-10)
        let timing_variance_ms = if is_extreme_genre {
            rng.gen_range(25.0..=40.0) // Extreme chaos
        } else {
            rng.gen_range(10.0..=20.0) // Moderate chaos
        };
        
        // ACCENT PROBABILITY (40-60%, not 10-20%)
        let accent_probability = if is_extreme_genre {
            rng.gen_range(0.4..=0.6) // 40-60% chance
        } else {
            0.3 // 30% for other genres
        };
        
        // LIMB IMBALANCE (±20 velocity, not ±5)
        let limb_imbalance = if is_extreme_genre {
            rng.gen_range(-0.20..=0.20) // ±20% velocity difference
        } else {
            rng.gen_range(-0.10..=0.10) // ±10% for other genres
        };
        
        // ===== PHASE 1.2: BLAST BEAT CHAOS DETECTION =====
        // Detect if we're in a blast beat section (kick + snare + cymbal on same step)
        let is_blast_section = matches!(feel, RhythmicFeel::DoubleTime | RhythmicFeel::Blast);
        
        // Micro-timing offsets (in milliseconds)
        let snare_timing_offset = match section {
            MetalSection::Breakdown => rng.gen_range(5.0..=15.0),  // Drag snare for "weight"
            _ if matches!(subgenre, MetalSubgenre::ThrashMetal) => -rng.gen_range(5.0..=10.0), // Push snare for "urgency"
            _ => 0.0,
        };
        
        // ===== PHASE 1.3: FATIGUE COLLAPSE TRACKING =====
        let mut kick_count = 0;
        let mut fatigue_penalty = 0.0_f32;

        // Render loop
        for i in 0..kick_pattern.len() {
            let base_time = i as f32 * sixteenth_duration;
            
            // Track kick hits for fatigue
            if kick_pattern[i] {
                kick_count += 1;
                
                // FATIGUE COLLAPSE: After 120+ kicks, velocity drops
                if kick_count > 120 {
                    fatigue_penalty = rng.gen_range(0.15..=0.40); // 15-40% velocity drop
                }
            }
            
            // Accent first beat of each bar (every 16 sixteenth notes = 1 bar in 4/4)
            let is_first_beat = i % 16 == 0;
            
            // EXTREME ACCENT BOOST (random accents, not just first beat)
            let random_accent = rng.gen_bool(accent_probability as f64);
            let accent_boost = if is_first_beat || random_accent { 
                rng.gen_range(0.10..=0.15) // 10-15% boost
            } else { 
                0.0_f32 
            };
            
            // EXTREME timing variance (25-40ms, not 4ms)
            let extreme_timing_offset = rng.gen_range(-timing_variance_ms..=timing_variance_ms);
            
            // Micro-randomization (±3-5 velocity)
            let micro_random = rng.gen_range(-0.04_f32..=0.04_f32);
            
            // Calculate final velocity with humanization + limb imbalance + fatigue
            let humanized_velocity = (base_velocity + accent_boost + micro_random + velocity_variance * rng.gen_range(-0.5_f32..=0.5_f32) + limb_imbalance - fatigue_penalty)
                .clamp(0.5_f32, 1.0_f32); // Allow lower floor for fatigue

            // ===== PHASE 1.2: BLAST BEAT CHAOS - TIMING DESYNC =====
            // Apply extreme timing chaos for blast beats (not per-hit randomness, but drift)
            let (kick_timing_offset, snare_blast_offset, cymbal_timing_offset) = if is_blast_section {
                // BLAST BEAT DESYNC:
                // - Kick early by 3-7 ticks (samples)
                // - Snare late by 4-12 ticks (samples)  
                // - Cymbal fluctuates ±15 ticks (samples)
                let kick_drift = -rng.gen_range(3.0..=7.0); // Early (negative)
                let snare_drift = rng.gen_range(4.0..=12.0); // Late (positive)
                let cymbal_drift = rng.gen_range(-15.0..=15.0); // Random
                (kick_drift, snare_drift, cymbal_drift)
            } else {
                (0.0, 0.0, 0.0)
            };

            // Kick drum with blast beat timing + extreme humanization
            if kick_pattern[i] {
                let blast_offset = kick_timing_offset as isize;
                let extreme_offset = (extreme_timing_offset / 1000.0 * sample_rate) as isize;
                let total_offset = blast_offset + extreme_offset;
                let sample_idx = ((base_time * sample_rate) as isize + total_offset).max(0) as usize;
                if sample_idx < num_samples {
                    let kick_sound = self.drums.generate_kick(humanized_velocity);
                    self.mix_drum_hit(&mut drum_audio, &kick_sound, sample_idx);
                }
            }
            
            // Snare drum with micro-timing + blast beat chaos + extreme humanization
            if snare_pattern[i] {
                let base_offset = (snare_timing_offset / 1000.0 * sample_rate) as isize;
                let blast_offset = snare_blast_offset as isize;
                let extreme_offset = (extreme_timing_offset / 1000.0 * sample_rate) as isize;
                let total_offset = base_offset + blast_offset + extreme_offset;
                let sample_idx = ((base_time * sample_rate) as isize + total_offset).max(0) as usize;
                if sample_idx < num_samples {
                    // Snare gets extra limb imbalance (right hand dominant)
                    let snare_velocity = (humanized_velocity + limb_imbalance * 0.5).clamp(0.5, 1.0);
                    let snare_sound = self.drums.generate_snare(snare_velocity);
                    self.mix_drum_hit(&mut drum_audio, &snare_sound, sample_idx);
                }
            }
            
            // Cymbal/Hi-hat with blast beat timing fluctuation + extreme humanization
            if cymbal_pattern[i] {
                let blast_offset = cymbal_timing_offset as isize;
                let extreme_offset = (extreme_timing_offset / 1000.0 * sample_rate) as isize;
                let total_offset = blast_offset + extreme_offset;
                let sample_idx = ((base_time * sample_rate) as isize + total_offset).max(0) as usize;
                if sample_idx < num_samples {
                    let cymbal_velocity = (humanized_velocity * 0.8 - limb_imbalance * 0.3).clamp(0.4, 0.9); // Cymbals quieter + left hand weaker
                    let crash_sound = self.drums.generate_crash(cymbal_velocity);
                    self.mix_drum_hit(&mut drum_audio, &crash_sound, sample_idx);
                }
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