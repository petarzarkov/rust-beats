mod composition;
mod synthesis;
mod audio;
mod config;

use std::fs;
use rand::Rng;
use composition::{
    generate_song_name, generate_genre_tags,
    Key, Tempo, generate_chord_progression,
    generate_drum_pattern, DrumHit, GrooveStyle, select_random_drum_kit,
    Arrangement, Genre, select_random_genre, get_genre_config, BassStyle,
};
use composition::beat_maker::DrumKit;
use synthesis::{
    generate_kick, generate_snare, generate_hihat, generate_clap,
    generate_pads,
    generate_rock_bassline, generate_dubstep_bassline, generate_dnb_bassline,
    SAMPLE_RATE, LofiProcessor,
    generate_riser, generate_downlifter, generate_crash, generate_impact,
};
use audio::{render_to_wav_with_metadata, encode_to_mp3, SongMetadata, Track, mix_tracks, master_lofi, stereo_to_mono, normalize_loudness};
use config::Config;

/// Check if a year is a leap year
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Sanitize a string for use in filenames
/// Removes or replaces invalid filesystem characters
fn sanitize_filename(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_underscore = false;
    
    for c in s.chars() {
        let mapped = match c {
            ' ' | '_' | '\'' | '"' | ',' | ';' | ':' | '!' | '?' => {
                if !last_was_underscore {
                    last_was_underscore = true;
                    '_'
                } else {
                    continue; // Skip consecutive underscores
                }
            }
            c if c.is_alphanumeric() => {
                last_was_underscore = false;
                c
            }
            '-' | '.' => {
                last_was_underscore = false;
                c
            }
            _ => {
                if !last_was_underscore {
                    last_was_underscore = true;
                    '_'
                } else {
                    continue; // Skip consecutive underscores
                }
            }
        };
        result.push(mapped);
    }
    
    result
        .trim_matches('_') // Remove leading/trailing underscores
        .to_lowercase()
        .chars()
        .take(50) // Limit length
        .collect()
}

