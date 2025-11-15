use hound;
use super::beat_maker::DrumSound;

type AudioSample = i16;
const SAMPLE_RATE: u32 = 44100;

const MILLISECONDS_PER_STEP: f32 = 250.0;

// At 44.1kHz, that is 44100 * 0.25 = 11,025 samples.
const SAMPLES_PER_STEP: u32 = (SAMPLE_RATE as f32 * (MILLISECONDS_PER_STEP / 1000.0)) as u32;

pub fn render_beat_to_wav(beat: &[DrumSound]) -> Result<(), hound::Error> {
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
    
    // This assumes all files are 44.1kHz, 16-bit.
    let spec = kick_reader.spec();
    let mut writer = hound::WavWriter::create("output.wav", spec)?;

    for sound in beat {
        let sound_data: &Vec<AudioSample> = match sound {
            DrumSound::Kick => &kick_samples,
            DrumSound::Snare => &snare_samples,
            DrumSound::HiHat => &hihat_samples,
        };

        for &sample in sound_data {
            writer.write_sample(sample)?;
        }

        // We must pad the rest of the "step" with silence
        // so that each beat step has the same length.
        let samples_written = sound_data.len() as u32;
        if samples_written < SAMPLES_PER_STEP {
            let silence_to_add = SAMPLES_PER_STEP - samples_written;

            // `0` is silence in PCM audio.
            for _ in 0..silence_to_add {
                writer.write_sample(0 as AudioSample)?;
            }
        }
    }

    writer.finalize()?;
    
    Ok(())
}