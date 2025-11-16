mod audio_renderer;
mod beat_maker;

use std::fs;
use std::thread;

use audio_renderer::render_beat_to_wav;
use beat_maker::create_beat;

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

                let my_beat = create_beat(64).expect("Could not create beat");

                let file_path = format!("{}/beat_{}.wav", OUTPUT_DIR, i + 1);

                match render_beat_to_wav(&my_beat, &file_path) {
                    Ok(_) => println!("Thread {}: Successfully wrote {}", i, file_path),
                    Err(e) => eprintln!("Thread {}: Error writing WAV: {}", i, e),
                }
            });
        }
    });

    println!("\nAll beats generated! Check the '{}' folder.", OUTPUT_DIR);
}