fn main() {
    // Load configuration
    let config = Config::load_default().unwrap_or_else(|e| {
        eprintln!("⚠️  Warning: Could not load config.toml: {}", e);
        eprintln!("   Using default configuration\n");
        Config::default()
    });
    
    println!("🎵 Rust Beats - Procedural Music Generator");
    println!("============================================");
    println!("Artist: {}", config.metadata.artist);
    println!("Sample Rate: {} Hz", config.audio.sample_rate);
    println!("Structure: {}\n", config.composition.structure);

    let output_dir = &config.generation.output_dir;
    fs::create_dir_all(output_dir).expect("Could not create output directory");

    // Generate song identity
    let song_name = generate_song_name();
    let genre_tags = generate_genre_tags();
    
    println!("📝 Song Name: {}", song_name);
    println!("🎸 Genres: {:?}", genre_tags);

    // Select genre and get its configuration
    let genre = select_random_genre();
    let genre_config = get_genre_config(genre);
    
    // Generate musical parameters based on genre - use genre config
    let key = if !genre_config.preferred_scales.is_empty() {
        let mut rng = rand::thread_rng();
        let scale = genre_config.preferred_scales[rng.gen_range(0..genre_config.preferred_scales.len())];
        Key::from_scale(scale)
    } else {
        Key::random_funky()
    };
    // Clamp tempo to config limits (respect user's min/max preferences)
    // Ensure valid range: take intersection of genre range and config range
    let tempo_min = genre_config.tempo_min.max(config.composition.min_tempo);
    let tempo_max = genre_config.tempo_max.min(config.composition.max_tempo);
    // Ensure min < max (in case genre range doesn't overlap with config)
    // If ranges don't overlap or are invalid, use config range as fallback
    let (tempo_min, tempo_max) = if tempo_min < tempo_max {
        (tempo_min, tempo_max)
    } else {
        // Genre range doesn't overlap with config - use config range
        // Ensure config range is valid
        let cfg_min = config.composition.min_tempo;
        let cfg_max = config.composition.max_tempo;
        if cfg_min < cfg_max {
            (cfg_min, cfg_max)
        } else {
            // Config is invalid, use a safe default
            eprintln!("⚠️  Warning: Invalid tempo range in config ({} >= {}), using default 80-120 BPM", cfg_min, cfg_max);
            (80.0, 120.0)
        }
    };
    let tempo = Tempo::random_funky_range(tempo_min, tempo_max);
    
    // Map genre to groove style for drum patterns
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
    
    println!("🎹 Key: Root MIDI {}, Scale: {:?}", key.root, key.scale_type);
    println!("⏱️  Tempo: {:.1} BPM", tempo.bpm);
    println!("🎵 Genre: {:?}", genre);
    println!("🥁 Groove: {:?}", groove_style);
    
    // Per-song instrument and style selection for variety
    let mut rng = rand::thread_rng();
    
    // Select lead instrument using config probabilities
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
    
    // Select bass type based on genre config
    let bass_type = match genre_config.bass_style {
        BassStyle::Standard => "Standard",
        BassStyle::Rock => "Rock",
        BassStyle::Synth => "Synth",
        BassStyle::Upright => "Upright",
        BassStyle::Finger => "Finger",
        BassStyle::Slap => "Slap",
        BassStyle::Wobble => "Wobble",
        BassStyle::Reese => "Reese",
    };
    
    // Select drum kit
    let drum_kit = select_random_drum_kit();
    
    // Percussion additions using config probability
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
    
    // Pad intensity using config probabilities
    let pad_cfg = &config.composition.pads;
    let pad_intensity = {
        let roll: f32 = rng.gen_range(0.0..1.0);
        if roll < pad_cfg.subtle { "Subtle" }
        else if roll < pad_cfg.subtle + pad_cfg.medium { "Medium" }
        else { "Prominent" }
    };
    
    // Mixing style using config probabilities
    let mix_cfg = &config.composition.mixing;
    let mixing_style = {
        let roll: f32 = rng.gen_range(0.0..1.0);
        if roll < mix_cfg.clean { "Clean" }
        else if roll < mix_cfg.clean + mix_cfg.warm { "Warm" }
        else if roll < mix_cfg.clean + mix_cfg.warm + mix_cfg.punchy { "Punchy" }
        else { "Spacious" }
    };
    
    println!("🎸 Lead: {}, Bass: {}, Drums: {:?}", lead_instrument, bass_type, drum_kit);
    println!("🥁 Percussion: {}, Pads: {}, Mix: {}\n", percussion_type, pad_intensity, mixing_style);

    // Generate song arrangement
    let arrangement = if config.composition.structure == "short" {
        Arrangement::generate_short()
    } else {
        Arrangement::generate_standard()
    };
    
    let num_bars = arrangement.total_bars;
    println!("🎼 Generating {} bars of music...", num_bars);
    println!("   Structure: {} sections", arrangement.sections.len());
    for (section, bars) in &arrangement.sections {
        println!("   {:?}: {} bars", section, bars);
    }
    println!();

    // Generate chord progression for entire song - use genre preferred chord types
    let chords = if !genre_config.preferred_chord_types.is_empty() {
        composition::music_theory::generate_chord_progression_with_types(&key, num_bars, Some(&genre_config.preferred_chord_types))
    } else {
        generate_chord_progression(&key, num_bars)
    };

    // Generate drums with arrangement awareness
    println!("  ├─ Drums (with dynamics)");
    let drums = render_arranged_drums(&arrangement, groove_style, tempo.bpm, drum_kit, &genre);

    // Generate bassline with arrangement awareness and genre-specific routing
    println!("  ├─ Bass (with sections)");
    let bassline = render_arranged_bass(&arrangement, &chords, tempo.bpm, &genre, &config.composition.bass_drops, &genre_config);

    // Generate melody with arrangement awareness (double tracking for stereo width)
    println!("  ├─ Melody (with variation, double-tracked)");
    let melody_l = render_arranged_melody(&arrangement, &key, &chords, tempo.bpm, &config.composition.melody, &genre);
    let melody_r = render_arranged_melody(&arrangement, &key, &chords, tempo.bpm, &config.composition.melody, &genre);
    
    // Generate atmospheric pads for appropriate sections (double tracking for stereo width)
    println!("  ├─ Pads (atmospheric, double-tracked)");
    let pads_l = generate_pads_with_arrangement(&arrangement, &chords, tempo.bpm, num_bars);
    let pads_r = generate_pads_with_arrangement(&arrangement, &chords, tempo.bpm, num_bars);
    
    // Generate transition effects (risers, crashes, impacts)
    println!("  ├─ Transition FX (risers & crashes)");
    let fx_track = render_fx_track(&arrangement, tempo.bpm);

    // Professional multi-track mixing with style variations
    println!("  ├─ Multi-track mixing ({})", mixing_style);
    
    // Adjust mixing parameters based on style
    let (drum_vol, drum_eq_l, drum_eq_m, drum_eq_h) = match mixing_style {
        "Clean" => (1.1, 1.2, 1.0, 1.0),   // Clean, balanced
        "Warm" => (1.0, 1.5, 0.9, 0.7),    // More bass, less treble
        "Punchy" => (1.3, 1.4, 1.1, 0.9),  // Louder, emphasized low-mid
        "Spacious" => (1.0, 1.1, 0.95, 1.1), // Airier, more high-end
        _ => (1.2, 1.4, 1.0, 0.9),
    };
    
    // Increased bass volume significantly (0.60-0.80) and boosted low frequencies for presence
    let (bass_vol, bass_eq_l, bass_eq_m, bass_eq_h) = match mixing_style {
        "Clean" => (0.65, 1.4, 0.9, 0.4),   // Increased volume, boosted lows
        "Warm" => (0.75, 1.6, 0.85, 0.3),   // Much warmer, more bass presence
        "Punchy" => (0.70, 1.5, 0.95, 0.4), // Punchy with strong lows
        "Spacious" => (0.60, 1.3, 0.85, 0.5), // Still present but airier
        _ => (0.68, 1.45, 0.9, 0.4),
    };
    
    let (melody_vol, melody_eq_l, melody_eq_m, melody_eq_h) = match mixing_style {
        "Clean" => (0.40, 0.85, 1.1, 1.05),
        "Warm" => (0.36, 0.95, 1.0, 0.9),  // Darker
        "Punchy" => (0.40, 0.9, 1.15, 1.0),
        "Spacious" => (0.35, 0.8, 1.0, 1.15), // Airier
        _ => (0.38, 0.9, 1.1, 1.0),
    };
    
    // Pad volume based on intensity
    let pad_vol = match pad_intensity {
        "Subtle" => 0.25,
        "Medium" => 0.40,
        "Prominent" => 0.55,
        _ => 0.35,
    };
    
    // Build tracks vector
    let mut tracks = vec![
        // Drums
        Track::new(drums)
            .with_volume(drum_vol)
            .with_pan(0.0)
            .with_eq(drum_eq_l, drum_eq_m, drum_eq_h),
        
        // Bass
        Track::new(bassline)
            .with_volume(bass_vol)
            .with_pan(0.0)
            .with_eq(bass_eq_l, bass_eq_m, bass_eq_h),
        
        // Melody (stereo doubled, double-tracked for authentic width)
        Track::new(melody_l)
            .with_volume(melody_vol)
            .with_pan(-0.20)  // Slightly wider
            .with_eq(melody_eq_l, melody_eq_m, melody_eq_h),
        
        Track::new(melody_r)
            .with_volume(melody_vol)
            .with_pan(0.20)   // Slightly wider
            .with_eq(melody_eq_l, melody_eq_m, melody_eq_h),
        
        // Pads (stereo wide, double-tracked for authentic width)
        Track::new(pads_l)
            .with_volume(pad_vol)
            .with_pan(-0.6)  // Wider stereo image
            .with_eq(0.8, 0.9, 0.8),
        
        Track::new(pads_r)
            .with_volume(pad_vol)
            .with_pan(0.6)   // Wider stereo image
            .with_eq(0.8, 0.9, 0.8),
    ];
    
    // Add percussion track if enabled
    if let Some(perc_track) = add_percussion_track(percussion_type, &arrangement, &tempo) {
        tracks.push(
            Track::new(perc_track)
                .with_volume(0.25)
                .with_pan(0.0)
                .with_eq(0.9, 1.0, 1.1)
        );
    }
    
    // Add transition FX track
    tracks.push(
        Track::new(fx_track)
            .with_volume(0.35)  // Moderate volume for FX
            .with_pan(0.0)      // Center
            .with_eq(0.7, 1.0, 1.2) // Emphasize highs for sweeps
    );
    
    let mut stereo_mix = mix_tracks(tracks);
    
    // Apply arrangement-aware dynamics (volume automation)
    println!("  ├─ Arrangement dynamics (volume automation)");
    apply_arrangement_dynamics(&mut stereo_mix, &arrangement, tempo.bpm);
    
    // Apply lofi mastering chain (compression, warmth, limiting)
    println!("  ├─ Lofi mastering (compression, warmth & limiting)");
    master_lofi(&mut stereo_mix, 0.70, 0.5);  // Target 70% loudness, medium lofi intensity
    
    // Apply lofi effects (vinyl crackle, tape saturation)
    println!("  ├─ Lofi effects (vinyl crackle & tape saturation)");
    let mut final_mix = stereo_to_mono(&stereo_mix);
    
    // Apply lofi processing based on genre - heavier for lofi genre
    // Disable wow/flutter to prevent alien scratching sounds
    let mut lofi_processor = match genre {
        Genre::Lofi => {
            // Heavy lofi for lofi genre, but disable wow/flutter
            let mut proc = if rng.gen_range(0..100) < 50 {
                LofiProcessor::heavy()
            } else {
                LofiProcessor::medium()
            };
            proc.wow_flutter_intensity = 0.0; // Disable wow/flutter - causes scratching
            proc
        }
        Genre::Jazz | Genre::Funk => {
            // Medium for jazz/funk, but disable wow/flutter
            let mut proc = LofiProcessor::medium();
            proc.wow_flutter_intensity = 0.0; // Disable wow/flutter
            proc
        }
        _ => LofiProcessor::subtle(),  // Subtle for other genres (already has no wow/flutter)
    };
    lofi_processor.process(&mut final_mix);
    
    // Normalize loudness based on RMS measurement (more aggressive)
    println!("  ├─ Loudness normalization");
    let final_rms = normalize_loudness(&mut final_mix, 0.25, 0.18);
    println!("     RMS: {:.3} (target: 0.25, min: 0.18)", final_rms);
    
    println!("  └─ Finalizing\n");

    // Get current date (YYYY-MM-DD format)
    // Use environment variable if set (for testing with specific dates), otherwise use current date
    let date = std::env::var("SONG_DATE")
        .unwrap_or_else(|_| {
            // Calculate current date properly
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now();
            let duration = now.duration_since(UNIX_EPOCH).unwrap();
            let secs = duration.as_secs();
            
            // Convert seconds to days since epoch
            let days = secs / 86400;
            
            // Calculate year (accounting for leap years)
            let mut year = 1970;
            let mut remaining_days = days;
            
            loop {
                let days_in_year = if is_leap_year(year) { 366 } else { 365 };
                if remaining_days < days_in_year {
                    break;
                }
                remaining_days -= days_in_year;
                year += 1;
            }
            
            // Calculate month and day
            let days_in_months = if is_leap_year(year) {
                [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            } else {
                [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            };
            
            let mut month = 1;
            let mut day = remaining_days as u32;
            
            for &days_in_month in &days_in_months {
                if day < days_in_month {
                    break;
                }
                day -= days_in_month;
                month += 1;
            }
            
            day += 1; // Days are 1-indexed
            
            format!("{:04}-{:02}-{:02}", year, month, day)
        });
    
    // Create metadata
    let metadata = SongMetadata {
        title: song_name.clone(),
        artist: config.metadata.artist.clone(),
        copyright: config.metadata.copyright.clone(),
        genre: genre_tags.clone(),
        date: date.clone(),
    };

    // Generate filename: {date}_{author}_{song_name}
    // Sanitize author and song name for filesystem safety
    let sanitized_author = sanitize_filename(&config.metadata.artist);
    let sanitized_song_name = sanitize_filename(&song_name);
    let filename_base = format!("{}_{}_{}", date, sanitized_author, sanitized_song_name);
    
    // Save the song
    let final_song_path = format!("{}/{}.wav", output_dir, filename_base);
    match render_to_wav_with_metadata(&final_mix, &final_song_path, &metadata) {
        Ok(_) => {
            println!("✅ Successfully created: {}", final_song_path);
            println!("   Duration: {:.1}s", final_mix.len() as f32 / SAMPLE_RATE as f32);
            println!("   Samples: {}", final_mix.len());
        }
        Err(e) => {
            eprintln!("❌ Error creating song: {}", e);
        }
    }

    // Generate MP3 file for smaller file size
    let mp3_path = format!("{}/{}.mp3", output_dir, filename_base);
    match encode_to_mp3(&final_mix, &mp3_path, &song_name, &config.metadata.artist) {
        Ok(_) => {
            println!("✅ Successfully created MP3: {}", mp3_path);
        }
        Err(e) => {
            eprintln!("⚠️  Warning: Could not create MP3: {}", e);
        }
    }

    // Save metadata for the workflow
    if config.generation.write_metadata_json {
        let metadata_path = format!("{}/{}.json", output_dir, filename_base);
        let metadata_json = serde_json::json!({
            "name": song_name,
            "artist": config.metadata.artist,
            "genre": genre_tags,
            "tempo": tempo.bpm,
            "duration": final_mix.len() as f32 / SAMPLE_RATE as f32,
            "date": date,
        });
        
        if let Err(e) = fs::write(&metadata_path, serde_json::to_string_pretty(&metadata_json).unwrap()) {
            eprintln!("⚠️  Warning: Could not write metadata: {}", e);
        }
    }

    println!("\n🎉 Song generation complete!");
    println!("   Name: {}", song_name);
    println!("   Style: {} @ {:.0} BPM", genre_tags.join(", "), tempo.bpm);
}

/// Render drums with arrangement awareness
fn render_arranged_drums(arrangement: &Arrangement, base_groove: GrooveStyle, bpm: f32, drum_kit: DrumKit, genre: &Genre) -> Vec<f32> {
    use rand::Rng;
    
    let mut all_drums = Vec::new();
    let mut bar_idx = 0;
    let mut rng = rand::thread_rng();
    
    for (section_idx, (section, bars)) in arrangement.sections.iter().enumerate() {
        let complexity = Arrangement::get_drum_complexity(*section);
        let intensity = Arrangement::get_section_intensity(*section);
        
        // Allow groove style variation per section (30% chance to change)
        let groove = if rng.gen_range(0..100) < 30 {
            // Map genre to appropriate groove styles
            match genre {
                Genre::Rock => GrooveStyle::Rock,
                Genre::Dubstep => GrooveStyle::Dubstep,
                Genre::DnB => GrooveStyle::DnB,
                Genre::Jazz => GrooveStyle::Jazz,
                Genre::Funk => GrooveStyle::Funk,
                Genre::HipHop => GrooveStyle::HipHop,
                Genre::ElectroSwing => GrooveStyle::ElectroSwing,
                Genre::Lofi => GrooveStyle::Lofi,
            }
        } else {
            base_groove
        };
        
        // Check for transitions
        let is_transition = arrangement.should_add_fill(bar_idx + bars - 1);
        let needs_buildup = if section_idx > 0 {
            let prev_section = arrangement.sections[section_idx - 1].0;
            Arrangement::needs_buildup(prev_section, *section)
        } else {
            false
        };
        let needs_breakdown = if section_idx > 0 {
            let prev_section = arrangement.sections[section_idx - 1].0;
            Arrangement::needs_breakdown(prev_section, *section)
        } else {
            false
        };
        
        // Generate pattern with complexity awareness
        let pattern = generate_drum_pattern_with_complexity(groove, *bars, complexity, is_transition, needs_buildup, needs_breakdown);
        
        // Render with kit-specific sounds and swing
        let drums = render_drum_pattern_with_kit(&pattern, bpm, intensity, drum_kit, genre);
        all_drums.extend(drums);
        
        bar_idx += bars;
    }
    
    all_drums
}

/// Generate drum pattern with complexity and transition awareness
fn generate_drum_pattern_with_complexity(
    groove: GrooveStyle,
    bars: usize,
    complexity: composition::arranger::DrumComplexity,
    add_fill: bool,
    needs_buildup: bool,
    needs_breakdown: bool,
) -> composition::beat_maker::DrumPattern {
    use composition::beat_maker::DrumHit;
    use composition::arranger::DrumComplexity;
    use rand::Rng;
    
    let mut pattern = generate_drum_pattern(groove, bars);
    let mut rng = rand::thread_rng();
    
    // Adjust pattern based on complexity
    match complexity {
        DrumComplexity::Simple => {
            // Simplify: remove some hi-hats and keep only kick/snare
            for step in pattern.iter_mut() {
                step.retain(|hit| matches!(hit, DrumHit::Kick | DrumHit::Snare | DrumHit::Rest));
            }
        }
        DrumComplexity::Medium => {
            // Keep as-is (already medium complexity)
        }
        DrumComplexity::Complex => {
            // Add more variation: occasional extra hits
            for step in pattern.iter_mut() {
                if rng.gen_range(0..100) < 15 {
                    if !step.contains(&DrumHit::HiHatClosed) {
                        step.push(DrumHit::HiHatClosed);
                    }
                }
            }
        }
    }
    
    // Add fill on last bar if needed
    if add_fill {
        let fill_start = pattern.len().saturating_sub(4); // Last 4 steps
        for i in fill_start..pattern.len() {
            if i % 2 == 0 {
                pattern[i].push(DrumHit::Snare);
            }
            if i % 3 == 0 {
                pattern[i].push(DrumHit::HiHatClosed);
            }
        }
    }
    
    // Add buildup: gradually increase density
    if needs_buildup {
        let buildup_start = pattern.len().saturating_sub(8); // Last 8 steps
        for (i, step) in pattern.iter_mut().enumerate().skip(buildup_start) {
            let progress = (i - buildup_start) as f32 / 8.0;
            if rng.gen_range(0.0..1.0) < progress {
                if !step.contains(&DrumHit::HiHatClosed) {
                    step.push(DrumHit::HiHatClosed);
                }
            }
        }
    }
    
    // Add breakdown: gradually decrease density
    if needs_breakdown {
        let breakdown_length = 8.min(pattern.len());
        for (i, step) in pattern.iter_mut().enumerate().take(breakdown_length) {
            let progress = i as f32 / breakdown_length as f32;
            if rng.gen_range(0.0..1.0) < (1.0 - progress) {
                step.retain(|hit| matches!(hit, DrumHit::Kick | DrumHit::Snare | DrumHit::Rest));
            }
        }
    }
    
    pattern
}

/// Render drum pattern with kit-specific sounds and swing
fn render_drum_pattern_with_kit(
    pattern: &[Vec<composition::beat_maker::DrumHit>],
    bpm: f32,
    intensity: f32,
    drum_kit: DrumKit,
    genre: &Genre,
) -> Vec<f32> {
    use composition::beat_maker::DrumHit;
    use synthesis::drums::{generate_rock_kick, generate_dubstep_kick, generate_dnb_snare, generate_rock_snare};
    
    let beat_duration = 60.0 / bpm;
    let sixteenth_duration = beat_duration / 4.0;
    
    // Genre-specific swing amount (applied to off-beats)
    let swing_amount = match genre {
        Genre::Lofi => 0.30,      // Heavy swing for laid-back feel
        Genre::Jazz => 0.35,      // Even more swing for jazz
        Genre::HipHop => 0.25,    // Medium swing
        Genre::Funk => 0.20,      // Subtle funk groove
        Genre::ElectroSwing => 0.28, // Swing for electro-swing
        Genre::DnB => 0.10,       // Slight swing for variation
        _ => 0.05,                // Minimal swing for rock/dubstep
    };
    
    let total_duration = pattern.len() as f32 * sixteenth_duration;
    let total_samples = (total_duration * SAMPLE_RATE as f32) as usize;
    
    let mut output = vec![0.0; total_samples];
    
    for (step_idx, hits) in pattern.iter().enumerate() {
        // Apply swing: delay every second 16th note (odd indices)
        let swing_offset = if step_idx % 2 == 1 {
            sixteenth_duration * swing_amount
        } else {
            0.0
        };
        
        let step_time = step_idx as f32 * sixteenth_duration + swing_offset;
        let start_sample = (step_time * SAMPLE_RATE as f32) as usize;
        
        for hit in hits {
            let drum_samples = match (hit, drum_kit) {
                (DrumHit::Kick, DrumKit::Rock) => generate_rock_kick(0.8 * intensity),
                (DrumHit::Kick, DrumKit::Electronic808) | (DrumHit::Kick, DrumKit::HipHop) => {
                    // Use dubstep kick for electronic kits
                    generate_dubstep_kick(0.8 * intensity)
                }
                (DrumHit::Kick, _) => generate_kick(0.8 * intensity),
                
                (DrumHit::Snare, DrumKit::Rock) => generate_rock_snare(0.7 * intensity),
                (DrumHit::Snare, DrumKit::Electronic808) | (DrumHit::Snare, DrumKit::HipHop) => {
                    // Use DnB snare for electronic kits
                    generate_dnb_snare(0.7 * intensity)
                }
                (DrumHit::Snare, _) => generate_snare(0.7 * intensity),
                
                (DrumHit::HiHatClosed, _) => generate_hihat(0.4 * intensity, false),
                (DrumHit::HiHatOpen, _) => generate_hihat(0.5 * intensity, true),
                (DrumHit::Clap, _) => generate_clap(0.6 * intensity),
                (DrumHit::Conga, _) => synthesis::generate_conga(200.0, 0.5 * intensity),
                (DrumHit::Shaker, _) => synthesis::generate_shaker(0.3 * intensity),
                (DrumHit::Rest, _) => continue,
            };
            
            // Mix the drum hit into the output
            for (i, &sample) in drum_samples.iter().enumerate() {
                let idx = start_sample + i;
                if idx < output.len() {
                    output[idx] += sample;
                }
            }
        }
    }
    
    output
}

/// Render bass with arrangement awareness and genre-specific routing
fn render_arranged_bass(arrangement: &Arrangement, chords: &[composition::Chord], bpm: f32, genre: &Genre, bass_drop_cfg: &config::BassDropConfig, genre_config: &composition::genre::GenreConfig) -> Vec<f32> {
    
    let mut all_bass = Vec::new();
    let mut bar_idx = 0;
    let mut rng = rand::thread_rng();
    
    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords.iter()
            .skip(bar_idx)
            .take(*bars)
            .cloned()
            .collect();
        
        if Arrangement::section_has_heavy_bass(*section) {
            // Route to genre-specific bass generator
            let bass = match genre {
                Genre::Rock => generate_rock_bassline(&section_chords, bpm, *bars, bass_drop_cfg),
                Genre::Dubstep => generate_dubstep_bassline(&section_chords, bpm, *bars),
                Genre::DnB => generate_dnb_bassline(&section_chords, bpm, *bars, bass_drop_cfg),
                Genre::Funk => {
                    // Use slap bass for funk sections
                    generate_bassline_with_style(&section_chords, bpm, *bars, bass_drop_cfg, Some(composition::genre::BassStyle::Slap))
                }
                Genre::Jazz => {
                    // Use upright bass for jazz
                    generate_bassline_with_style(&section_chords, bpm, *bars, bass_drop_cfg, Some(composition::genre::BassStyle::Upright))
                }
                _ => {
                    // Use genre config bass style
                    generate_bassline_with_style(&section_chords, bpm, *bars, bass_drop_cfg, Some(genre_config.bass_style))
                }
            };
            all_bass.extend(bass);
        } else {
            // Light bass or no bass - just add silence
            let bar_duration = 60.0 / bpm * 4.0;
            let silence_samples = (bar_duration * *bars as f32 * SAMPLE_RATE as f32) as usize;
            all_bass.extend(vec![0.0; silence_samples]);
        }
        
        bar_idx += bars;
    }
    
    all_bass
}

/// Generate bassline with specific bass style
fn generate_bassline_with_style(
    chords: &[composition::Chord],
    tempo: f32,
    bars: usize,
    bass_drop_cfg: &config::BassDropConfig,
    style: Option<composition::genre::BassStyle>,
) -> Vec<f32> {
    use synthesis::bass::{generate_synth_bass_note, generate_upright_bass_note, generate_finger_bass_note, generate_slap_bass_note, generate_bass_note};
    use composition::music_theory::midi_to_freq;
    use rand::Rng;
    
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;
    
    let mut bassline = Vec::new();
    let mut rng = rand::thread_rng();
    
    // Select bass note generator based on style
    let bass_note_fn: fn(f32, f32, f32) -> Vec<f32> = match style {
        Some(composition::genre::BassStyle::Synth) => generate_synth_bass_note,
        Some(composition::genre::BassStyle::Upright) => generate_upright_bass_note,
        Some(composition::genre::BassStyle::Finger) => generate_finger_bass_note,
        Some(composition::genre::BassStyle::Slap) => generate_slap_bass_note,
        _ => generate_bass_note, // Default
    };
    
    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        let root_note = chord.root;
        let frequency = midi_to_freq(root_note);
        
        // Generate funky bass pattern for this bar
        let mut pattern = generate_funk_bass_pattern_with_style(frequency, bar_duration, bass_note_fn);
        
        // Add occasional bass drop using config values
        let should_drop = if bar_idx > 0 && bar_idx % 8 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.default_chance_8th_bar
        } else if bar_idx > 0 && bar_idx % 12 == 0 {
            rng.gen_range(0.0..1.0) < bass_drop_cfg.default_chance_12th_bar
        } else {
            false
        };
        
        if should_drop {
            // Add a sub-bass drop on beat 1 (start of bar)
            let drop_freq = frequency * 0.5; // One octave down
            let drop_duration = beat_duration * bass_drop_cfg.default_duration_beats;
            let drop = synthesis::bass::generate_sub_bass(drop_freq, drop_duration, bass_drop_cfg.amplitude);
            
            // Mix the drop into the pattern
            for (i, &drop_sample) in drop.iter().enumerate() {
                if i < pattern.len() {
                    pattern[i] += drop_sample;
                }
            }
        }
        
        bassline.extend(pattern);
    }
    
    bassline
}

