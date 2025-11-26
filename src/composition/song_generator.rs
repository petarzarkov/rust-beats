use crate::audio::{
    add_percussion_track, apply_arrangement_dynamics, build_tracks, calculate_voice_timings,
    encode_to_mp3, generate_pads_with_arrangement, generate_voice_segment, master_lofi,
    mix_tracks, mix_with_ducking, normalize_loudness, preset_from_str, render_arranged_bass,
    render_arranged_drums, render_arranged_melody_with_instrument, render_fx_track,
    render_to_wav_with_metadata, select_wisdom_with_chorus, stereo_to_mono, SongMetadata,
    VoiceSegment, WisdomData,
};
use crate::composition::beat_maker::DrumKit;
/// Song generation orchestrator
use crate::composition::{
    generate_chord_progression, get_genre_config, select_preferred_drum_kit, select_random_genre,
    Arrangement, Genre, GrooveStyle, Key, Tempo,
};
use crate::config::Config;
use crate::synthesis::{get_sample_rate, InstrumentType, LofiProcessor};
use rand::Rng;
use std::fs;

/// Song generation parameters
pub struct SongParams {
    pub song_name: String,
    pub genre_tags: Vec<String>,
    pub genre: Genre,
    pub key: Key,
    pub tempo: Tempo,
    pub groove_style: GrooveStyle,
    pub drum_kit: DrumKit,
    pub lead_instrument: String,
    pub bass_type: String,
    pub percussion_type: String,
    pub pad_intensity: String,
    pub mixing_style: String,
    pub arrangement: Arrangement,
    pub chords: Vec<crate::composition::music_theory::Chord>,
}

/// Song generator that orchestrates the entire generation process
pub struct SongGenerator {
    config: Config,
    params: SongParams,
    genre_config: crate::composition::genre::GenreConfig,
    wisdom_data: Option<WisdomData>,
}

