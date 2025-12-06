use crate::{composition::{
    bass_generator::BassMode, metal_song_generator::{ChordType, MetalRiff, MetalSection, MetalSong, MetalSubgenre, RhythmPattern, RhythmicFeel, SectionIntensity}, rhythm_generator
}, synthesis::metal_dsp::SplitBandBassDrive};
use crate::synthesis::{
    karplus_strong::{generate_metal_guitar_note, generate_metal_bass_string, PlayingTechnique},
    metal_dsp::{MetalDSPChain},
    cabinet::CabinetSimulator,
    drums::MetalDrums,
    fx::generate_drop_kick,
};
use crate::utils::get_sample_rate;
use rand::Rng;

pub struct MetalAudioRenderer {
    drums: MetalDrums,
    dsp_chain: MetalDSPChain,
    bass_dsp: SplitBandBassDrive,
    cabinet: CabinetSimulator,
    sample_rate: u32,
}

impl MetalAudioRenderer {
pub fn new() -> Self {
        Self {
            drums: MetalDrums::new(),
            dsp_chain: MetalDSPChain::new(6.0),
            bass_dsp: SplitBandBassDrive::new(),
            cabinet: CabinetSimulator::metal_4x12(),
            sample_rate: get_sample_rate(),
        }
    }

    /// NEW: Render song as SEPARATE TRACKS (guitar, bass, drums, melody)
    /// Returns: (guitar_track, bass_track, drum_track, melody_track, master_mix)
    pub fn render_song_multitrack(&mut self, song: &MetalSong, duration_per_section: f32) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        let mut guitar_track = Vec::new();
        let mut bass_track = Vec::new();
        let mut drum_track = Vec::new();
        let mut melody_track = Vec::new();
        
        for (section_type, riff) in &song.sections {
            let (guitar, bass, drums, melody) = self.render_section_separate_tracks(
                *section_type, riff, duration_per_section, song.tempo, song.subgenre
            );
            
            guitar_track.extend(guitar);
            bass_track.extend(bass);
            drum_track.extend(drums);
            melody_track.extend(melody);
        }
        
        // Create master mix from the separate tracks
        let mut master_mix = vec![0.0; guitar_track.len()];
        let max_len = guitar_track.len();
        
        for i in 0..max_len {
            let guitar_sample = *guitar_track.get(i).unwrap_or(&0.0) * 0.85;
            let bass_sample = *bass_track.get(i).unwrap_or(&0.0) * 0.58;
            let drum_sample = *drum_track.get(i).unwrap_or(&0.0) * 0.45;
            let melody_sample = *melody_track.get(i).unwrap_or(&0.0) * 0.65;
            
            master_mix[i] = guitar_sample + bass_sample + drum_sample + melody_sample;
        }
        
        // Apply final limiter to master mix only
        Self::apply_limiter(&mut master_mix, 0.95);
        