/// Generate a funky bass pattern with custom bass note generator
fn generate_funk_bass_pattern_with_style(
    root_freq: f32,
    bar_duration: f32,
    bass_note_fn: fn(f32, f32, f32) -> Vec<f32>,
) -> Vec<f32> {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let note_duration = bar_duration / 8.0; // 8th notes
    
    let mut pattern = Vec::new();
    
    // Typical funk bass pattern: emphasize 1 and 3, add syncopation
    let hits = vec![
        (0.0, 1.0, 1.0),      // Beat 1 - strong
        (0.5, 0.5, 0.7),      // Off-beat
        (1.0, 0.8, 0.8),      // Beat 2
        (1.75, 0.4, 0.6),     // Syncopation
        (2.0, 1.0, 1.0),      // Beat 3 - strong
        (2.5, 0.5, 0.7),      // Off-beat
        (3.0, 0.6, 0.8),      // Beat 4
        (3.5, 0.4, 0.6),      // Off-beat
    ];
    
    for (beat_pos, duration_mult, velocity) in hits {
        let start_time = beat_pos * (bar_duration / 4.0);
        let duration = note_duration * duration_mult;
        
        // Mostly stick to root note for consistency
        let freq_mult = match rng.gen_range(0..100) {
            0..=92 => 1.0,        // Root (most of the time)
            93..=97 => 0.5,       // Octave down (rare)
            _ => 1.5,             // Fifth (very rare)
        };
        
        let note = bass_note_fn(
            root_freq * freq_mult,
            duration,
            velocity * 0.65,
        );
        
        // Add to pattern at the right position
        let start_sample = (start_time * SAMPLE_RATE as f32) as usize;
        if start_sample + note.len() > pattern.len() {
            pattern.resize(start_sample + note.len(), 0.0);
        }
        
        for (i, &sample) in note.iter().enumerate() {
            let idx = start_sample + i;
            if idx < pattern.len() {
                pattern[idx] += sample;
            }
        }
    }
    
    // Fill to full bar duration
    let total_samples = (bar_duration * SAMPLE_RATE as f32) as usize;
    pattern.resize(total_samples, 0.0);
    
    pattern
}

