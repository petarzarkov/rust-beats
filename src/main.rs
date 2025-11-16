mod audio_renderer;
mod beat_maker;

use std::fs;
use std::thread;
use hound;

use audio_renderer::render_beat_to_wav;
use beat_maker::{create_beat};

fn main() {
    const OUTPUT_DIR: &str = "output";
    fs::create_dir_all(OUTPUT_DIR).expect("Could not create output directory");

    let num_threads = std::thread::available_parallelism().unwrap().get();
    
    println!("System has {} logical cores.", num_threads);
    println!("Generating {} beats concurrently...", num_threads);

    thread::scope(|s| {
        for i in 0..num_threads {
            s.spawn(move || {
                println!("Thread {}: Generating beat...", i);
                
                let beat_steps = 40; 
                
                match create_beat(beat_steps) {
                    Ok(my_beat) => {
                        let file_path = format!("{}/beat_{}.wav", OUTPUT_DIR, i + 1);
                        
                        match render_beat_to_wav(&my_beat, &file_path) {
                            Ok(_) => println!("Thread {}: Successfully wrote {}", i, file_path),
                            Err(e) => eprintln!("Thread {}: Error writing WAV: {}", i, e),
                        }
                    },
                    Err(e) => {
                        eprintln!("Thread {}: Error creating beat: {}", i, e);
                    }
                }
            });
        }
    });

    println!("\nAll beats generated! Check the '{}' folder.", OUTPUT_DIR);

    println!("Now joining beats into final_song.wav...");

    let song_filename = format!("{}/final_song.wav", OUTPUT_DIR);

    let first_beat_path = format!("{}/beat_1.wav", OUTPUT_DIR);
    let spec = match hound::WavReader::open(&first_beat_path) {
        Ok(reader) => reader.spec(),
        Err(e) => {
            eprintln!("Error opening first beat file '{}': {}", first_beat_path, e);
            eprintln!("Cannot create final song. Aborting.");
            return;
        }
    };

    let mut writer = hound::WavWriter::create(&song_filename, spec)
        .expect("Failed to create final song writer");

    for i in 0..num_threads {
        let beat_path = format!("{}/beat_{}.wav", OUTPUT_DIR, i + 1);
        println!("... Appending {}", beat_path);

        match hound::WavReader::open(&beat_path) {
            Ok(mut reader) => {
                for sample in reader.samples::<i16>() {
                    let s = sample.expect("Failed to read sample");
                    writer.write_sample(s).expect("Failed to write sample");
                }
            }
            Err(e) => {
                eprintln!("Could not read {}: {}", beat_path, e);
            }
        }
    }

    writer.finalize().expect("Failed to finalize song");
    println!("\nSuccessfully created {}!", song_filename);
}