impl SongGenerator {
    /// Create a new song generator with random parameters
    pub fn new(config: Config) -> Self {
        let mut rng = rand::thread_rng();

        // Generate song identity
        let song_name = crate::composition::generate_song_name();

        // Select genre and get its configuration
        let genre = select_random_genre();
        let genre_config = get_genre_config(genre);

        // Generate genre tags based on actual genre
        let genre_tags = crate::composition::generate_genre_tags(genre);

        // Generate musical parameters based on genre
        let key = if !genre_config.preferred_scales.is_empty() {
            let scale = genre_config.preferred_scales
                [rng.gen_range(0..genre_config.preferred_scales.len())];
            Key::from_scale(scale)
        } else {
            Key::random_funky()
        };

        // Clamp tempo to config limits
        let tempo_min = genre_config.tempo_min.max(config.composition.min_tempo);
        let tempo_max = genre_config.tempo_max.min(config.composition.max_tempo);
        let (tempo_min, tempo_max) = if tempo_min < tempo_max {
            (tempo_min, tempo_max)
        } else {
            let cfg_min = config.composition.min_tempo;
            let cfg_max = config.composition.max_tempo;
            if cfg_min < cfg_max {
                (cfg_min, cfg_max)
            } else {
                eprintln!("⚠️  Warning: Invalid tempo range in config ({} >= {}), using default 80-120 BPM", cfg_min, cfg_max);
                (80.0, 120.0)
            }
        };
        let tempo = Tempo::random_funky_range(tempo_min, tempo_max);

        // Map genre to groove style
        let groove_style = match genre {
            Genre::Rock => GrooveStyle::Rock,
            Genre::Dubstep => GrooveStyle::Dubstep,
            Genre::DnB => GrooveStyle::DnB,
            Genre::Jazz => GrooveStyle::Jazz,
            Genre::Funk => GrooveStyle::Funk,
            Genre::HipHop => GrooveStyle::HipHop,
            Genre::ElectroSwing => GrooveStyle::ElectroSwing,
            Genre::Lofi => GrooveStyle::Lofi,
        };

        // Select instruments based on genre
        let lead_instrument = match genre {
            Genre::Lofi => match rng.gen_range(0..100) {
                0..=60 => "Rhodes",
                61..=90 => "SoftPluck",
                _ => "WarmOrgan"
            },
            Genre::Jazz | Genre::Funk => match rng.gen_range(0..100) {
                0..=40 => "Rhodes",
                41..=70 => "Piano", // Mapped to Rhodes usually but implies style
                71..=90 => "Guitar",
                _ => "Organ"
            },
            Genre::Rock => match rng.gen_range(0..100) {
                0..=70 => "Electric Guitar",
                71..=90 => "Acoustic Guitar",
                _ => "Organ"
            },
            Genre::DnB | Genre::Dubstep => match rng.gen_range(0..100) {
                0..=50 => "Synth",
                51..=80 => "Electric",
                _ => "Pluck"
            },
            Genre::ElectroSwing => match rng.gen_range(0..100) {
                0..=40 => "Clarinet", // Simulated via filtered Square/Saw
                41..=70 => "Organ",
                _ => "Piano"
            },
            Genre::HipHop => match rng.gen_range(0..100) {
                0..=50 => "Rhodes",
                51..=80 => "Synth",
                _ => "Piano"
            }
        };

        let bass_type = match genre_config.bass_style {
            crate::composition::genre::BassStyle::Standard => "Standard",
            crate::composition::genre::BassStyle::Rock => "Rock",
            crate::composition::genre::BassStyle::Synth => "Synth",
            crate::composition::genre::BassStyle::Upright => "Upright",
            crate::composition::genre::BassStyle::Finger => "Finger",
            crate::composition::genre::BassStyle::Slap => "Slap",
            crate::composition::genre::BassStyle::Wobble => "Wobble",
            crate::composition::genre::BassStyle::Reese => "Reese",
        };

        let drum_kit = select_preferred_drum_kit(&genre_config.drum_kit_preference);

        // Percussion
        let pc = &config.composition.percussion;
        let add_percussion = rng.gen_range(0.0..1.0) < pc.chance;
        let percussion_type = if add_percussion {
            let roll: f32 = rng.gen_range(0.0..1.0);
            if roll < pc.tambourine {
                "Tambourine"
            } else if roll < pc.tambourine + pc.cowbell {
                "Cowbell"
            } else if roll < pc.tambourine + pc.cowbell + pc.bongo {
                "Bongo"
            } else {
                "Woodblock"
            }
        } else {
            "None"
        };

        // Pad intensity
        let pad_cfg = &config.composition.pads;
        let pad_intensity = {
            let roll: f32 = rng.gen_range(0.0..1.0);
            if roll < pad_cfg.subtle {
                "Subtle"
            } else if roll < pad_cfg.subtle + pad_cfg.medium {
                "Medium"
            } else {
                "Prominent"
            }
        };

        // Mixing style
        let mix_cfg = &config.composition.mixing;
        let mixing_style = {
            let roll: f32 = rng.gen_range(0.0..1.0);
            if roll < mix_cfg.clean {
                "Clean"
            } else if roll < mix_cfg.clean + mix_cfg.warm {
                "Warm"
            } else if roll < mix_cfg.clean + mix_cfg.warm + mix_cfg.punchy {
                "Punchy"
            } else {
                "Spacious"
            }
        };

        // Generate arrangement
        let arrangement = if config.composition.structure == "short" {
            Arrangement::generate_short()
        } else {
            // Dubstep and DnB should be shorter (60-90 seconds) for better impact
            // Other genres use standard 3 minutes (180 seconds)
            let target_duration = match genre {
                Genre::Dubstep | Genre::DnB => rng.gen_range(60.0..90.0), // 1-1.5 minutes
                _ => 180.0 // Standard 3 minutes for other genres
            };
            Arrangement::generate_for_duration(
                genre_config.arrangement_style,
                tempo.bpm,
                target_duration
            )
        };

        let num_bars = arrangement.total_bars;

        // Generate chord progression
        let chords = if !genre_config.preferred_chord_types.is_empty() {
            crate::composition::music_theory::generate_chord_progression_with_types(
                &key,
                num_bars,
                Some(&genre_config.preferred_chord_types),
            )
        } else {
            generate_chord_progression(&key, num_bars)
        };

        let params = SongParams {
            song_name,
            genre_tags,
            genre,
            key,
            tempo,
            groove_style,
            drum_kit,
            lead_instrument: lead_instrument.to_string(),
            bass_type: bass_type.to_string(),
            percussion_type: percussion_type.to_string(),
            pad_intensity: pad_intensity.to_string(),
            mixing_style: mixing_style.to_string(),
            arrangement,
            chords,
        };

        // Load wisdom data if voice is enabled
        let wisdom_data = if config.voice.enabled {
            match WisdomData::load(&config.voice.wisdom_file) {
                Ok(data) => {
                    println!("  ✓ Loaded {} wisdom quotes", data.wisdom.len());
                    Some(data)
                }
                Err(e) => {
                    eprintln!("  ⚠️  Warning: Could not load wisdom file: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            config,
            params,
            genre_config,
            wisdom_data,
        }
    }

    /// Get song parameters (for printing)
    pub fn params(&self) -> &SongParams {
        &self.params
    }

    /// Generate all audio tracks in parallel
    pub fn generate_audio_tracks(
        &self,
    ) -> (
        Vec<f32>,                // drums
        Vec<f32>,                // bass
        Vec<f32>,                // melody_l
        Vec<f32>,                // melody_r
        Vec<f32>,                // pads_l
        Vec<f32>,                // pads_r
        Vec<f32>,                // fx
        Option<Vec<f32>>,        // percussion
        Vec<VoiceSegment>,       // voice segments
    ) {
        use std::thread;

        // Clone all data needed for threads
        let arrangement1 = self.params.arrangement.clone();
        let arrangement2 = self.params.arrangement.clone();
        let arrangement3 = self.params.arrangement.clone();
        let arrangement4 = self.params.arrangement.clone();
        let arrangement5 = self.params.arrangement.clone();
        let arrangement6 = self.params.arrangement.clone();
        let arrangement7 = self.params.arrangement.clone();
        let arrangement8 = self.params.arrangement.clone();

        let chords1 = self.params.chords.clone();
        let chords2 = self.params.chords.clone();
        let chords3 = self.params.chords.clone();
        let chords4 = self.params.chords.clone();
        let chords5 = self.params.chords.clone();

        let tempo_bpm = self.params.tempo.bpm;
        let genre = self.params.genre;
        let groove_style = self.params.groove_style;
        let drum_kit = self.params.drum_kit;
        let percussion_type = self.params.percussion_type.clone();
        let num_bars = arrangement1.total_bars;
        let tempo2 = self.params.tempo.clone();
        let key1 = self.params.key.clone();
        let key2 = self.params.key.clone();
        let melody_cfg1 = self.config.composition.melody.clone();
        let melody_cfg2 = self.config.composition.melody.clone();
        let bass_drop_cfg = self.config.composition.bass_drops.clone();
        let genre_config_bass = self.genre_config.clone();
        let genre_config_drums = self.genre_config.clone();
        let melody_density = self.genre_config.melody_density;

        // Generate audio tracks in parallel using threads
        let drums_handle = thread::spawn(move || {
            render_arranged_drums(
                &arrangement1,
                groove_style,
                tempo_bpm,
                drum_kit,
                &genre,
                &genre_config_drums,
            )
        });
        let bass_handle = thread::spawn(move || {
            render_arranged_bass(
                &arrangement2,
                &chords1,
                tempo_bpm,
                &genre,
                &bass_drop_cfg,
                &genre_config_bass,
            )
        });
        // Generate L/R melody channels with different instrument preferences for stereo width
        let melody_l_handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            // L channel: prefer warmer, rounder instruments
            let instrument_pref = match genre {
                Genre::Rock => if rng.gen_bool(0.7) { Some(InstrumentType::AcousticGuitar) } else { Some(InstrumentType::ElectricGuitar) },
                Genre::Jazz | Genre::Funk => if rng.gen_bool(0.6) { Some(InstrumentType::Rhodes) } else { Some(InstrumentType::WarmOrgan) },
                Genre::Lofi => if rng.gen_bool(0.5) { Some(InstrumentType::SoftPluck) } else { Some(InstrumentType::Rhodes) },
                Genre::Dubstep | Genre::DnB => if rng.gen_bool(0.5) { Some(InstrumentType::WarmOrgan) } else { Some(InstrumentType::ElectricGuitar) },
                _ => if rng.gen_bool(0.5) { Some(InstrumentType::Ukulele) } else { Some(InstrumentType::Mallet) },
            };
            render_arranged_melody_with_instrument(
                &arrangement3,
                &key1,
                &chords2,
                tempo_bpm,
                &melody_cfg1,
                melody_density,
                &genre,
                instrument_pref,
            )
        });
        let melody_r_handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            // R channel: prefer brighter, contrasting instruments
            let instrument_pref = match genre {
                Genre::Rock => if rng.gen_bool(0.8) { Some(InstrumentType::ElectricGuitar) } else { Some(InstrumentType::AcousticGuitar) },
                Genre::Jazz | Genre::Funk => if rng.gen_bool(0.6) { Some(InstrumentType::Mallet) } else { Some(InstrumentType::Rhodes) },
                Genre::Lofi => if rng.gen_bool(0.5) { Some(InstrumentType::WarmOrgan) } else { Some(InstrumentType::SoftPluck) },
                Genre::Dubstep | Genre::DnB => if rng.gen_bool(0.7) { Some(InstrumentType::ElectricGuitar) } else { Some(InstrumentType::Mallet) },
                _ => if rng.gen_bool(0.6) { Some(InstrumentType::AcousticGuitar) } else { Some(InstrumentType::Ukulele) },
            };
            render_arranged_melody_with_instrument(
                &arrangement4,
                &key2,
                &chords3,
                tempo_bpm,
                &melody_cfg2,
                melody_density,
                &genre,
                instrument_pref,
            )
        });
        let pads_l_handle = thread::spawn(move || {
            generate_pads_with_arrangement(&arrangement5, &chords4, tempo_bpm, num_bars)
        });
        let pads_r_handle = thread::spawn(move || {
            generate_pads_with_arrangement(&arrangement6, &chords5, tempo_bpm, num_bars)
        });
        let fx_handle = thread::spawn(move || render_fx_track(&arrangement7, tempo_bpm));
        let percussion_handle = thread::spawn(move || {
            add_percussion_track(&percussion_type, &arrangement8, &tempo2, &genre)
        });