/// Generate pads with arrangement awareness (add drones for intro/outro)
fn generate_pads_with_arrangement(arrangement: &Arrangement, chords: &[composition::Chord], bpm: f32, num_bars: usize) -> Vec<f32> {
    // Temporarily disable drones - they might be causing the scratching issue
    // Just use regular pads for now
    generate_pads(chords, bpm, num_bars)
}

/// Render melody with arrangement awareness
fn render_arranged_melody(arrangement: &Arrangement, key: &Key, chords: &[composition::Chord], bpm: f32, melody_cfg: &config::MelodyConfig, genre: &Genre) -> Vec<f32> {
    use synthesis::melody::generate_melody_with_style;
    
    let mut all_melody = Vec::new();
    let mut bar_idx = 0;
    
    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords.iter()
            .skip(bar_idx)
            .take(*bars)
            .cloned()
            .collect();
        
        if Arrangement::section_has_melody(*section) {
            // Use new melody generation with style awareness
            let melody = generate_melody_with_style(key, &section_chords, bpm, *bars, melody_cfg, Some(*section), Some(*genre));
            all_melody.extend(melody);
        } else {
            // No melody - just add silence
            let bar_duration = 60.0 / bpm * 4.0;
            let silence_samples = (bar_duration * *bars as f32 * SAMPLE_RATE as f32) as usize;
            all_melody.extend(vec![0.0; silence_samples]);
        }
        
        bar_idx += bars;
    }
    
    all_melody
}

