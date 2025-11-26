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
    
    // Render the audio
    println!("🔊 Rendering audio...");
    let mut renderer = MetalAudioRenderer::new();
    let duration_per_section = 4.0; // 4 seconds per section
    let audio_samples = renderer.render_song(&song, duration_per_section);
    
    let duration_seconds = audio_samples.len() as f32 / get_sample_rate() as f32;
    println!("   Duration: {:.1}s", duration_seconds);
    println!("   Samples: {}", audio_samples.len());
    println!();
    
    // Save files
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
