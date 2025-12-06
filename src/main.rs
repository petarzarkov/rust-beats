mod audio;
mod composition;
mod config;
mod synthesis;
mod utils;

use audio::encode_to_mp3;
use composition::{
    generate_song_name, generate_genre_tags,
    metal_song_generator::{MetalSongGenerator, MetalSubgenre},
};
use config::Config;
use synthesis::{get_sample_rate, init_sample_rate, metal_audio_renderer::MetalAudioRenderer};
use utils::{get_current_date, sanitize_filename};
use std::fs;

fn main() {
    println!("🤘 RUST BEATS - METAL MUSIC GENERATOR 🤘");
    println!("=========================================\n");

    // Load configuration
    let config = Config::load_default().unwrap_or_else(|e| {
        eprintln!("⚠️  Warning: Could not load config.toml: {}", e);
        eprintln!("   Using default configuration\n");
        Config::default()
    });

    // Initialize sample rate from config
    init_sample_rate(config.audio.sample_rate);
    
    println!("Artist: {}", config.metadata.artist);
    println!("Sample Rate: {} Hz\n", config.audio.sample_rate);
    
    // Choose a random metal subgenre
    let subgenres = vec![
        MetalSubgenre::HeavyMetal,
        MetalSubgenre::ThrashMetal,
        MetalSubgenre::DeathMetal,
        MetalSubgenre::DoomMetal,
        MetalSubgenre::ProgressiveMetal,
    ];
    
    let subgenre = subgenres[rand::random::<usize>() % subgenres.len()];
    
    println!("🎸 Generating {:?} song...\n", subgenre);
    
    // Generate song name and genre tags
    let song_name = generate_song_name();
    let genre = composition::Genre::SwampMetal; // Map to our genre system
    let genre_tags = generate_genre_tags(genre);
    
    // Generate the song structure
    let generator = MetalSongGenerator::new(subgenre);
    let song = generator.generate_song();
    
    println!("📝 Song Details:");
    println!("   Name: {}", song_name);
    println!("   Genre: {}", genre_tags.join(", "));
    println!("   Subgenre: {:?}", song.subgenre);
    println!("   Key: {:?} {:?}", song.key.root, song.key.scale_type);
    println!("   Tempo: {} BPM", song.tempo);
    println!("   Tuning: {:?}", song.tuning);
    println!("   Sections: {}", song.sections.len());
    println!();
    
    // Print section breakdown
    println!("🎼 Song Structure:");
    for (i, (section, riff)) in song.sections.iter().enumerate() {
        println!("   {}. {:?} - {} notes", i + 1, section, riff.notes.len());
    }
    println!();
    
    // Render the audio (MULTITRACK)
    println!("🔊 Rendering audio (multi-track)...");
    let mut renderer = MetalAudioRenderer::new();
    
    // Calculate variable durations for each section
    let mut total_duration = 0.0;
    let mut section_durations = Vec::new();
    
    for (section, _) in &song.sections {
        let duration = get_section_duration(*section, song.tempo);
        section_durations.push(duration);
        total_duration += duration;
    }
    
    println!("   Estimated Duration: {:.1}s ({:.1} min)", total_duration, total_duration / 60.0);
    
    // PARALLEL MULTITRACK RENDERING using multi-threading
    use rayon::prelude::*;
    
    let section_data: Vec<_> = song.sections.iter()
        .zip(section_durations.iter())
        .map(|((section, riff), duration)| (*section, riff.clone(), *duration, song.tempo, song.subgenre))
        .collect();
    
    // Render sections in parallel
    let section_tracks: Vec<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>)> = section_data.par_iter()
        .map(|(section, riff, duration, tempo, subgenre)| {
            let mut local_renderer = MetalAudioRenderer::new();
            local_renderer.render_section_separate_tracks(*section, riff, *duration, *tempo, *subgenre)
        })
        .collect();
    
    // Concatenate all sections
    let mut guitar_track = Vec::new();
    let mut bass_track = Vec::new();
    let mut drum_track = Vec::new();
    let mut melody_track = Vec::new();
    
    for (guitar, bass, drums, melody) in section_tracks {
        guitar_track.extend(guitar);
        bass_track.extend(bass);
        drum_track.extend(drums);
        melody_track.extend(melody);
    }
    
    // Create master mix from separate tracks (in parallel)
    let mut audio_samples = vec![0.0; guitar_track.len()];
    audio_samples.par_iter_mut().enumerate().for_each(|(i, sample)| {
        let guitar = *guitar_track.get(i).unwrap_or(&0.0) * 0.85;  // LOUD
        let bass = *bass_track.get(i).unwrap_or(&0.0) * 0.58;      // Medium
        let drums = *drum_track.get(i).unwrap_or(&0.0) * 0.45;     // Support
        let melody = *melody_track.get(i).unwrap_or(&0.0) * 0.65;  // Clear
        
        *sample = guitar + bass + drums + melody;
    });
    
    // Apply final limiter to master mix
    let peak = audio_samples.par_iter().map(|&s| s.abs()).reduce(|| 0.0, f32::max);
    if peak > 0.0 {
        let scale = 0.95 / peak;
        audio_samples.par_iter_mut().for_each(|sample| {
            *sample *= scale;
        });
    }
    
    let duration_seconds = audio_samples.len() as f32 / get_sample_rate() as f32;
    println!("   Duration: {:.1}s", duration_seconds);
    println!();
    
    // Save ONLY the master mix (no individual tracks)
    println!("💾 Saving audio...");
    let date = get_current_date();
    
    // Create sanitized filename
    let sanitized_artist = sanitize_filename(&config.metadata.artist);
    let sanitized_song_name = sanitize_filename(&song_name);
    let filename_base = format!("{}_{}_{}",  date, sanitized_artist, sanitized_song_name);
    
    // Create output directory
    let output_dir = &config.generation.output_dir;
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("❌ Error creating output directory: {}", e);
        return;
    }
    
    let wav_path = format!("{}/{}.wav", output_dir, filename_base);
    let mp3_path = format!("{}/{}.mp3", output_dir, filename_base);
    let json_path = format!("{}/{}.json", output_dir, filename_base);
    
    // Save WAV file
    match save_wav(&wav_path, &audio_samples, get_sample_rate()) {
        Ok(_) => println!("✅ Successfully created: {}", wav_path),
        Err(e) => {
            eprintln!("❌ Error saving WAV file: {}", e);
            return;
        }
    }
    
    // Save MP3 file (if enabled in config)
    if config.generation.encode_mp3 {
        match encode_to_mp3(&audio_samples, &mp3_path, &song_name, &config.metadata.artist) {
            Ok(_) => println!("✅ Successfully created: {}", mp3_path),
            Err(e) => eprintln!("⚠️  Warning: Could not create MP3: {}", e),
        }
    }
    
    // Save JSON metadata (if enabled in config)
    if config.generation.write_metadata_json {
        let metadata = serde_json::json!({
            "name": song_name,
            "artist": config.metadata.artist,
            "genre": genre_tags,
            "tempo": song.tempo as f32,
            "duration": duration_seconds,
            "date": date,
            "subgenre": format!("{:?}", song.subgenre),
            "key": format!("{:?} {:?}", song.key.root, song.key.scale_type),
            "tuning": format!("{:?}", song.tuning),
            "sections": song.sections.len(),
        });
        
        match fs::write(&json_path, serde_json::to_string_pretty(&metadata).unwrap()) {
            Ok(_) => println!("✅ Successfully created: {}", json_path),
            Err(e) => eprintln!("⚠️  Warning: Could not write metadata: {}", e),
        }
    }
    
    println!();
    println!("🎉 Metal song generation complete!");
    println!("   Name: {}", song_name);
    println!("   Artist: {}", config.metadata.artist);
    println!("   Style: {:?}", subgenre);
    println!("   Tempo: {} BPM", song.tempo);
    println!("   Duration: {:.1}s", duration_seconds);
}