/// Apply volume automation based on arrangement sections
/// Intro/Outro: quieter, Verse: normal, Chorus: louder, Bridge: medium
/// Works with interleaved stereo buffer (L, R, L, R, ...)
fn apply_arrangement_dynamics(stereo_buffer: &mut Vec<f32>, arrangement: &Arrangement, bpm: f32) {
    use composition::Section;
    
    let samples_per_bar = (SAMPLE_RATE as f32 * 60.0 / bpm * 4.0) as usize * 2; // *2 for stereo
    let mut current_sample = 0;
    
    for (section_type, bars) in &arrangement.sections {
        let section_samples = samples_per_bar * bars;
        let section_end = (current_sample + section_samples).min(stereo_buffer.len());
        
        // Determine volume multiplier for this section
        let base_volume = match section_type {
            Section::Intro => 0.75,   // Quieter intro
            Section::Verse => 0.95,   // Standard verse
            Section::Chorus => 1.15,  // Louder chorus
            Section::Bridge => 0.90,  // Medium bridge
            Section::Outro => 0.70,   // Quiet outro
        };
        
        // Apply volume to section with smooth transitions
        let fade_samples = samples_per_bar / 4; // Quarter bar fade
        
        for i in (current_sample..section_end).step_by(2) {
            if i + 1 >= stereo_buffer.len() {
                break;
            }
            
            let progress_in_section = i - current_sample;
            let samples_remaining = section_end - i;
            
            // Smooth fade in/out
            let mut volume = base_volume;
            
            // Fade in at section start
            if progress_in_section < fade_samples {
                let fade_in = progress_in_section as f32 / fade_samples as f32;
                volume *= fade_in;
            }
            
            // Fade out at section end (especially for outro)
            if *section_type == Section::Outro && samples_remaining < section_samples / 2 {
                let fade_out = samples_remaining as f32 / (section_samples as f32 / 2.0);
                volume *= fade_out.max(0.3); // Don't fade completely to silence
            }
            
            // Apply volume to both channels
            stereo_buffer[i] *= volume;
            stereo_buffer[i + 1] *= volume;
        }
        
        current_sample = section_end;
    }
}

