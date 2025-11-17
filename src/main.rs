mod composition;
mod synthesis;
mod audio;
mod config;

use std::fs;
use rand::Rng;
use composition::{
    generate_song_name, generate_genre_tags,
    Key, Tempo, generate_chord_progression,
    generate_drum_pattern, random_groove_style, DrumHit, GrooveStyle, select_random_drum_kit,
    Arrangement,
};
use synthesis::{
    generate_kick, generate_snare, generate_hihat, generate_clap,
    generate_bassline, generate_melody, generate_pads,
    SAMPLE_RATE, LofiProcessor,
};
use audio::{render_to_wav_with_metadata, encode_to_mp3, SongMetadata, Track, mix_tracks, master_lofi, stereo_to_mono};
use config::Config;

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

    // Generate musical parameters
    let key = Key::random_funky();
    let tempo = Tempo::random_funky_range(config.composition.min_tempo, config.composition.max_tempo);
    let groove_style = random_groove_style();
    
    println!("🎹 Key: Root MIDI {}, Scale: {:?}", key.root, key.scale_type);
    println!("⏱️  Tempo: {:.1} BPM", tempo.bpm);
    println!("🥁 Groove: {:?}", groove_style);
    
    // Per-song instrument and style selection for variety
    let mut rng = rand::thread_rng();
    
    // Select lead instrument (Rhodes 40%, Ukulele 15%, Guitar 15%, etc.)
    let lead_instrument = {
        let roll: f32 = rng.gen_range(0.0..1.0);
        if roll < 0.40 { 
            "Rhodes" 
        } else if roll < 0.55 {
            "Ukulele"
        } else if roll < 0.70 {
            "Guitar"
        } else if roll < 0.85 {
            "Electric"
        } else {
            "Organ"
        }
    };
    
    // Select bass type (current 50%, synth 20%, upright 15%, finger 10%, slap 5%)
    let bass_type = {
        let roll: f32 = rng.gen_range(0.0..1.0);
        if roll < 0.50 {
            "Standard"
        } else if roll < 0.70 {
            "Synth"
        } else if roll < 0.85 {
            "Upright"
        } else if roll < 0.95 {
            "Finger"
        } else {
            "Slap"
        }
    };
    
    // Select drum kit
    let drum_kit = select_random_drum_kit();
    
    // Percussion additions (30% chance)
    let add_percussion = rng.gen_range(0.0..1.0) < 0.30;
    let percussion_type = if add_percussion {
        let roll: f32 = rng.gen_range(0.0..1.0);
        if roll < 0.33 { "Tambourine" } 
        else if roll < 0.66 { "Cowbell" }
        else { "Bongo" }
    } else {
        "None"
    };
    
    // Pad intensity (subtle 40%, medium 40%, prominent 20%)
    let pad_intensity = {
        let roll: f32 = rng.gen_range(0.0..1.0);
        if roll < 0.40 { "Subtle" }
        else if roll < 0.80 { "Medium" }
        else { "Prominent" }
    };
    
    // Mixing style
    let mixing_style = {
        let roll: f32 = rng.gen_range(0.0..1.0);
        if roll < 0.25 { "Clean" }
        else if roll < 0.50 { "Warm" }
        else if roll < 0.75 { "Punchy" }
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

    // Generate chord progression for entire song
    let chords = generate_chord_progression(&key, num_bars);

    // Generate drums with arrangement awareness
    println!("  ├─ Drums (with dynamics)");
    let drums = render_arranged_drums(&arrangement, groove_style, tempo.bpm);

    // Generate bassline with arrangement awareness
    println!("  ├─ Bass (with sections)");
    let bassline = render_arranged_bass(&arrangement, &chords, tempo.bpm);

    // Generate melody with arrangement awareness
    println!("  ├─ Melody (with variation)");
    let melody = render_arranged_melody(&arrangement, &key, &chords, tempo.bpm);
    
    // Generate atmospheric pads for appropriate sections
    println!("  ├─ Pads (atmospheric)");
    let pads = generate_pads(&chords, tempo.bpm, num_bars);

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
    
    let (bass_vol, bass_eq_l, bass_eq_m, bass_eq_h) = match mixing_style {
        "Clean" => (0.40, 1.0, 0.8, 0.5),
        "Warm" => (0.50, 1.3, 0.75, 0.4),  // Warmer, more bass
        "Punchy" => (0.42, 1.2, 0.85, 0.5),
        "Spacious" => (0.35, 0.9, 0.8, 0.6),
        _ => (0.45, 1.1, 0.8, 0.5),
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
    
    let tracks = vec![
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
        
        // Melody (stereo doubled)
        Track::new(melody.clone())
            .with_volume(melody_vol)
            .with_pan(-0.15)
            .with_eq(melody_eq_l, melody_eq_m, melody_eq_h),
        
        Track::new(melody)
            .with_volume(melody_vol)
            .with_pan(0.15)
            .with_eq(melody_eq_l, melody_eq_m, melody_eq_h),
        
        // Pads (stereo wide)
        Track::new(pads.clone())
            .with_volume(pad_vol)
            .with_pan(-0.5)
            .with_eq(0.8, 0.9, 0.8),
        
        Track::new(pads)
            .with_volume(pad_vol)
            .with_pan(0.5)
            .with_eq(0.8, 0.9, 0.8),
    ];
    
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
    
    // Apply subtle lofi processing
    let lofi_processor = LofiProcessor::subtle();
    lofi_processor.process(&mut final_mix);
    
    println!("  └─ Finalizing\n");

    // Get current date (YYYY-MM-DD format)
    let now = std::time::SystemTime::now();
    let duration_since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let secs = duration_since_epoch.as_secs();
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let remaining_days = days % 365;
    let month = (remaining_days / 30) + 1;
    let day = (remaining_days % 30) + 1;
    let date = format!("{:04}-{:02}-{:02}", years, month.min(12), day.min(31));
    
    // Create metadata
    let metadata = SongMetadata {
        title: song_name.clone(),
        artist: config.metadata.artist.clone(),
        copyright: config.metadata.copyright.clone(),
        genre: genre_tags.clone(),
        date: date.clone(),
    };

    // Save the song
    let final_song_path = format!("{}/final_song.wav", output_dir);
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
    let mp3_path = format!("{}/final_song.mp3", output_dir);
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
        let metadata_path = format!("{}/song_metadata.json", output_dir);
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
fn render_arranged_drums(arrangement: &Arrangement, groove: GrooveStyle, bpm: f32) -> Vec<f32> {
    let mut all_drums = Vec::new();
    
    for (section, bars) in &arrangement.sections {
        let pattern = generate_drum_pattern(groove, *bars);
        let intensity = Arrangement::get_section_intensity(*section);
        let drums = render_drum_pattern_with_intensity(&pattern, bpm, intensity);
        all_drums.extend(drums);
    }
    
    all_drums
}

/// Render bass with arrangement awareness
fn render_arranged_bass(arrangement: &Arrangement, chords: &[composition::Chord], bpm: f32) -> Vec<f32> {
    let mut all_bass = Vec::new();
    let mut bar_idx = 0;
    
    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords.iter()
            .skip(bar_idx)
            .take(*bars)
            .cloned()
            .collect();
        
        if Arrangement::section_has_heavy_bass(*section) {
            let bass = generate_bassline(&section_chords, bpm, *bars);
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

/// Render melody with arrangement awareness
fn render_arranged_melody(arrangement: &Arrangement, key: &Key, chords: &[composition::Chord], bpm: f32) -> Vec<f32> {
    let mut all_melody = Vec::new();
    let mut bar_idx = 0;
    
    for (section, bars) in &arrangement.sections {
        let section_chords: Vec<_> = chords.iter()
            .skip(bar_idx)
            .take(*bars)
            .cloned()
            .collect();
        
        if Arrangement::section_has_melody(*section) {
            let melody = generate_melody(key, &section_chords, bpm, *bars);
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
    let beat_duration = 60.0 / bpm;
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