        // Generate voice segments if enabled
        let voice_segments = if self.config.voice.enabled && self.wisdom_data.is_some() {
            self.generate_voice_segments()
        } else {
            Vec::new()
        };

        (
            drums_handle.join().unwrap(),
            bass_handle.join().unwrap(),
            melody_l_handle.join().unwrap(),
            melody_r_handle.join().unwrap(),
            pads_l_handle.join().unwrap(),
            pads_r_handle.join().unwrap(),
            fx_handle.join().unwrap(),
            percussion_handle.join().unwrap(),
            voice_segments,
        )
    }

    /// Generate voice segments with wisdom quotes
    fn generate_voice_segments(&self) -> Vec<VoiceSegment> {
        let wisdom_data = match &self.wisdom_data {
            Some(data) => data,
            None => return Vec::new(),
        };

        // Generate seed from song name for deterministic selection
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.params.song_name.hash(&mut hasher);
        let seed = hasher.finish();

        // Calculate total duration
        let beats_per_sec = self.params.tempo.bpm / 60.0;
        let seconds_per_bar = 4.0 / beats_per_sec;
        let total_duration_seconds = self.params.arrangement.total_bars as f32 * seconds_per_bar;
        let sample_rate = get_sample_rate();
        let total_samples = (total_duration_seconds * sample_rate as f32) as usize;

        // Select wisdom quotes with chorus structure
        let (intro_quote, chorus_quotes, outro_quote) = select_wisdom_with_chorus(wisdom_data, seed);

        if intro_quote.is_empty() && chorus_quotes.is_empty() && outro_quote.is_empty() {
            return Vec::new();
        }

        // Build the sequence: intro + chorus + chorus + ... + outro
        // Calculate how many times to repeat the chorus based on song duration
        let duration_minutes = total_duration_seconds / 60.0;
        let target_segments = (duration_minutes * self.config.voice.segments_per_minute).round() as usize;
        let target_segments = target_segments.max(2); // At least intro + outro

        // Calculate number of chorus repetitions
        // Formula: 1 (intro) + N * 3 (chorus repeats) + 1 (outro) = target_segments
        let chorus_repeats = ((target_segments - 2) as f32 / 3.0).max(1.0).ceil() as usize;

        let mut text_sequence = Vec::new();
        text_sequence.push(intro_quote.clone()); // Intro

        // Add chorus repetitions
        for _ in 0..chorus_repeats {
            for chorus_quote in &chorus_quotes {
                text_sequence.push(chorus_quote.clone());
            }
        }

        text_sequence.push(outro_quote.clone()); // Outro

        println!("  🎵 Voice structure: 1 intro + {} chorus repeats (3 quotes each) + 1 outro = {} segments",
                 chorus_repeats, text_sequence.len());

        // Calculate voice timings for distributed placement
        let timings = calculate_voice_timings(
            "distributed",
            total_samples,
            text_sequence.len(),
            sample_rate,
        );

        // Generate voice segments using detected language
        let mut segments = Vec::new();
        for (i, text) in text_sequence.iter().enumerate() {
            if i >= timings.len() {
                break;
            }

            let segment_type = if i == 0 {
                "Intro"
            } else if i == text_sequence.len() - 1 {
                "Outro"
            } else {
                "Chorus"
            };

            println!("  🎤 Generating {} voice: {}", segment_type, text);

            match generate_voice_segment(
                text,
                &self.config.voice.language,
                &self.config.voice.espeak_data,
            ) {
                Ok(samples) => {
                    segments.push(VoiceSegment {
                        text: text.clone(),
                        start_sample: timings[i],
                        samples,
                    });
                }
                Err(e) => {
                    eprintln!("  ⚠️  Warning: Voice generation failed: {}", e);
                }
            }
        }

        segments
    }

    /// Mix and master the audio
    pub fn mix_and_master(
        &self,
        drums: Vec<f32>,
        bass: Vec<f32>,
        melody_l: Vec<f32>,
        melody_r: Vec<f32>,
        pads_l: Vec<f32>,
        pads_r: Vec<f32>,
        fx: Vec<f32>,
        percussion: Option<Vec<f32>>,
        voice_segments: Vec<VoiceSegment>,
    ) -> Vec<f32> {
        let mut rng = rand::thread_rng();

        // Build tracks with mixing preset
        let preset = preset_from_str(&self.params.mixing_style);
        let tracks = build_tracks(
            drums,
            bass,
            melody_l,
            melody_r,
            pads_l,
            pads_r,
            fx,
            percussion,
            preset,
            &self.params.pad_intensity,
        );

        let mut stereo_mix = mix_tracks(tracks);

        // Apply arrangement-aware dynamics
        apply_arrangement_dynamics(
            &mut stereo_mix,
            &self.params.arrangement,
            self.params.tempo.bpm,
        );

        // Apply lofi mastering
        master_lofi(&mut stereo_mix, 0.70, 0.5);

        // Convert to mono and apply lofi effects
        let mut final_mix = stereo_to_mono(&stereo_mix);

        // Apply lofi processing based on genre
        let lofi_processor = match self.params.genre {
            Genre::Lofi => {
                let mut proc = if rng.gen_range(0..100) < 50 {
                    LofiProcessor::heavy()
                } else {
                    LofiProcessor::medium()
                };
                proc.wow_flutter_intensity = 0.0;
                proc
            }
            Genre::Jazz | Genre::Funk => {
                let mut proc = LofiProcessor::medium();
                proc.wow_flutter_intensity = 0.0;
                proc
            }
            _ => LofiProcessor::subtle(),
        };
        lofi_processor.process(&mut final_mix);

        // Mix in voice segments with ducking if any
        if !voice_segments.is_empty() {
            let sample_rate = get_sample_rate();
            println!("  🎤 Mixing {} voice segment(s) with music ducking", voice_segments.len());

            for segment in voice_segments {
                mix_with_ducking(
                    &mut final_mix,
                    &segment,
                    self.config.voice.volume,
                    self.config.voice.duck_music_db,
                    sample_rate,
                );
            }
        }

        // Normalize loudness
        normalize_loudness(&mut final_mix, 0.25, 0.18);

        final_mix
    }

    /// Save song files (WAV, MP3, JSON) in parallel
    pub fn save_song(&self, final_mix: &[f32], date: &str) -> Result<(), String> {
        use crate::utils::{create_output_directory, sanitize_filename};

        let output_dir = &self.config.generation.output_dir;
        create_output_directory(output_dir)?;

        let sanitized_author = sanitize_filename(&self.config.metadata.artist);
        let sanitized_song_name = sanitize_filename(&self.params.song_name);
        let filename_base = format!("{}_{}_{}", date, sanitized_author, sanitized_song_name);

        // Create metadata
        let metadata = SongMetadata {
            title: self.params.song_name.clone(),
            artist: self.config.metadata.artist.clone(),
            copyright: self.config.metadata.copyright.clone(),
            genre: self.params.genre_tags.clone(),
            date: date.to_string(),
        };

        // Spawn threads for parallel file I/O
        use std::thread;

        let wav_path = format!("{}/{}.wav", output_dir, filename_base);
        let mp3_path = format!("{}/{}.mp3", output_dir, filename_base);
        let json_path = format!("{}/{}.json", output_dir, filename_base);

        // Clone data for threads
        let final_mix_wav = final_mix.to_vec();
        let final_mix_mp3 = final_mix.to_vec();
        let metadata_wav = metadata.clone();
        let song_name_mp3 = self.params.song_name.clone();
        let artist_mp3 = self.config.metadata.artist.clone();
        let song_name_json = self.params.song_name.clone();
        let artist_json = self.config.metadata.artist.clone();
        let genre_tags_json = self.params.genre_tags.clone();
        let tempo_bpm = self.params.tempo.bpm;
        let duration = final_mix.len() as f32 / get_sample_rate() as f32;
        let date_json = date.to_string();
        let write_json = self.config.generation.write_metadata_json;
        let encode_mp3 = self.config.generation.encode_mp3;
        let wav_path_clone = wav_path.clone();
        let mp3_path_clone = mp3_path.clone();
        let json_path_clone = json_path.clone();

        let wav_handle = thread::spawn(move || {
            render_to_wav_with_metadata(&final_mix_wav, &wav_path_clone, &metadata_wav)
                .map_err(|e| format!("WAV: {}", e))
        });

        let mp3_handle = thread::spawn(move || {
            if encode_mp3 {
                encode_to_mp3(&final_mix_mp3, &mp3_path_clone, &song_name_mp3, &artist_mp3)
                    .map_err(|e| format!("MP3: {}", e))
            } else {
                Ok(())
            }
        });

        let json_handle = thread::spawn(move || {
            if write_json {
                let metadata_json = serde_json::json!({
                    "name": song_name_json,
                    "artist": artist_json,
                    "genre": genre_tags_json,
                    "tempo": tempo_bpm,
                    "duration": duration,
                    "date": date_json,
                });

                fs::write(
                    &json_path_clone,
                    serde_json::to_string_pretty(&metadata_json).unwrap(),
                )
                .map_err(|e| format!("JSON: {}", e))
            } else {
                Ok(())
            }
        });

        // Wait for all threads and collect results
        let wav_result = wav_handle.join().unwrap();
        let mp3_result = mp3_handle.join().unwrap();
        let json_result = json_handle.join().unwrap();

        // Report results
        match wav_result {
            Ok(_) => println!("✅ Successfully created: {}", wav_path),
            Err(e) => eprintln!("❌ Error creating WAV: {}", e),
        }

        if self.config.generation.encode_mp3 {
            match mp3_result {
                Ok(_) => println!("✅ Successfully created MP3: {}", mp3_path),
                Err(e) => eprintln!("⚠️  Warning: Could not create MP3: {}", e),
            }
        }

        if write_json {
            match json_result {
                Ok(_) => {}
                Err(e) => eprintln!("⚠️  Warning: Could not write metadata: {}", e),
            }
        }

        Ok(())
    }
}
