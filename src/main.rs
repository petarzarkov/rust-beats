mod composition;
mod synthesis;
mod audio;
mod config;

use std::fs;
use composition::{
    generate_song_name, generate_genre_tags,
    Key, Tempo, generate_chord_progression,
    generate_drum_pattern, random_groove_style, DrumHit, GrooveStyle,
    Arrangement,
};
use synthesis::{
    generate_kick, generate_snare, generate_hihat, generate_clap,
    generate_bassline, generate_melody, generate_pads,
    SAMPLE_RATE, LofiProcessor,
};
use audio::{render_to_wav_with_metadata, SongMetadata, Track, mix_tracks, master_lofi, stereo_to_mono};
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
    println!("🥁 Groove: {:?}\n", groove_style);

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

    // Professional multi-track mixing with stereo, panning, EQ
    println!("  ├─ Multi-track mixing");
    let tracks = vec![
        // Drums: LOUD and punchy - the foundation!
        Track::new(drums)
            .with_volume(1.2)  // Much louder!
            .with_pan(0.0)
            .with_eq(1.4, 1.0, 0.9),  // Punchy, balanced
        
        // Bass: VERY subtle, background only
        Track::new(bassline)
            .with_volume(0.45)  // Much quieter - reduced from 0.65
            .with_pan(0.0)
            .with_eq(1.1, 0.8, 0.5),  // Minimal presence
        
        // Melody: Brighter, happier (but still tasteful)
        Track::new(melody.clone())
            .with_volume(0.38)  // Slightly louder for happiness
            .with_pan(-0.15)
            .with_eq(0.9, 1.1, 1.0),  // Brighter, more present
        
        // Melody doubled for stereo width
        Track::new(melody)
            .with_volume(0.38)  // Slightly louder
            .with_pan(0.15)
            .with_eq(0.9, 1.1, 1.0),  // Brighter
        
        // Pads: subtle atmospheric layer
        Track::new(pads.clone())
            .with_volume(0.35)
            .with_pan(-0.5)
            .with_eq(0.8, 0.9, 0.8),  // Darker
        
        Track::new(pads)
            .with_volume(0.35)
            .with_pan(0.5)
            .with_eq(0.8, 0.9, 0.8),
    ];
    
    let mut stereo_mix = mix_tracks(tracks);
    
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

    // Create metadata
    let metadata = SongMetadata {
        title: song_name.clone(),
        artist: config.metadata.artist.clone(),
        copyright: config.metadata.copyright.clone(),
        genre: genre_tags.clone(),
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

    // Save metadata for the workflow
    if config.generation.write_metadata_json {
        let metadata_path = format!("{}/song_metadata.json", output_dir);
        let metadata_json = serde_json::json!({
            "name": song_name,
            "artist": config.metadata.artist,
            "genre": genre_tags,
            "tempo": tempo.bpm,
            "duration": final_mix.len() as f32 / SAMPLE_RATE as f32,
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
