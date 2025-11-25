use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WisdomData {
    pub wisdom: Vec<String>,
}

impl WisdomData {
    /// Load wisdom strings from JSON file
    /// Supports both formats:
    /// - Array: ["quote1", "quote2", ...]
    /// - Object: {"wisdom": ["quote1", "quote2", ...]}
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        // Try parsing as array first (simpler format)
        let wisdom: Vec<String> = serde_json::from_str(&contents)?;
        Ok(WisdomData { wisdom })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceType {
    Male,
    Female,
}

#[derive(Debug, Clone)]
pub struct VoiceSegment {
    pub text: String,
    pub voice_type: VoiceType,
    pub start_sample: usize,
    pub samples: Vec<f32>,
}

/// Generate TTS audio from text using gTTS (Google Text-to-Speech) Python package
/// Returns (samples, sample_rate) tuple
pub fn generate_tts(
    text: &str,
    model_path: &str,
    _espeak_data: &str, // Not used by gTTS (kept for API compatibility)
) -> Result<(Vec<i16>, u32), Box<dyn std::error::Error>> {
    // Create temporary output file
    let temp_wav = format!("/tmp/gtts_tts_{}.wav", std::process::id());

    // Call Python script: python3 scripts/generate_tts.py <text> <model_path> <output_wav>
    // model_path is used to determine voice gender (male/female)
    // gTTS uses different TLDs (.com for male, .co.uk for female)
    let output = Command::new("python3")
        .arg("scripts/generate_tts.py")
        .arg(text)
        .arg(model_path)
        .arg(&temp_wav)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python TTS generation failed: {}", stderr).into());
    }

    // Read the WAV file
    let reader = hound::WavReader::open(&temp_wav)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    // Read all samples
    let samples_i16: Vec<i16> = reader
        .into_samples::<i16>()
        .collect::<Result<Vec<_>, _>>()?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_wav);

    // Verify it's mono (pyttsx3 outputs mono)
    if spec.channels != 1 {
        return Err(format!(
            "Expected mono audio, got {} channels",
            spec.channels
        )
        .into());
    }

    Ok((samples_i16, sample_rate))
}

/// Resample audio from 22050 Hz to 44100 Hz (2x upsampling)
pub fn resample_22050_to_44100(input: Vec<i16>) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // Convert i16 to f32 for resampling (-1.0 to 1.0 range)
    let input_f32: Vec<f32> = input.iter().map(|&s| s as f32 / 32768.0).collect();

    // Create resampler parameters for high-quality 2x upsampling
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    // Create resampler (ratio 2.0 for 22050 -> 44100)
    let mut resampler = SincFixedIn::<f32>::new(
        2.0,              // ratio: 44100 / 22050 = 2.0
        2.0,              // max_ratio
        params,
        input_f32.len(),  // chunk_size
        1,                // channels (mono)
    )?;

    // Resample (wrapping in vector for mono channel)
    let output_waves = resampler.process(&[input_f32], None)?;

    // Convert back to i16
    let output: Vec<i16> = output_waves[0]
        .iter()
        .map(|&s| (s * 32768.0).clamp(-32768.0, 32767.0) as i16)
        .collect();

    Ok(output)
}

/// Convert i16 PCM samples to f32 format (-1.0 to 1.0)
pub fn convert_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples.iter().map(|&s| s as f32 / 32768.0).collect()
}

/// Resample audio to target sample rate using high-quality resampling
pub fn resample_to_target(input: Vec<i16>, from_rate: u32, to_rate: u32) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    if input.is_empty() || from_rate == to_rate {
        return Ok(input);
    }

    // Convert i16 to f32 for resampling (-1.0 to 1.0 range)
    let input_f32: Vec<f32> = input.iter().map(|&s| s as f32 / 32768.0).collect();

    // Calculate resampling ratio
    let ratio = to_rate as f64 / from_rate as f64;

    // Create resampler parameters for high-quality resampling
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    // Create resampler
    let mut resampler = SincFixedIn::<f32>::new(
        ratio,              // ratio
        ratio.max(2.0),     // max_ratio
        params,
        input_f32.len(),    // chunk_size
        1,                  // channels (mono)
    )?;

    // Resample (wrapping in vector for mono channel)
    let output_waves = resampler.process(&[input_f32], None)?;

    // Convert back to i16
    let output: Vec<i16> = output_waves[0]
        .iter()
        .map(|&s| (s * 32768.0).clamp(-32768.0, 32767.0) as i16)
        .collect();

    Ok(output)
}

