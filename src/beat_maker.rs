use rand::Rng;

#[derive(Debug)]
pub enum DrumSound {
    Kick,
    Snare,
    HiHat,
}

pub fn create_random_beat() -> Vec<DrumSound> {
    let mut beat = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..8 {
        let random_sound = rng.gen_range(0..3);

        let sound_to_add = match random_sound {
            0 => DrumSound::Kick,
            1 => DrumSound::Snare,
            _ => DrumSound::HiHat,
        };

        beat.push(sound_to_add);
    }

    beat
}