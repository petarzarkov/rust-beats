use crate::composition::beat_maker::DrumKit;
use crate::composition::music_theory::{ChordType, ScaleType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Genre {
    SwampMetal,
}

#[derive(Debug, Clone)]
pub struct GenreConfig {
    pub tempo_min: f32,
    pub tempo_max: f32,
    pub preferred_scales: Vec<ScaleType>,
    pub preferred_chord_types: Vec<ChordType>,
    pub drum_kit_preference: Vec<DrumKit>,
    pub bass_style: BassStyle,
    pub melody_density: MelodyDensity,
    pub arrangement_style: ArrangementStyle,
    pub energy_profile: EnergyProfile,
}

#[derive(Debug, Clone, Copy)]
pub enum BassStyle { Metal }
#[derive(Debug, Clone, Copy)]
pub enum MelodyDensity { Sparse, Heavy }
#[derive(Debug, Clone, Copy)]
pub enum ArrangementStyle { VerseChorus, Linear }
#[derive(Debug, Clone, Copy)]
pub enum EnergyProfile { Aggressive, Heavy }

impl Genre {
    pub fn config(&self) -> GenreConfig {
        match self {
            Genre::SwampMetal => GenreConfig {
                // CRITICAL FIX: Lower tempo to prevent "DnB" feel
                tempo_min: 65.0, 
                tempo_max: 95.0, 
                preferred_scales: vec![
                    ScaleType::Phrygian,
                    ScaleType::Locrian, // Very dark
                    ScaleType::Blues,
                ],
                preferred_chord_types: vec![
                    ChordType::Power5,
                ],
                drum_kit_preference: vec![DrumKit::Metal],
                bass_style: BassStyle::Metal,
                melody_density: MelodyDensity::Heavy,
                arrangement_style: ArrangementStyle::VerseChorus,
                energy_profile: EnergyProfile::Heavy,
            },
        }
    }
}

pub fn select_random_genre() -> Genre { Genre::SwampMetal }
pub fn get_genre_config(genre: Genre) -> GenreConfig { genre.config() }