use rand::Rng;

#[derive(Debug)]
pub enum DrumSound {
    Kick,
    Snare,
    HiHat,
}

const MIN_NUM_STEPS: u32 = 8;
const MAX_NUM_STEPS: u32 = 64;

pub fn create_random_beat(num_steps: u32) -> Result<Vec<DrumSound>, &'static str> {
    if num_steps < MIN_NUM_STEPS {
        return Err("Number of steps must be greater than or equal to 8")
    }

    if num_steps > MAX_NUM_STEPS {
        return Err("Number of steps must be less than or equal to 64")
    }

    let mut beat = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..num_steps {
        let random_sound = rng.gen_range(0..3);

        let sound_to_add = match random_sound {
            0 => DrumSound::Kick,
            1 => DrumSound::Snare,
            _ => DrumSound::HiHat,
        };

        beat.push(sound_to_add);
    }

    Ok(beat)
}