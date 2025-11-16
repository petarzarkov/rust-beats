use hound;
use super::beat_maker::DrumSound;

type AudioSample = i16;
const SAMPLE_RATE: u32 = 44100;

const MILLISECONDS_PER_STEP: f32 = 250.0; 

const SWING_OFFSET_MS: f32 = 80.0; 

const SAMPLES_PER_STEP_BASE: u32 = (SAMPLE_RATE as f32 * (MILLISECONDS_PER_STEP / 1000.0)) as u32;
const SWING_OFFSET_SAMPLES: u32 = (SAMPLE_RATE as f32 * (SWING_OFFSET_MS / 1000.0)) as u32;

const ON_BEAT_STEP_LENGTH: u32 = SAMPLES_PER_STEP_BASE + SWING_OFFSET_SAMPLES;
const OFF_BEAT_STEP_LENGTH: u32 = SAMPLES_PER_STEP_BASE - SWING_OFFSET_SAMPLES;


pub fn render_beat_to_wav(beat: &[DrumSound], filename: &str) -> Result<(), hound::Error> {
    let mut kick_reader = hound::WavReader::open("samples/kick.wav")?;
    let mut snare_reader = hound::WavReader::open("samples/snare.wav")?;
    let mut hihat_reader = hound::WavReader::open("samples/hihat.wav")?;

    let kick_samples: Vec<AudioSample> = kick_reader
        .samples::<AudioSample>()
        .collect::<Result<Vec<_>, _>>()?;
    
    let snare_samples: Vec<AudioSample> = snare_reader
        .samples::<AudioSample>()
        .collect::<Result<Vec<_>, _>>()?;

    let hihat_samples: Vec<AudioSample> = hihat_reader
        .samples::<AudioSample>()
        .collect::<Result<Vec<_>, _>>()?;
    
    let spec = kick_reader.spec();
    let mut writer = hound::WavWriter::create(filename, spec)?;

    for (i, sound) in beat.iter().enumerate() {
        
        let is_on_beat = (i % 2) == 0;
        
        let target_step_length = if is_on_beat {
            ON_BEAT_STEP_LENGTH
        } else {
            OFF_BEAT_STEP_LENGTH
        };
        
        let sound_data: &Vec<AudioSample> = match sound {
            DrumSound::Kick => &kick_samples,
            DrumSound::Snare => &snare_samples,
            DrumSound::HiHat => &hihat_samples,
            DrumSound::Rest => &Vec::new(),
        };

        for &sample in sound_data {
            writer.write_sample(sample)?;
        }

        let samples_written = sound_data.len() as u32;
        if samples_written < target_step_length {
            let silence_to_add = target_step_length - samples_written;

            for _ in 0..silence_to_add {
                writer.write_sample(0 as AudioSample)?;
            }
        }
    }

    writer.finalize()?;
    
    Ok(())
}