use rand::Rng;

#[derive(Debug, Clone)]
pub enum DrumSound {
    Kick,
    Snare,
    HiHat,
    Rest,
}

pub fn create_beat(num_steps: u32) -> Result<Vec<DrumSound>, &'static str> {
    if num_steps < 8 || num_steps > 64 {
        return Err("Number of steps must be between 8 and 64");
    }

    let mut beat = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..num_steps {
        let position_in_measure = i % 16;
        
        let roll = rng.gen_range(0..100);

        let sound_to_add = match position_in_measure {
            
            0 => {
                if roll < 85 { DrumSound::Kick } else { DrumSound::HiHat }
            }

            4 => {
                if roll < 80 { DrumSound::Snare } else { DrumSound::HiHat }
            }

            8 => {
                if roll < 70 { DrumSound::Kick } else { DrumSound::HiHat }
            }

            12 => {
                if roll < 80 { DrumSound::Snare } else { DrumSound::HiHat }
            }

            2 | 6 | 10 | 14 => {
                 if roll < 90 { DrumSound::HiHat } else { DrumSound::Rest }
            }

            _ => {
                if roll < 60 { DrumSound::HiHat }
                else if roll < 90 { DrumSound::Rest }
                else { DrumSound::Snare }
            }
        };

        beat.push(sound_to_add);
    }

    Ok(beat)
}