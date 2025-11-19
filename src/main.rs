mod composition;
mod synthesis;
mod audio;
mod config;
mod utils;

use composition::SongGenerator;
use synthesis::{SAMPLE_RATE, init_sample_rate};
use config::Config;
use utils::get_current_date;

fn main() {
    // Load configuration
    let config = Config::load_default().unwrap_or_else(|e| {
        eprintln!("⚠️  Warning: Could not load config.toml: {}", e);
        eprintln!("   Using default configuration\n");
        Config::default()
    });
    
    // Initialize sample rate from config (must be done before any synthesis)
    init_sample_rate(config.audio.sample_rate);
    
    println!("🎵 Rust Beats - Procedural Music Generator");
    println!("============================================");
    println!("Artist: {}", config.metadata.artist);
    println!("Sample Rate: {} Hz", config.audio.sample_rate);
    println!("Structure: {}\n", config.composition.structure);

    // Create song generator
    let generator = SongGenerator::new(config);
    let params = generator.params();
    
    println!("📝 Song Name: {}", params.song_name);
    println!("🎸 Genres: {:?}", params.genre_tags);
    println!("🎹 Key: Root MIDI {}, Scale: {:?}", params.key.root, params.key.scale_type);
    println!("⏱️  Tempo: {:.1} BPM", params.tempo.bpm);
    println!("🎵 Genre: {:?}", params.genre);
    println!("🥁 Groove: {:?}", params.groove_style);
    println!("🎸 Lead: {}, Bass: {}, Drums: {:?}", params.lead_instrument, params.bass_type, params.drum_kit);
    println!("🥁 Percussion: {}, Pads: {}, Mix: {}\n", params.percussion_type, params.pad_intensity, params.mixing_style);
    
    println!("🎼 Generating {} bars of music...", params.arrangement.total_bars);
    println!("   Structure: {} sections", params.arrangement.sections.len());
    for (section, bars) in &params.arrangement.sections {
        println!("   {:?}: {} bars", section, bars);
    }
    println!();

    // Generate all audio tracks in parallel
    println!("  ├─ Generating audio tracks (parallel)");
    let (drums, bass, melody_l, melody_r, pads_l, pads_r, fx, percussion) = generator.generate_audio_tracks();
    
    // Mix and master
    println!("  ├─ Mixing & mastering");
    let final_mix = generator.mix_and_master(drums, bass, melody_l, melody_r, pads_l, pads_r, fx, percussion);
    
    println!("  └─ Finalizing\n");
    
    // Get current date
    let date = get_current_date();
    
    // Save song files in parallel
    if let Err(e) = generator.save_song(&final_mix, &date) {
        eprintln!("❌ Error saving song: {}", e);
        return;
    }
    
    println!("   Duration: {:.1}s", final_mix.len() as f32 / SAMPLE_RATE() as f32);
    println!("   Samples: {}", final_mix.len());
    println!("\n🎉 Song generation complete!");
    println!("   Name: {}", params.song_name);
    println!("   Style: {} @ {:.0} BPM", params.genre_tags.join(", "), params.tempo.bpm);
}