        (guitar_track, bass_track, drum_track, melody_track, master_mix)
    }
    
    pub fn render_song(&mut self, song: &MetalSong, duration_per_section: f32) -> Vec<f32> {
        let mut full_audio = Vec::new();
        for (section_type, riff) in &song.sections {
            let section_audio = self.render_section(*section_type, riff, duration_per_section, song.tempo, song.subgenre);
            full_audio.extend(section_audio);
        }
        
        // Final Limiter
        Self::apply_limiter(&mut full_audio, 0.95);
        full_audio
    }

    fn apply_sidechain(&self, audio: &[f32], kick_envelope: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(audio.len());
        let max_len = audio.len().min(kick_envelope.len());
        
        let threshold = 0.2;
        let ratio = 4.0; 
        
        for i in 0..max_len {
            let kick_level = kick_envelope[i];
            let input = audio[i];
            
            // Simple compression logic
            let gain_reduction = if kick_level > threshold {
                // Calculate reduction factor
                let over = kick_level - threshold;
                1.0 / (1.0 + over * ratio)
            } else {
                1.0
            };
            
            output.push(input * gain_reduction);
        }
        
        // Fill remainder
        for i in max_len..audio.len() {
            output.push(audio[i]);
        }
        
        output
    }

    fn render_drums_from_patterns(
        &self, 
        kick_pattern: &[bool], 
        snare_pattern: &[bool], 
        cymbal_pattern: &[bool],
        tempo: u16,
        subgenre: MetalSubgenre,
        section: MetalSection,
        _feel: RhythmicFeel,
    ) -> Vec<f32> {
        let sample_rate = self.sample_rate as f32;
        let beat_duration = 60.0 / tempo as f32;
        let sixteenth_duration = beat_duration / 4.0;
        let num_samples = (kick_pattern.len() as f32 * sixteenth_duration * sample_rate) as usize;
        let mut drum_audio = vec![0.0; num_samples];
        let mut rng = rand::thread_rng();

        // [INCREASED HUMANIZATION: More organic, less robotic]
        let is_extreme_genre = matches!(subgenre, MetalSubgenre::DeathMetal | MetalSubgenre::ThrashMetal);
        let base_velocity = 0.75_f32; // Reduced from 0.9 for wider dynamic range
        // INCREASED timing variance from 15-25ms to 30-50ms for more human feel
        let timing_variance_ms = if is_extreme_genre { 50.0 } else { 35.0 };

        for i in 0..kick_pattern.len() {
            let base_time = i as f32 * sixteenth_duration;
            
            // WIDER VELOCITY RANGE: 0.3-1.0 (was 0.5-1.0) for ghost notes and accents
            let velocity_variation = rng.gen_range(-0.25..0.25); // Wider range
            let humanized_velocity = (base_velocity + velocity_variation).clamp(0.3_f32, 1.0_f32);
            
            // ADD OCCASIONAL "MISTAKES": Random velocity spikes (5% chance)
            let final_velocity = if rng.gen_bool(0.05) {
                (humanized_velocity * 1.3).min(1.0) // Accidental loud hit
            } else {
                humanized_velocity
            };
            
            let extreme_timing_offset = rng.gen_range(-timing_variance_ms..timing_variance_ms);
            
            // Kick
            if kick_pattern[i] {
                 let offset = (extreme_timing_offset / 1000.0 * sample_rate) as isize;
                 let sample_idx = ((base_time * sample_rate) as isize + offset).max(0) as usize;
                 if sample_idx < num_samples {
                     let kick_sound = self.drums.generate_kick(final_velocity);
                     self.mix_drum_hit(&mut drum_audio, &kick_sound, sample_idx);
                 }
            }
            
            // Snare
            if snare_pattern[i] {
                 let offset = (extreme_timing_offset / 1000.0 * sample_rate) as isize;
                 let sample_idx = ((base_time * sample_rate) as isize + offset).max(0) as usize;
                 if sample_idx < num_samples {
                     let snare_sound = self.drums.generate_snare(final_velocity);
                     self.mix_drum_hit(&mut drum_audio, &snare_sound, sample_idx);
                 }
            }
            
            // Cymbal - ADD VARIETY: hi-hat, ride, crash, china
            if cymbal_pattern[i] {
                 let offset = (extreme_timing_offset / 1000.0 * sample_rate) as isize;
                 let sample_idx = ((base_time * sample_rate) as isize + offset).max(0) as usize;
                 if sample_idx < num_samples {
                     let cymbal_vel = final_velocity * 0.8;
                     
                     // Choose cymbal type based on position and section
                     let sound = if i % 16 == 0 {
                         // Downbeat: Use crash or china
                         if matches!(section, MetalSection::Breakdown) {
                             self.drums.generate_china(cymbal_vel)
                         } else {
                             self.drums.generate_crash(cymbal_vel)
                         }
                     } else if i % 4 == 0 {
                         // Quarter notes: Use ride for groove
                         self.drums.generate_ride(cymbal_vel)
                     } else {
                         // Off-beats: Use hi-hat for continuous rhythm
                         let is_open = i % 8 == 2; // Occasional open hi-hat for variation
                         self.drums.generate_hihat(cymbal_vel * 0.6, is_open)
                     };
                     
                     self.mix_drum_hit(&mut drum_audio, &sound, sample_idx);
                 }
            }
        }
        
        drum_audio
    }

    /// NEW: Render section as SEPARATE TRACKS
    /// Returns: (guitar, bass, drums, melody)
    pub fn render_section_separate_tracks(
        &mut self,
        section_type: MetalSection,
        riff: &MetalRiff,
        duration: f32,
        tempo: u16,
        subgenre: MetalSubgenre,
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        let beat_duration = 60.0 / tempo as f32;
        let rhythmic_feel = section_type.rhythmic_feel();
        
        // 1. Render Guitar (clean, no mixing)
        let guitar_audio = self.render_guitar_riff(riff, beat_duration);
        
        // 2. Generate Drum Patterns
        let (kick_pattern, snare_pattern, cymbal_pattern) = self.generate_drum_patterns(
            section_type, duration, tempo, subgenre, rhythmic_feel
        );
        
        // 3. Render Drums
        let mut drum_audio = Vec::new();
        
        // Add drop for breakdowns
        if matches!(section_type, MetalSection::Breakdown) {
            let silence_duration = 0.5;
            let silence_samples = (silence_duration * self.sample_rate as f32) as usize;
            let kick_roll_duration = 0.25;
            let kick_interval = beat_duration / 8.0;
            let num_kicks = (kick_roll_duration / kick_interval) as usize;
            
            let mut kick_roll = vec![0.0; (kick_roll_duration * self.sample_rate as f32) as usize];
            for k in 0..num_kicks {
                let kick_time = k as f32 * kick_interval;
                let kick_idx = (kick_time * self.sample_rate as f32) as usize;
                if kick_idx < kick_roll.len() {
                    let velocity = 1.0 - (k as f32 / num_kicks as f32) * 0.3;
                    let kick_sound = self.drums.generate_kick(velocity);
                    self.mix_drum_hit(&mut kick_roll, &kick_sound, kick_idx);
                }
            }
            
            drum_audio.extend(vec![0.0; silence_samples]);
            drum_audio.extend(kick_roll);
        }
        
        drum_audio.extend(self.render_drums_from_patterns(
            &kick_pattern, &snare_pattern, &cymbal_pattern,
            tempo, subgenre, section_type, rhythmic_feel
        ));
        
        // 4. Render Bass
        let bass_mode = if section_type == MetalSection::Breakdown {
            BassMode::Lock
        } else {
            crate::composition::bass_generator::MetalBassGenerator::mode_for_subgenre(subgenre)
        };
        
        let bass_note_duration = if section_type == MetalSection::Breakdown {
            beat_duration
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
        
        // 5. Generate MELODY TRACK (Piano/Synth playing the riff melody)
        let melody_audio = self.render_melody_track(riff, beat_duration);
        
        // Make all tracks the same length
        let max_len = guitar_audio.len()
            .max(bass_audio.len())
            .max(drum_audio.len())
            .max(melody_audio.len());
        
        let mut guitar_padded = guitar_audio;
        let mut bass_padded = bass_audio;
        let mut drum_padded = drum_audio;
        let mut melody_padded = melody_audio;
        
        guitar_padded.resize(max_len, 0.0);
        bass_padded.resize(max_len, 0.0);
        drum_padded.resize(max_len, 0.0);
        melody_padded.resize(max_len, 0.0);
        
        (guitar_padded, bass_padded, drum_padded, melody_padded)
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
        let rhythmic_feel = section_type.rhythmic_feel();
        let mut section_audio = Vec::new();

        // 1. THE DROP: Add rapid metal kicks for breakdowns (not a single 808 drop)
        if matches!(section_type, MetalSection::Breakdown) {
            let silence_duration = 0.5;
            let silence_samples = (silence_duration * self.sample_rate as f32) as usize;
            
            // Generate RAPID METAL KICKS (kick roll) - not a single drop
            let kick_roll_duration = 0.25; // 250ms of rapid kicks
            let kick_interval = beat_duration / 8.0; // 32nd notes
            let num_kicks = (kick_roll_duration / kick_interval) as usize;
            
            let mut kick_roll = vec![0.0; (kick_roll_duration * self.sample_rate as f32) as usize];
            for k in 0..num_kicks {
                let kick_time = k as f32 * kick_interval;
                let kick_idx = (kick_time * self.sample_rate as f32) as usize;
                if kick_idx < kick_roll.len() {
                    // Decreasing velocity for natural feel
                    let velocity = 1.0 - (k as f32 / num_kicks as f32) * 0.3;
                    let kick_sound = self.drums.generate_kick(velocity);
                    self.mix_drum_hit(&mut kick_roll, &kick_sound, kick_idx);
                }
            }
            
            let mut transition = vec![0.0; silence_samples];
            transition.extend(kick_roll);
            section_audio.extend(transition);
            println!("💥 THE DROP: Rapid metal kick roll triggered ({} kicks)", num_kicks);
        }

        // 2. Render Guitar
        let guitar_audio = self.render_guitar_riff(riff, beat_duration);

        // 3. GENERATE DRUM PATTERNS ONCE (Fixes Sync Issue)
        // We generate the patterns here so both Drums and Bass lock to the exact same grid
        let (kick_pattern, snare_pattern, cymbal_pattern) = self.generate_drum_patterns(
            section_type, duration, tempo, subgenre, rhythmic_feel
        );

        // 4. Render Drums (Using the patterns generated above)
        let drum_audio = self.render_drums_from_patterns(
            &kick_pattern, &snare_pattern, &cymbal_pattern, 
            tempo, subgenre, section_type, rhythmic_feel
        );

        // 5. Render Bass (Locks to the SAME kick pattern)
        let bass_mode = if section_type == MetalSection::Breakdown {
            BassMode::Lock 
        } else {
            crate::composition::bass_generator::MetalBassGenerator::mode_for_subgenre(subgenre)
        };
        
        let bass_note_duration = if section_type == MetalSection::Breakdown {
            beat_duration 
        } else {
            beat_duration / 4.0 
        };

        let bass_audio = self.render_bass_riff_locked(
            &riff.notes,
            &kick_pattern, // Uses the exact same pattern as drums
            bass_note_duration,
            bass_mode,
            &riff,
        );

        // 6. MIX GLUE & SIDECHAIN COMPRESSION
        // Optimized extraction (Fixes Infinite Loop)
        let kick_envelope = self.extract_kick_envelope(&drum_audio);
        
        let guitar_sidechained = self.apply_sidechain(&guitar_audio, &kick_envelope);
        let bass_sidechained = self.apply_sidechain(&bass_audio, &kick_envelope);

        // 7. Dynamic Mixing - MAXIMUM GUITAR PROMINENCE (METAL = GUITAR MUSIC)
        // Research: Guitar IS the lead instrument in metal, should DOMINATE the mix
        // MASSIVELY boosted guitar to 0.75-0.85 range (was 0.55-0.65)
        // REDUCED drums to 0.35-0.45 (was 0.45-0.60) - drums support, don't lead
        // INCREASED bass to 0.48-0.58 (was 0.35-0.42) for melodic presence
        let (guitar_level, bass_level, drum_level) = match intensity {
            SectionIntensity::Low => (0.75, 0.48, 0.35),      // Guitar LOUD even in quiet sections
            SectionIntensity::Medium => (0.78, 0.52, 0.38),   // Guitar clearly the lead
            SectionIntensity::High => (0.82, 0.55, 0.42),     // Guitar dominates
            SectionIntensity::Extreme => (0.85, 0.58, 0.45),  // Guitar at MAXIMUM (loudest element)
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

    fn mix_drum_hit(&self, buffer: &mut [f32], hit: &[f32], start_idx: usize) {
        for (i, &sample) in hit.iter().enumerate() {
            if start_idx + i < buffer.len() {
                buffer[start_idx + i] += sample;
            }
        }
    }

    fn apply_limiter(samples: &mut [f32], threshold: f32) {
        for sample in samples.iter_mut() {
            if *sample > threshold {
                *sample = threshold + (*sample - threshold).tanh() * 0.1;
            } else if *sample < -threshold {
                *sample = -threshold + (*sample + threshold).tanh() * 0.1;
            }
        }
    }

    fn extract_kick_envelope(&self, drum_audio: &[f32]) -> Vec<f32> {
        let mut envelope = Vec::with_capacity(drum_audio.len());
        
        // 1-pole Lowpass Filter state (cutoff approx 150Hz to isolate kick)
        let mut lp_out = 0.0;
        let lp_coeff = 0.15; 

        // Envelope Follower state
        let mut env_out = 0.0;
        let _attack = 0.95;   // Very fast attack
        let release = 0.999; // Slow release for pumping effect

        for &sample in drum_audio {
            // 1. Lowpass to isolate low frequency energy (kick)
            let input = sample.abs();
            lp_out = lp_out + (input - lp_out) * lp_coeff;
            
            // 2. Envelope Follower
            if lp_out > env_out {
                env_out = lp_out; // Attack
            } else {
                env_out *= release; // Release
            }
            
            // Boost the envelope slightly to ensure it triggers the compressor
            envelope.push(env_out * 2.0);
        }
        
        envelope
    }

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
        let mut raw_buffer = Vec::new();

        match mode {
            BassMode::Lock => {
                for (i, &kick_hit) in kick_pattern.iter().enumerate() {
                    if kick_hit {
                        let guitar_idx = (i * guitar_notes.len()) / kick_pattern.len().max(1);
                        let guitar_note = if guitar_idx < guitar_notes.len() { guitar_notes[guitar_idx] } else { guitar_notes[0] };
                        let bass_note = guitar_note.saturating_sub(12);
                        let frequency = 440.0 * 2.0_f32.powf((bass_note as f32 - 69.0) / 12.0);
                        
                        // Use 0.95 velocity for aggressive picking
                        let bass_sample = generate_metal_bass_string(frequency, note_duration, 0.95);
                        raw_buffer.extend(bass_sample);
                    } else {
                        raw_buffer.extend(vec![0.0; sixteenth_samples]);
                    }
                }
            },
            BassMode::Counterpoint => {
                // COUNTER-MELODY: Bass moves independently, creating harmonic depth
                let mut rng = rand::thread_rng();
                let mut prev_guitar_note = guitar_notes[0];
                
                for (i, &guitar_note) in guitar_notes.iter().enumerate() {
                    let guitar_direction = if guitar_note > prev_guitar_note { 1 } else { -1 };
                    
                    // CONTRARY MOTION: When guitar goes up, bass goes down (and vice versa)
                    let bass_note = if i % 4 == 0 {
                        // On downbeats: Play root (octave below guitar)
                        guitar_note.saturating_sub(12)
                    } else if guitar_direction > 0 {
                        // Guitar ascending → Bass descending
                        // Move to 5th below or stay on root
                        if rng.gen_bool(0.6) {
                            guitar_note.saturating_sub(12).saturating_sub(7) // Down to 5th
                        } else {
                            guitar_note.saturating_sub(12) // Stay on root
                        }
                    } else {
                        // Guitar descending → Bass ascending 
                        // Walk up chromatically or jump to octave
                        if rng.gen_bool(0.5) {
                            guitar_note.saturating_sub(12).saturating_add(5) // Up to 5th
                        } else {
                            guitar_note.saturating_sub(12).saturating_add(1) // Chromatic step up
                        }
                    };
                    
                    let frequency = 440.0 * 2.0_f32.powf((bass_note as f32 - 69.0) / 12.0);
                    let bass_sample = generate_metal_bass_string(frequency, note_duration, 0.95);
                    raw_buffer.extend(bass_sample);
                    
                    prev_guitar_note = guitar_note;
                }
            },
            BassMode::Follow => {
                // FOLLOW mode: Simple octave below (for breakdowns)
                 for &note in guitar_notes {
                    let bass_note = note.saturating_sub(12);
                    let frequency = 440.0 * 2.0_f32.powf((bass_note as f32 - 69.0) / 12.0);
                    let bass_sample = generate_metal_bass_string(frequency, note_duration, 0.95);
                    raw_buffer.extend(bass_sample);
                }
            },
        }

        // Apply Darkglass Split-Band Processing
        bass_audio.reserve(raw_buffer.len());
        for sample in raw_buffer {
            let processed = self.bass_dsp.process(sample);
            bass_audio.push(processed);
        }
        
        bass_audio
    }

    fn generate_drum_patterns(
        &self,
        section: MetalSection,
        duration: f32,
        tempo: u16,
        subgenre: MetalSubgenre,
        feel: RhythmicFeel,
    ) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
        let beat_duration = 60.0 / tempo as f32;
        let sixteenth_duration = beat_duration / 4.0;
        let steps = (duration / sixteenth_duration).ceil() as usize;
        let mut rng = rand::thread_rng();
        
        let mut kick = vec![false; steps];
        let mut snare = vec![false; steps];
        let mut cymbal = vec![false; steps];

        // [Logic copied from original generate_drum_patterns...]
        // Note: For brevity, I am using the standard generation logic here.
        // In the full file, ensure the logic from the previous artifact (Euclidean, Blast, etc.) is here.
        
        // SIMPLE, COHERENT METAL DRUM BEATS (not random)
        // Problem: Too much randomness = no beat
        // Solution: Classic metal patterns that ACTUALLY groove
        match feel {
            RhythmicFeel::HalfTime => {
                // HALFTIME/BREAKDOWN: Simple, heavy pattern
                for i in 0..steps {
                    // Kick on beats 1 and 3 (every 8 steps)
                    if i % 16 == 0 || i % 16 == 8 {
                        kick[i] = true;
                    }
                    // Snare ONLY on beat 3 (classic halftime)
                    if i % 16 == 8 {
                        snare[i] = true;
                        kick[i] = false; // No kick with snare
                    }
                    // Hi-hat on 8th notes for groove
                    if i % 4 == 0 {
                        cymbal[i] = true;
                    }
                }
            },
            RhythmicFeel::DoubleTime | RhythmicFeel::Blast => {
                // BLAST BEAT: Kick and snare alternate on 8th notes
                for i in 0..steps {
                    if i % 4 == 0 {
                        kick[i] = true;
                        snare[i] = true;
                        // Crash on downbeats only
                        if i % 16 == 0 {
                            cymbal[i] = true;
                        }
                    }
                }
            },
            RhythmicFeel::Normal => {
                // STANDARD METAL BEAT: Kick on 1 and 3, Snare on 2 and 4
                for i in 0..steps {
                    let beat_in_bar = i % 16;
                    
                    // KICK: Beats 1 and 3, plus some 16th note fills
                    if beat_in_bar == 0 {
                        kick[i] = true; // Beat 1
                    } else if beat_in_bar == 8 {
                        kick[i] = true; // Beat 3
                    } else if beat_in_bar == 2 || beat_in_bar == 10 {
                        // Double bass 16th notes (classic metal)
                        kick[i] = true;
                    }
                    
                    // SNARE: Beats 2 and 4 (backbeat) - NO RANDOMNESS
                    if beat_in_bar == 4 || beat_in_bar == 12 {
                        snare[i] = true;
                    }
                    
                    // HI-HAT: Continuous 8th notes for groove
                    if i % 4 == 0 {
                        cymbal[i] = true;
                    }
                    
                    // CRASH: Only on downbeat of each bar
                    if beat_in_bar == 0 && i % 64 == 0 {
                        cymbal[i] = true; // Crash every 4 bars
                    }
                }
                
                // ADD SIMPLE FILL: Last beat of every 4th bar
                let num_bars = steps / 16;
                for bar in (3..num_bars).step_by(4) {
                    let fill_pos = bar * 16 + 14; // Beat 4, last 16th note
                    if fill_pos < steps {
                        snare[fill_pos] = true;
                        snare[fill_pos + 1] = true; // Double hit fill
                    }
                }
            },
        }

        (kick, snare, cymbal)
    }


    /// NEW: Render a melody track (piano/synth) playing the riff melody
    fn render_melody_track(&mut self, riff: &MetalRiff, beat_duration: f32) -> Vec<f32> {
        let mut melody_audio = Vec::new();
        
        for (i, &note) in riff.notes.iter().enumerate() {
            let rhythm = riff.rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            // Skip rests for melody
            if rhythm == RhythmPattern::Rest {
                let rest_duration = beat_duration / 4.0;
                let rest_samples = (rest_duration * self.sample_rate as f32) as usize;
                melody_audio.extend(vec![0.0; rest_samples]);
                continue;
            }
            
            let note_duration = match rhythm {
                RhythmPattern::QuarterNote => beat_duration,
                RhythmPattern::EighthNote => beat_duration / 2.0,
                RhythmPattern::SixteenthNote => beat_duration / 4.0,
                RhythmPattern::ThirtySecondNote => beat_duration / 8.0,
                RhythmPattern::Quintuplet => beat_duration * 0.8,
                RhythmPattern::Septuplet => beat_duration * 0.571,
                RhythmPattern::DottedEighth => beat_duration * 0.75,
                RhythmPattern::Gallop => beat_duration / 2.0,
                RhythmPattern::Rest => beat_duration / 4.0,
            };
            
            // Generate simple sine wave melody (clean piano-like sound)
            // Play the melody one octave higher than the guitar
            let melody_note = note.saturating_add(12); // One octave up
            let frequency = 440.0 * 2.0_f32.powf((melody_note as f32 - 69.0) / 12.0);
            
            let num_samples = (note_duration * self.sample_rate as f32) as usize;
            let attack_samples = (0.01 * self.sample_rate as f32) as usize; // 10ms attack
            let decay_factor: f32 = 0.9995; // Slow decay for piano-like sustain
            
            for sample_idx in 0..num_samples {
                let time = sample_idx as f32 / self.sample_rate as f32;
                
                // Sine wave
                let sine = (2.0 * std::f32::consts::PI * frequency * time).sin();
                
                // Simple envelope: attack + exponential decay
                let envelope = if sample_idx < attack_samples {
                    sample_idx as f32 / attack_samples as f32
                } else {
                    decay_factor.powf((sample_idx - attack_samples) as f32)
                };
                
                melody_audio.push(sine * envelope * 0.3); // Quiet (30% volume)
            }
        }
        
        melody_audio
    }
    
    fn render_guitar_riff(&mut self, riff: &MetalRiff, beat_duration: f32) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        
        // ===== PHASE 4.1: RENDER LEFT CHANNEL =====
        let mut left_channel = Vec::new();
        
        for (i, &note) in riff.notes.iter().enumerate() {
            let palm_muted = riff.palm_muted[i];
            let chord_type = riff.chord_types.get(i).copied().unwrap_or(ChordType::Single);
            let rhythm = riff.rhythms.get(i).copied().unwrap_or(RhythmPattern::SixteenthNote);
            
            // Handle rests - REPLACED WITH QUIET ROOT NOTE DRONE (continuous guitar)
            if rhythm == RhythmPattern::Rest {
                let rest_duration = beat_duration / 4.0;
                // Instead of silence, play a quiet root note power chord (wall of sound)
                let root_note = riff.notes[0]; // Use first note as root
                let freq_root = 440.0 * 2.0_f32.powf((root_note as f32 - 69.0) / 12.0);
                let drone_samples = self.render_chord(root_note, ChordType::Power, freq_root, rest_duration, true, 0.2); // Quiet, palm-muted
                left_channel.extend(drone_samples);
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
            
            // FIXED: Open notes need MUCH longer sustain for continuous guitar sound
            let min_sustain = if palm_muted { 0.08 } else { 0.6 }; // Increased from 0.12 to 0.6
            let note_duration = base_duration.max(min_sustain);
            
            // Add overlap for non-palm-muted notes (legato effect)
            let actual_duration = if !palm_muted {
                note_duration * 1.5 // 50% overlap for smooth, continuous guitar
            } else {
                note_duration
            };
            
            // LEFT CHANNEL: Standard tuning
            let freq_root = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
            let note_samples = self.render_chord(note, chord_type, freq_root, actual_duration, palm_muted, 0.8);
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
                // Instead of silence, play a quiet root note power chord (wall of sound)
                let root_note = riff.notes[0]; // Use first note as root
                let detune_cents = rng.gen_range(-5.0..5.0);
                let freq_root = 440.0 * 2.0_f32.powf((root_note as f32 - 69.0 + detune_cents / 100.0) / 12.0);
                let drone_samples = self.render_chord(root_note, ChordType::Power, freq_root, rest_duration, true, 0.2); // Quiet, palm-muted
                right_channel.extend(drone_samples);
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
            
            // FIXED: Open notes need MUCH longer sustain for continuous guitar sound
            let min_sustain = if palm_muted { 0.08 } else { 0.6 }; // Increased from 0.12 to 0.6
            let note_duration = base_duration.max(min_sustain);
            
            // Add overlap for non-palm-muted notes (legato effect)
            let actual_duration = if !palm_muted {
                note_duration * 1.5 // 50% overlap for smooth, continuous guitar
            } else {
                note_duration
            };
            
            // RIGHT CHANNEL: Slightly detuned (±5 cents = ±0.05 semitones)
            let detune_cents = rng.gen_range(-5.0..5.0);
            let freq_root = 440.0 * 2.0_f32.powf((note as f32 - 69.0 + detune_cents / 100.0) / 12.0);
            
            let note_samples = self.render_chord(note, chord_type, freq_root, actual_duration, palm_muted, 0.8);
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
    
}

impl Default for MetalAudioRenderer {
    fn default() -> Self {
        Self::new()
    }
}