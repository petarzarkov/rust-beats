/// MP3 encoding for file size optimization
use std::fs::File;
use std::io::Write;
use crate::synthesis::SAMPLE_RATE;

/// Encode float32 samples to MP3 file
/// Reduces file size by ~85% (10-20MB WAV → 1-3MB MP3)
pub fn encode_to_mp3(
    samples: &[f32],
    filename: &str,
    _title: &str,
    _artist: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert float samples to i16 for MP3 encoder
    let pcm_samples: Vec<i16> = samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();
    
    // Create encoder with parameters using builder
    let mut encoder = mp3lame_encoder::Builder::new()
        .expect("Failed to create encoder builder");
    encoder.set_num_channels(1).expect("Failed to set channels");
    encoder.set_sample_rate(SAMPLE_RATE()).expect("Failed to set sample rate");
    encoder.set_brate(mp3lame_encoder::Birtate::Kbps192).expect("Failed to set bitrate");
    encoder.set_quality(mp3lame_encoder::Quality::Best).expect("Failed to set quality");
    
    let mut encoder = encoder.build().expect("Failed to build encoder");
    
    // Encode audio - allocate buffer for MP3 data using MaybeUninit
    use std::mem::MaybeUninit;
    let mut mp3_buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); pcm_samples.len() * 5 / 4 + 7200];
    
    // For mono (1 channel), use MonoPcm instead of InterleavedPcm
    // InterleavedPcm divides length by 2 (expects stereo), MonoPcm uses full length
    let encoded_size = encoder.encode(
        mp3lame_encoder::MonoPcm(&pcm_samples),
        mp3_buffer.as_mut_slice()
    ).expect("Failed to encode");
    
    // Flush remaining data
    let flushed_size = encoder.flush::<mp3lame_encoder::FlushNoGap>(
        &mut mp3_buffer[encoded_size..]
    ).expect("Failed to flush");
    
    // Convert MaybeUninit to initialized bytes  
    let total_size = encoded_size + flushed_size;
    let mp3_bytes: Vec<u8> = mp3_buffer[..total_size]
        .iter()
        .map(|b| unsafe { b.assume_init() })
        .collect();
    
    // Write to file
    let mut file = File::create(filename)?;
    file.write_all(&mp3_bytes)?;
    
    Ok(())
}

