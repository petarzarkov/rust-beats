use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub audio: AudioConfig,
    pub metadata: MetadataConfig,
    pub composition: CompositionConfig,
    pub generation: GenerationConfig,
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
    #[serde(default = "default_instrument_probabilities")]
    pub instrument_probabilities: InstrumentProbabilities,
    #[serde(default = "default_percussion_config")]
    pub percussion: PercussionConfig,
    #[serde(default = "default_pad_config")]
    pub pads: PadConfig,
    #[serde(default = "default_mixing_config")]
    pub mixing: MixingConfig,
    #[serde(default = "default_bass_drop_config")]
    pub bass_drops: BassDropConfig,
    #[serde(default = "default_melody_config")]
    pub melody: MelodyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentProbabilities {
    pub rhodes: f32,
    pub ukulele: f32,
    pub guitar: f32,
    pub electric: f32,
    pub organ: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercussionConfig {
    pub chance: f32,
    pub tambourine: f32,
    pub cowbell: f32,
    pub bongo: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadConfig {
    pub subtle: f32,
    pub medium: f32,
    pub prominent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixingConfig {
    pub clean: f32,
    pub warm: f32,
    pub punchy: f32,
    pub spacious: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BassDropConfig {
    pub default_chance_8th_bar: f32,
    pub default_chance_12th_bar: f32,
    pub rock_chance_8th_bar: f32,
    pub rock_chance_12th_bar: f32,
    pub dnb_chance_8th_bar: f32,
    pub dnb_chance_16th_bar: f32,
    pub amplitude: f32,
    pub rock_amplitude: f32,
    pub dnb_amplitude: f32,
    pub default_duration_beats: f32,
    pub rock_duration_beats: f32,
    pub dnb_duration_beats: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MelodyConfig {
    pub occurrence_chance: f32,
    pub rhodes_usage_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub output_dir: String,
    pub write_metadata_json: bool,
}

fn default_instrument_probabilities() -> InstrumentProbabilities {
    InstrumentProbabilities {
        rhodes: 0.40,
        ukulele: 0.15,
        guitar: 0.15,
        electric: 0.15,
        organ: 0.15,
    }
}

fn default_percussion_config() -> PercussionConfig {
    PercussionConfig {
        chance: 0.30,
        tambourine: 0.33,
        cowbell: 0.33,
        bongo: 0.34,
    }
}

fn default_pad_config() -> PadConfig {
    PadConfig {
        subtle: 0.40,
        medium: 0.40,
        prominent: 0.20,
    }
}

fn default_mixing_config() -> MixingConfig {
    MixingConfig {
        clean: 0.25,
        warm: 0.25,
        punchy: 0.25,
        spacious: 0.25,
    }
}

fn default_bass_drop_config() -> BassDropConfig {
    BassDropConfig {
        default_chance_8th_bar: 0.6,
        default_chance_12th_bar: 0.4,
        rock_chance_8th_bar: 0.5,
        rock_chance_12th_bar: 0.3,
        dnb_chance_8th_bar: 0.5,
        dnb_chance_16th_bar: 0.4,
        amplitude: 0.4,
        rock_amplitude: 0.35,
        dnb_amplitude: 0.4,
        default_duration_beats: 1.5,
        rock_duration_beats: 1.5,
        dnb_duration_beats: 2.0,
    }
}

fn default_melody_config() -> MelodyConfig {
    MelodyConfig {
        occurrence_chance: 0.15,
        rhodes_usage_percent: 60,
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
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
                software: "Rust Beats - Procedural Music Generator".to_string(),
            },
            composition: CompositionConfig {
                structure: "standard".to_string(),
                min_tempo: 90.0,
                max_tempo: 130.0,
                instrument_probabilities: default_instrument_probabilities(),
                percussion: default_percussion_config(),
                pads: default_pad_config(),
                mixing: default_mixing_config(),
                bass_drops: default_bass_drop_config(),
                melody: default_melody_config(),
            },
            generation: GenerationConfig {
                output_dir: "output".to_string(),
                write_metadata_json: true,
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

