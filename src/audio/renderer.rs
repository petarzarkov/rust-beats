use crate::synthesis::get_sample_rate;
use hound;
use std::io::Write;

/// Song metadata for WAV file
#[derive(Debug, Clone)]
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub copyright: String,
    pub genre: Vec<String>,
    pub date: String, // Creation date in YYYY-MM-DD format
}

impl SongMetadata {
    pub fn new(
        title: String,
        artist: String,
        copyright: String,
        genre: Vec<String>,
        date: String,
    ) -> Self {
        SongMetadata {
            title,
            artist,
            copyright,
            genre,
            date,
        }
    }
}

/// Render audio samples to WAV file with metadata
pub fn render_to_wav_with_metadata(
    samples: &[f32],
    filename: &str,
    metadata: &SongMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    // First write the basic WAV file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: get_sample_rate(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(filename, spec)?;

    for &sample in samples {
        // Convert float32 (-1.0 to 1.0) to i16
        let amplitude = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(amplitude)?;
    }

    writer.finalize()?;

    // Now append metadata as LIST INFO chunk
    // Note: hound doesn't support writing custom chunks, so we'll append them manually
    append_info_chunk(filename, metadata)?;

    Ok(())
}

/// Append RIFF LIST INFO chunk to existing WAV file
fn append_info_chunk(
    filename: &str,
    metadata: &SongMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom};

    let mut file = OpenOptions::new().read(true).write(true).open(filename)?;

    // Read the file size from RIFF header
    file.seek(SeekFrom::Start(4))?;
    let mut size_bytes = [0u8; 4];
    std::io::Read::read_exact(&mut file, &mut size_bytes)?;
    let file_size = u32::from_le_bytes(size_bytes);

    // Seek to end of file
    file.seek(SeekFrom::End(0))?;

    // Build INFO chunk
    let mut info_data = Vec::new();

    // Add INAM (title)
    add_info_field(&mut info_data, b"INAM", metadata.title.as_bytes());

    // Add IART (artist)
    add_info_field(&mut info_data, b"IART", metadata.artist.as_bytes());

    // Add ICOP (copyright)
    add_info_field(&mut info_data, b"ICOP", metadata.copyright.as_bytes());

    // Add IGNR (genre)
    if !metadata.genre.is_empty() {
        let genre_str = metadata.genre.join(", ");
        add_info_field(&mut info_data, b"IGNR", genre_str.as_bytes());
    }

    // Add ICRD (creation date)
    add_info_field(&mut info_data, b"ICRD", metadata.date.as_bytes());

    // Add ISFT (software)
    add_info_field(
        &mut info_data,
        b"ISFT",
        b"Rust Beats - Procedural Music Generator",
    );

    // Build LIST chunk
    let list_size = 4 + info_data.len(); // 4 for "INFO" + data

    // Write LIST chunk
    file.write_all(b"LIST")?;
    file.write_all(&(list_size as u32).to_le_bytes())?;
    file.write_all(b"INFO")?;
    file.write_all(&info_data)?;

    // Update RIFF chunk size
    let new_size = file_size + 8 + list_size as u32; // 8 for LIST header
    file.seek(SeekFrom::Start(4))?;
    file.write_all(&new_size.to_le_bytes())?;

    Ok(())
}

/// Add a field to INFO chunk data
fn add_info_field(data: &mut Vec<u8>, fourcc: &[u8; 4], value: &[u8]) {
    // FourCC
    data.extend_from_slice(fourcc);

    // Size (must be even, add null terminator)
    let mut size = value.len() as u32 + 1; // +1 for null terminator
    if size % 2 != 0 {
        size += 1; // Pad to even
    }
    data.extend_from_slice(&size.to_le_bytes());

    // Value
    data.extend_from_slice(value);
    data.push(0); // Null terminator

    // Pad if needed
    if (value.len() + 1) % 2 != 0 {
        data.push(0);
    }
}

/// Convert f32 samples to i16 for WAV writing

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = SongMetadata::new(
            "Test Song".to_string(),
            "Petar Zarkov".to_string(),
            "Free to use - CC0 Public Domain".to_string(),
            vec!["Funk".to_string(), "Jazz".to_string()],
            "2025-01-01".to_string(),
        );
        assert_eq!(metadata.artist, "Petar Zarkov");
        assert!(metadata.copyright.contains("CC0"));
    }
}
