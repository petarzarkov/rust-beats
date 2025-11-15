mod beat_maker;

use beat_maker::{create_random_beat, DrumSound};

fn main() {
    let my_beat = create_random_beat();

    println!("The generated beat is: {:?}", my_beat);
    println!("---");

    println!("--- Playing Beat ---");
    for sound in &my_beat {
        match sound {
            DrumSound::Kick => println!("Boom!"),
            DrumSound::Snare => println!("Clap!"),
            DrumSound::HiHat => println!("Tss..."),
        }
    }
    println!("--- Beat Finished ---");
}