/// Render a drum pattern with intensity control
fn render_drum_pattern_with_intensity(pattern: &[Vec<DrumHit>], bpm: f32, intensity: f32) -> Vec<f32> {
    // Use Tempo helper for timing
    let tempo = Tempo { bpm };
    let beat_duration = tempo.beat_duration();
    let sixteenth_duration = beat_duration / 4.0;
    let total_duration = pattern.len() as f32 * sixteenth_duration;
    let total_samples = (total_duration * SAMPLE_RATE as f32) as usize;
    
    let mut output = vec![0.0; total_samples];
    
    for (step_idx, hits) in pattern.iter().enumerate() {
        let step_time = step_idx as f32 * sixteenth_duration;
        let start_sample = (step_time * SAMPLE_RATE as f32) as usize;
        
        for hit in hits {
            let drum_samples = match hit {
                DrumHit::Kick => generate_kick(0.8 * intensity),
                DrumHit::Snare => generate_snare(0.7 * intensity),
                DrumHit::HiHatClosed => generate_hihat(0.4 * intensity, false),
                DrumHit::HiHatOpen => generate_hihat(0.5 * intensity, true),
                DrumHit::Clap => generate_clap(0.6 * intensity),
                DrumHit::Conga => synthesis::generate_conga(200.0, 0.5 * intensity),
                DrumHit::Shaker => synthesis::generate_shaker(0.3 * intensity),
                DrumHit::Rest => continue,
            };
            
            // Mix the drum hit into the output
            for (i, &sample) in drum_samples.iter().enumerate() {
                let idx = start_sample + i;
                if idx < output.len() {
                    output[idx] += sample;
                }
            }
        }
    }
    
    output
}