/// Generate voice segment with complete processing pipeline
pub fn generate_voice_segment(
    text: &str,
    voice_type: VoiceType,
    male_model: &str,
    female_model: &str,
    espeak_data: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // Select appropriate model
    let model_path = match voice_type {
        VoiceType::Male => male_model,
        VoiceType::Female => female_model,
    };

    // Generate TTS (returns samples and sample rate)
    let (tts_samples, tts_sample_rate) = generate_tts(text, model_path, espeak_data)?;

    // Resample to 44100 Hz if needed
    let tts_44k = if tts_sample_rate != 44100 {
        resample_to_target(tts_samples, tts_sample_rate, 44100)?
    } else {
        tts_samples
    };

    // Convert to f32 for mixing
    let tts_f32 = convert_i16_to_f32(&tts_44k);

    Ok(tts_f32)
}

/// Calculate voice envelope for ducking (smooth amplitude tracking)
pub fn calculate_voice_envelope(voice_samples: &[f32], window_size: usize) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(voice_samples.len());

    for i in 0..voice_samples.len() {
        let start = i.saturating_sub(window_size / 2);
        let end = (i + window_size / 2).min(voice_samples.len());

        // Calculate RMS in window
        let window_sum: f32 = voice_samples[start..end]
            .iter()
            .map(|&s| s * s)
            .sum();
        let rms = (window_sum / (end - start) as f32).sqrt();

        envelope.push(rms);
    }

    envelope
}

/// Mix voice with music, applying ducking to reduce music volume during speech
pub fn mix_with_ducking(
    music: &mut [f32],
    voice_segment: &VoiceSegment,
    voice_volume: f32,
    duck_db: f32,
    sample_rate: u32,
) {
    // Convert dB reduction to amplitude factor
    let duck_factor = 10f32.powf(duck_db / 20.0);

    // Calculate voice envelope for smooth ducking
    let envelope_window = (sample_rate / 100) as usize; // 10ms window
    let voice_envelope = calculate_voice_envelope(&voice_segment.samples, envelope_window);

    // Apply fade in/out to voice (50ms)
    let fade_samples = (sample_rate as f32 * 0.05) as usize;

    // Mix voice into music with ducking
    for (i, &voice_sample) in voice_segment.samples.iter().enumerate() {
        let music_idx = voice_segment.start_sample + i;

        if music_idx >= music.len() {
            break;
        }

        // Apply fade in/out to voice
        let fade_factor = if i < fade_samples {
            i as f32 / fade_samples as f32
        } else if i >= voice_segment.samples.len() - fade_samples {
            (voice_segment.samples.len() - i) as f32 / fade_samples as f32
        } else {
            1.0
        };

        // Calculate ducking amount based on voice envelope
        let voice_env = voice_envelope[i];
        let duck_amount = 1.0 - (voice_env * (1.0 - duck_factor));

        // Apply ducking to music and add voice
        music[music_idx] *= duck_amount;
        music[music_idx] += voice_sample * voice_volume * fade_factor;
    }
}

/// Select wisdom quotes for a song with chorus structure
/// Returns: (intro_quote, chorus_quotes, outro_quote)
/// - intro_quote: Single quote for beginning
/// - chorus_quotes: 3 quotes that will be repeated as chorus
/// - outro_quote: Single quote for ending
pub fn select_wisdom_with_chorus(
    wisdom_data: &WisdomData,
    seed: u64,
) -> (String, Vec<String>, String) {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    if wisdom_data.wisdom.is_empty() {
        return (String::new(), Vec::new(), String::new());
    }

    // Select 5 unique quotes: 1 intro + 3 chorus + 1 outro
    let mut selected_indices = std::collections::HashSet::new();
    while selected_indices.len() < 5.min(wisdom_data.wisdom.len()) {
        let idx = rng.gen_range(0..wisdom_data.wisdom.len());
        selected_indices.insert(idx);
    }

    let mut quotes: Vec<String> = selected_indices
        .iter()
        .map(|&idx| wisdom_data.wisdom[idx].clone())
        .collect();

    // Ensure we have enough quotes
    if quotes.len() < 5 {
        while quotes.len() < 5 {
            quotes.push(wisdom_data.wisdom[0].clone());
        }
    }

    let intro = quotes[0].clone();
    let chorus = vec![quotes[1].clone(), quotes[2].clone(), quotes[3].clone()];
    let outro = quotes[4].clone();

    (intro, chorus, outro)
}

