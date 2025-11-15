mod beat_maker;
mod audio_renderer;

use beat_maker::create_random_beat;
use audio_renderer::render_beat_to_wav;

fn main() {
    let top_notch_beat = create_random_beat(64).unwrap();

    match render_beat_to_wav(&top_notch_beat) {
        Ok(_) => println!("\nSuccessfully wrote beat to output.wav!"),
        Err(e) => println!("\nError writing WAV file: {}", e),
    }
}