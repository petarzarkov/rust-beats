/// Song generation orchestrator
use crate::composition::{
    Arrangement, Genre, Key, Tempo, generate_chord_progression,
    GrooveStyle, select_random_drum_kit, select_random_genre, get_genre_config,
};
use crate::composition::beat_maker::DrumKit;
use crate::audio::{
    render_arranged_drums, render_arranged_bass, render_arranged_melody,
    generate_pads_with_arrangement, render_fx_track, add_percussion_track,
    apply_arrangement_dynamics, master_lofi, stereo_to_mono, normalize_loudness,
    build_tracks, mix_tracks, MixingPreset, preset_from_str,
    render_to_wav_with_metadata, encode_to_mp3, SongMetadata,
};
use crate::synthesis::{SAMPLE_RATE, LofiProcessor};
use crate::config::Config;
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
}

impl SongGenerator {
    /// Create a new song generator with random parameters
    pub fn new(config: Config) -> Self {
        let mut rng = rand::thread_rng();
        
        // Generate song identity
        let song_name = crate::composition::generate_song_name();
        let genre_tags = crate::composition::generate_genre_tags();
        
        // Select genre and get its configuration
        let genre = select_random_genre();
        let genre_config = get_genre_config(genre);
        
        // Generate musical parameters based on genre
        let key = if !genre_config.preferred_scales.is_empty() {
            let scale = genre_config.preferred_scales[rng.gen_range(0..genre_config.preferred_scales.len())];
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
        
        // Select instruments
        let ip = &config.composition.instrument_probabilities;
        let lead_instrument = {
            let roll: f32 = rng.gen_range(0.0..1.0);
            if roll < ip.rhodes { 
                "Rhodes" 
            } else if roll < ip.rhodes + ip.ukulele {
                "Ukulele"
            } else if roll < ip.rhodes + ip.ukulele + ip.guitar {
                "Guitar"
            } else if roll < ip.rhodes + ip.ukulele + ip.guitar + ip.electric {
                "Electric"
            } else {
                "Organ"
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
        
        let drum_kit = select_random_drum_kit();
        
        // Percussion
        let pc = &config.composition.percussion;
        let add_percussion = rng.gen_range(0.0..1.0) < pc.chance;
        let percussion_type = if add_percussion {
            let roll: f32 = rng.gen_range(0.0..1.0);
            if roll < pc.tambourine { "Tambourine" } 
            else if roll < pc.tambourine + pc.cowbell { "Cowbell" }
            else if roll < pc.tambourine + pc.cowbell + pc.bongo { "Bongo" }
            else { "Woodblock" }
        } else {
            "None"
        };
        
        // Pad intensity
        let pad_cfg = &config.composition.pads;
        let pad_intensity = {
            let roll: f32 = rng.gen_range(0.0..1.0);
            if roll < pad_cfg.subtle { "Subtle" }
            else if roll < pad_cfg.subtle + pad_cfg.medium { "Medium" }
            else { "Prominent" }
        };
        
        // Mixing style
        let mix_cfg = &config.composition.mixing;
        let mixing_style = {
            let roll: f32 = rng.gen_range(0.0..1.0);
            if roll < mix_cfg.clean { "Clean" }
            else if roll < mix_cfg.clean + mix_cfg.warm { "Warm" }
            else if roll < mix_cfg.clean + mix_cfg.warm + mix_cfg.punchy { "Punchy" }
            else { "Spacious" }
        };
        
        // Generate arrangement
        let arrangement = if config.composition.structure == "short" {
            Arrangement::generate_short()
        } else {
            Arrangement::generate_standard()
        };
        
        let num_bars = arrangement.total_bars;
        
        // Generate chord progression
        let chords = if !genre_config.preferred_chord_types.is_empty() {
            crate::composition::music_theory::generate_chord_progression_with_types(
                &key,
                num_bars,
                Some(&genre_config.preferred_chord_types)
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
        
        Self {
            config,
            params,
            genre_config,
        }
    }
    
    /// Get song parameters (for printing)
    pub fn params(&self) -> &SongParams {
        &self.params
    }
    
    /// Generate all audio tracks in parallel
    pub fn generate_audio_tracks(&self) -> (
        Vec<f32>, // drums
        Vec<f32>, // bass
        Vec<f32>, // melody_l
        Vec<f32>, // melody_r
        Vec<f32>, // pads_l
        Vec<f32>, // pads_r
        Vec<f32>, // fx
        Option<Vec<f32>>, // percussion
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
        let chords6 = self.params.chords.clone();
        
        let tempo_bpm = self.params.tempo.bpm;
        let genre = self.params.genre;
        let groove_style = self.params.groove_style;
        let drum_kit = self.params.drum_kit;
        let percussion_type = self.params.percussion_type.clone();
        let num_bars = arrangement1.total_bars;
        let tempo = self.params.tempo.clone();
        let tempo2 = self.params.tempo.clone();
        let key1 = self.params.key.clone();
        let key2 = self.params.key.clone();
        let melody_cfg1 = self.config.composition.melody.clone();
        let melody_cfg2 = self.config.composition.melody.clone();
        let bass_drop_cfg = self.config.composition.bass_drops.clone();
        let genre_config = self.genre_config.clone();
        
        // Generate audio tracks in parallel using threads
        let drums_handle = thread::spawn(move || {
            render_arranged_drums(&arrangement1, groove_style, tempo_bpm, drum_kit, &genre)
        });
        let bass_handle = thread::spawn(move || {
            render_arranged_bass(&arrangement2, &chords1, tempo_bpm, &genre, &bass_drop_cfg, &genre_config)
        });
        let melody_l_handle = thread::spawn(move || {
            render_arranged_melody(&arrangement3, &key1, &chords2, tempo_bpm, &melody_cfg1, &genre)
        });
        let melody_r_handle = thread::spawn(move || {
            render_arranged_melody(&arrangement4, &key2, &chords3, tempo_bpm, &melody_cfg2, &genre)
        });
        let pads_l_handle = thread::spawn(move || {
            generate_pads_with_arrangement(&arrangement5, &chords4, tempo_bpm, num_bars)
        });
        let pads_r_handle = thread::spawn(move || {
            generate_pads_with_arrangement(&arrangement6, &chords5, tempo_bpm, num_bars)
        });
        let fx_handle = thread::spawn(move || {
            render_fx_track(&arrangement7, tempo_bpm)
        });
        let percussion_handle = thread::spawn(move || {
            add_percussion_track(&percussion_type, &arrangement8, &tempo2)
        });
        
        (
            drums_handle.join().unwrap(),
            bass_handle.join().unwrap(),
            melody_l_handle.join().unwrap(),
            melody_r_handle.join().unwrap(),
            pads_l_handle.join().unwrap(),
            pads_r_handle.join().unwrap(),
            fx_handle.join().unwrap(),
            percussion_handle.join().unwrap(),
        )
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
    ) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        
        // Build tracks with mixing preset
        let preset = preset_from_str(&self.params.mixing_style);
        let tracks = build_tracks(
            drums, bass, melody_l, melody_r, pads_l, pads_r, fx, percussion,
            preset, &self.params.pad_intensity,
        );
        
        let mut stereo_mix = mix_tracks(tracks);
        
        // Apply arrangement-aware dynamics
        apply_arrangement_dynamics(&mut stereo_mix, &self.params.arrangement, self.params.tempo.bpm);
        
        // Apply lofi mastering
        master_lofi(&mut stereo_mix, 0.70, 0.5);
        
        // Convert to mono and apply lofi effects
        let mut final_mix = stereo_to_mono(&stereo_mix);
        
        // Apply lofi processing based on genre
        let mut lofi_processor = match self.params.genre {
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
        
        // Normalize loudness
        normalize_loudness(&mut final_mix, 0.25, 0.18);
        
        final_mix
    }
    
    /// Save song files (WAV, MP3, JSON) in parallel
    pub fn save_song(&self, final_mix: &[f32], date: &str) -> Result<(), String> {
        use crate::utils::{sanitize_filename, create_output_directory};
        
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
        let duration = final_mix.len() as f32 / SAMPLE_RATE() as f32;
        let date_json = date.to_string();
        let write_json = self.config.generation.write_metadata_json;
        let wav_path_clone = wav_path.clone();
        let mp3_path_clone = mp3_path.clone();
        let json_path_clone = json_path.clone();
        
        let wav_handle = thread::spawn(move || {
            render_to_wav_with_metadata(&final_mix_wav, &wav_path_clone, &metadata_wav)
                .map_err(|e| format!("WAV: {}", e))
        });
        
        let mp3_handle = thread::spawn(move || {
            encode_to_mp3(&final_mix_mp3, &mp3_path_clone, &song_name_mp3, &artist_mp3)
                .map_err(|e| format!("MP3: {}", e))
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
                
                fs::write(&json_path_clone, serde_json::to_string_pretty(&metadata_json).unwrap())
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
        
        match mp3_result {
            Ok(_) => println!("✅ Successfully created MP3: {}", mp3_path),
            Err(e) => eprintln!("⚠️  Warning: Could not create MP3: {}", e),
        }
        
        if write_json {
            match json_result {
                Ok(_) => {},
                Err(e) => eprintln!("⚠️  Warning: Could not write metadata: {}", e),
            }
        }
        
        Ok(())
    }
}