/// Legacy function for backwards compatibility
pub fn select_wisdom(
    wisdom_data: &WisdomData,
    max_segments: usize,
    seed: u64,
) -> Vec<(String, VoiceType)> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let mut selected = Vec::new();

    for _ in 0..max_segments {
        if wisdom_data.wisdom.is_empty() {
            break;
        }

        // Select random wisdom
        let idx = rng.gen_range(0..wisdom_data.wisdom.len());
        let text = wisdom_data.wisdom[idx].clone();

        // All voices are female now
        selected.push((text, VoiceType::Female));
    }

    selected
}

/// Determine voice placement timestamps based on song arrangement
pub fn calculate_voice_timings(
    placement: &str,
    total_duration_samples: usize,
    num_segments: usize,
    sample_rate: u32,
) -> Vec<usize> {
    let mut timings = Vec::new();

    match placement {
        "intro" => {
            // Place at the start (after 1 second)
            timings.push(sample_rate as usize);
        }
        "intro_outro" => {
            // Place at start and end
            timings.push(sample_rate as usize);
            if num_segments > 1 {
                let outro_time = total_duration_samples.saturating_sub(sample_rate as usize * 10);
                timings.push(outro_time);
            }
        }
        "bridge" => {
            // Place in the middle
            let mid_time = total_duration_samples / 2;
            timings.push(mid_time);
        }
        "intro_bridge" => {
            // Place at intro and bridge
            timings.push(sample_rate as usize);
            if num_segments > 1 {
                let bridge_time = total_duration_samples / 2;
                timings.push(bridge_time);
            }
        }
        "distributed" => {
            // Distribute evenly throughout the song with gaps
            if num_segments == 0 {
                return timings;
            }

            // Start 2 seconds in, end 10 seconds before the end
            let start_offset = sample_rate as usize * 2;
            let end_offset = sample_rate as usize * 10;
            let usable_duration = total_duration_samples.saturating_sub(start_offset + end_offset);

            if num_segments == 1 {
                // Single segment: place in the middle
                timings.push(start_offset + usable_duration / 2);
            } else {
                // Multiple segments: distribute evenly
                let gap = usable_duration / (num_segments + 1);
                for i in 1..=num_segments {
                    timings.push(start_offset + gap * i);
                }
            }
        }
        _ => {
            // Default: intro
            timings.push(sample_rate as usize);
        }
    }

    // Limit to requested number of segments
    timings.truncate(num_segments);

    timings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_i16_to_f32() {
        let samples = vec![0i16, 16384, -16384, 32767, -32768];
        let converted = convert_i16_to_f32(&samples);

        assert!((converted[0] - 0.0).abs() < 0.001);
        assert!((converted[1] - 0.5).abs() < 0.001);
        assert!((converted[2] + 0.5).abs() < 0.001);
    }

    #[test]
    fn test_voice_envelope() {
        let samples = vec![0.0, 0.5, 1.0, 0.5, 0.0];
        let envelope = calculate_voice_envelope(&samples, 3);

        assert_eq!(envelope.len(), samples.len());
        // Envelope should be smooth and follow amplitude
        assert!(envelope[2] > envelope[0]);
    }

    #[test]
    fn test_calculate_voice_timings() {
        let timings = calculate_voice_timings("intro_bridge", 88200 * 180, 2, 44100);
        assert_eq!(timings.len(), 2);
        assert_eq!(timings[0], 44100); // 1 second
        assert!(timings[1] > timings[0]);
    }
}