/// Render transition FX track (risers, crashes, downlifters)
fn render_fx_track(arrangement: &Arrangement, bpm: f32) -> Vec<f32> {
    let bar_duration = 60.0 / bpm * 4.0;
    let total_samples = (bar_duration * arrangement.total_bars as f32 * SAMPLE_RATE as f32) as usize;
    let mut fx_track = vec![0.0; total_samples];
    
    let mut current_bar = 0;
    
    for (section_idx, (section, bars)) in arrangement.sections.iter().enumerate() {
        let section_start_sample = (current_bar as f32 * bar_duration * SAMPLE_RATE as f32) as usize;
        let section_end_sample = ((current_bar + bars) as f32 * bar_duration * SAMPLE_RATE as f32) as usize;
        
        // Check if we need a buildup to this section
        if section_idx > 0 {
            let prev_section = arrangement.sections[section_idx - 1].0;
            
            if Arrangement::needs_buildup(prev_section, *section) {
                // Add riser before section starts (2 bars)
                let riser_duration = bar_duration * 2.0;
                let riser = generate_riser(riser_duration);
                let riser_start = section_start_sample.saturating_sub(riser.len());
                
                for (i, &sample) in riser.iter().enumerate() {
                    let idx = riser_start + i;
                    if idx < fx_track.len() {
                        fx_track[idx] += sample;
                    }
                }
            }
            
            if Arrangement::needs_breakdown(prev_section, *section) {
                // Add downlifter at start of section (1 bar)
                let downlifter_duration = bar_duration;
                let downlifter = generate_downlifter(downlifter_duration);
                
                for (i, &sample) in downlifter.iter().enumerate() {
                    let idx = section_start_sample + i;
                    if idx < fx_track.len() {
                        fx_track[idx] += sample;
                    }
                }
            }
        }
        
        // Add crash at the start of important sections (Chorus, Bridge)
        if matches!(*section, composition::Section::Chorus | composition::Section::Bridge) {
            let crash = generate_crash(2.0);
            
            for (i, &sample) in crash.iter().enumerate() {
                let idx = section_start_sample + i;
                if idx < fx_track.len() {
                    fx_track[idx] += sample * 0.7; // Slightly quieter
                }
            }
        }
        
        // Add impact at very start of Chorus for emphasis
        if *section == composition::Section::Chorus {
            let impact = generate_impact();
            
            for (i, &sample) in impact.iter().enumerate() {
                let idx = section_start_sample + i;
                if idx < fx_track.len() {
                    fx_track[idx] += sample * 0.8;
                }
            }
        }
        
        current_bar += bars;
    }
    
    fx_track
}