/// Save audio samples to a WAV file
fn save_wav(filename: &str, samples: &[f32], sample_rate: u32) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(filename)?;
    
    // WAV header
    let num_samples = samples.len() as u32;
    let byte_rate = sample_rate * 2; // 16-bit mono
    let data_size = num_samples * 2;
    let file_size = data_size + 36;
    
    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    
    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // chunk size
    file.write_all(&1u16.to_le_bytes())?;  // audio format (PCM)
    file.write_all(&1u16.to_le_bytes())?;  // num channels (mono)
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?;  // block align
    file.write_all(&16u16.to_le_bytes())?; // bits per sample
    
    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    
    // Write audio data (convert f32 to i16)
    for &sample in samples {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        file.write_all(&sample_i16.to_le_bytes())?;
    }
    
    Ok(())
}

/// Calculate section duration based on bars and tempo
fn get_section_duration(section: composition::metal_song_generator::MetalSection, tempo: u16) -> f32 {
    use composition::metal_song_generator::MetalSection;
    
    let bars = match section {
        MetalSection::Intro => 4,
        MetalSection::Verse => 8,
        MetalSection::Chorus => 8,
        MetalSection::Breakdown => 4,
        MetalSection::Solo => 12,
        MetalSection::Outro => 4,
    };
    
    // Calculate duration: bars * beats_per_bar * seconds_per_beat
    let beats_per_bar = 4.0;
    let seconds_per_beat = 60.0 / tempo as f32;
    bars as f32 * beats_per_bar * seconds_per_beat
}

