use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub audio: AudioConfig,
    pub metadata: MetadataConfig,
    pub composition: CompositionConfig,
    pub generation: GenerationConfig,
    #[serde(default = "default_voice_config")]
    pub voice: VoiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub bit_depth: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub artist: String,
    pub copyright: String,
    pub software: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionConfig {
    pub structure: String, // "short" or "standard"
    pub min_tempo: f32,
    pub max_tempo: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub output_dir: String,
    pub write_metadata_json: bool,
    #[serde(default = "default_encode_mp3")]
    pub encode_mp3: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub wisdom_file: String,
    #[serde(skip)]
    pub language: String, // Detected from wisdom_file name
    pub placement: String, // "intro" | "intro_outro" | "bridge" | "intro_bridge" | "distributed"
    pub volume: f32,
    pub duck_music_db: f32,
    pub segments_per_minute: f32,
}

fn default_encode_mp3() -> bool {
    true
}

fn default_voice_config() -> VoiceConfig {
    VoiceConfig {
        enabled: false,
        wisdom_file: "wisdom.json".to_string(),
        language: "en".to_string(), // Default to English
        placement: "distributed".to_string(),
        volume: 0.7,
        duck_music_db: -6.0,
        segments_per_minute: 3.0,
    }
}

impl VoiceConfig {
    /// Detect language from wisdom file name
    /// - "wisdom.json" -> "en"
    /// - "wisdom-bg.json" -> "bg"
    /// - Default: "en"
    pub fn detect_language_from_filename(filename: &str) -> String {
        if filename.contains("-bg") {
            "bg".to_string()
        } else {
            "en".to_string() // Default to English
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&contents)?;
        // Detect language from wisdom file name
        config.voice.language = VoiceConfig::detect_language_from_filename(&config.voice.wisdom_file);
        Ok(config)
    }

    /// Load configuration from default location (config.toml in project root)
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load("config.toml")
    }

    /// Create a default configuration
    pub fn default() -> Self {
        Config {
            audio: AudioConfig {
                sample_rate: 44100,
                bit_depth: 16,
            },
            metadata: MetadataConfig {
                artist: "Petar Zarkov".to_string(),
                copyright: "Free to use - CC0 Public Domain".to_string(),
                software: "Rust Beats - Procedural Metal Generator".to_string(),
            },
            composition: CompositionConfig {
                structure: "standard".to_string(),
                min_tempo: 80.0,
                max_tempo: 250.0,
            },
            generation: GenerationConfig {
                output_dir: "output".to_string(),
                write_metadata_json: true,
                encode_mp3: true,
            },
            voice: {
                let mut voice = default_voice_config();
                voice.language = VoiceConfig::detect_language_from_filename(&voice.wisdom_file);
                voice
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.audio.sample_rate, 44100);
        assert_eq!(config.metadata.artist, "Petar Zarkov");
    }
}