/// Add percussion track if enabled
fn add_percussion_track(percussion_type: &str, arrangement: &Arrangement, tempo: &Tempo) -> Option<Vec<f32>> {
    use synthesis::percussion::{generate_tambourine, generate_cowbell, generate_bongo, generate_woodblock, generate_triangle_perc};
    use rand::Rng;
    
    if percussion_type == "None" {
        return None;
    }
    
    let mut percussion = Vec::new();
    let mut rng = rand::thread_rng();
    let bar_duration = tempo.bar_duration();
    let beat_duration = tempo.beat_duration();
    
    for (section, bars) in &arrangement.sections {
        if Arrangement::section_has_melody(*section) {
            // Add percussion to sections with melody
            for bar_idx in 0..*bars {
                let bar_samples = (bar_duration * SAMPLE_RATE as f32) as usize;
                let mut bar = vec![0.0; bar_samples];
                
                // Add percussion hits on beats
                for beat in 0..4 {
                    if rng.gen_range(0..100) < 40 { // 40% chance per beat
                        let hit_time = beat as f32 * beat_duration;
                        let hit_sample = (hit_time * SAMPLE_RATE as f32) as usize;
                        
                        let hit = match percussion_type {
                            "Tambourine" => generate_tambourine(0.4),
                            "Cowbell" => generate_cowbell(0.5),
                            "Bongo" => generate_bongo(rng.gen_range(0..100) < 50, 0.4),
                            "Woodblock" => generate_woodblock(0.4),
                            _ => generate_triangle_perc(0.3),
                        };
                        
                        for (i, &sample) in hit.iter().enumerate() {
                            let idx = hit_sample + i;
                            if idx < bar.len() {
                                bar[idx] += sample;
                            }
                        }
                    }
                }
                
                percussion.extend(bar);
            }
        } else {
            // Silence for sections without melody
            let silence_samples = (bar_duration * *bars as f32 * SAMPLE_RATE as f32) as usize;
            percussion.extend(vec![0.0; silence_samples]);
        }
    }
    
    Some(percussion)
}
