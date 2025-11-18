use rand::Rng;
use crate::composition::music_theory::{ScaleType, ChordType};
use crate::composition::beat_maker::DrumKit;

/// Musical genre determines the overall style and characteristics of a song
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Genre {
    Lofi,
    Rock,
    Dubstep,
    DnB,
    Jazz,
    Funk,
    HipHop,
    ElectroSwing,
}

/// Configuration for a genre defining its musical characteristics
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
}

/// Bass style preference for a genre
#[derive(Debug, Clone, Copy)]
pub enum BassStyle {
    Standard,
    Rock,
    Synth,
    Upright,
    Finger,
    Slap,
    Wobble,  // Dubstep
    Reese,   // DnB
}

/// Melody density preference
#[derive(Debug, Clone, Copy)]
pub enum MelodyDensity {
    Sparse,      // Rock - riff-based
    Moderate,    // Lofi, Jazz - tasteful accents
    Dense,       // DnB - complex patterns
    Glitchy,     // Dubstep - stutter effects
}

/// Arrangement style preference
#[derive(Debug, Clone, Copy)]
pub enum ArrangementStyle {
    VerseChorus,    // Rock, Pop
    BuildDrop,      // Dubstep, EDM
    Consistent,     // DnB - steady energy
    Groove,         // Funk, Jazz - extended sections
}

impl Genre {
    /// Get the configuration for this genre
    pub fn config(&self) -> GenreConfig {
        match self {
            Genre::Lofi => GenreConfig {
                tempo_min: 60.0,
                tempo_max: 100.0,
                preferred_scales: vec![ScaleType::Major, ScaleType::MajorPentatonic, ScaleType::Lydian, ScaleType::Dorian],
                preferred_chord_types: vec![ChordType::Major7, ChordType::Minor7, ChordType::Major9, ChordType::Minor9],
                drum_kit_preference: vec![DrumKit::Lofi, DrumKit::Acoustic, DrumKit::Jazz],
                bass_style: BassStyle::Standard,
                melody_density: MelodyDensity::Moderate,
                arrangement_style: ArrangementStyle::Groove,
            },
            Genre::Rock => GenreConfig {
                tempo_min: 100.0,
                tempo_max: 160.0,
                preferred_scales: vec![ScaleType::Minor, ScaleType::Major, ScaleType::MinorPentatonic, ScaleType::Blues],
                preferred_chord_types: vec![ChordType::Major, ChordType::Minor, ChordType::Sus4],
                drum_kit_preference: vec![DrumKit::Rock, DrumKit::Acoustic],
                bass_style: BassStyle::Rock,
                melody_density: MelodyDensity::Sparse,
                arrangement_style: ArrangementStyle::VerseChorus,
            },
            Genre::Dubstep => GenreConfig {
                tempo_min: 140.0,
                tempo_max: 150.0,
                preferred_scales: vec![ScaleType::Minor, ScaleType::Blues, ScaleType::Phrygian],
                preferred_chord_types: vec![ChordType::Minor, ChordType::Diminished, ChordType::Minor7],
                drum_kit_preference: vec![DrumKit::Electronic808, DrumKit::HipHop],
                bass_style: BassStyle::Wobble,
                melody_density: MelodyDensity::Glitchy,
                arrangement_style: ArrangementStyle::BuildDrop,
            },
            Genre::DnB => GenreConfig {
                tempo_min: 160.0,
                tempo_max: 180.0,
                preferred_scales: vec![ScaleType::Minor, ScaleType::Dorian, ScaleType::Mixolydian],
                preferred_chord_types: vec![ChordType::Minor7, ChordType::Dominant7, ChordType::Major9],
                drum_kit_preference: vec![DrumKit::Electronic808, DrumKit::HipHop],
                bass_style: BassStyle::Reese,
                melody_density: MelodyDensity::Dense,
                arrangement_style: ArrangementStyle::Consistent,
            },
            Genre::Jazz => GenreConfig {
                tempo_min: 90.0,
                tempo_max: 140.0,
                preferred_scales: vec![ScaleType::Dorian, ScaleType::Mixolydian, ScaleType::Lydian, ScaleType::Major],
                preferred_chord_types: vec![ChordType::Major7, ChordType::Minor7, ChordType::Dominant7, ChordType::Major9, ChordType::Minor9],
                drum_kit_preference: vec![DrumKit::Jazz, DrumKit::Acoustic],
                bass_style: BassStyle::Upright,
                melody_density: MelodyDensity::Moderate,
                arrangement_style: ArrangementStyle::Groove,
            },
            Genre::Funk => GenreConfig {
                tempo_min: 100.0,
                tempo_max: 130.0,
                preferred_scales: vec![ScaleType::Mixolydian, ScaleType::Dorian, ScaleType::Major],
                preferred_chord_types: vec![ChordType::Dominant7, ChordType::Minor7, ChordType::Sus4],
                drum_kit_preference: vec![DrumKit::Acoustic, DrumKit::HipHop],
                bass_style: BassStyle::Finger,
                melody_density: MelodyDensity::Moderate,
                arrangement_style: ArrangementStyle::Groove,
            },
            Genre::HipHop => GenreConfig {
                tempo_min: 80.0,
                tempo_max: 110.0,
                preferred_scales: vec![ScaleType::Minor, ScaleType::MinorPentatonic, ScaleType::Blues],
                preferred_chord_types: vec![ChordType::Minor, ChordType::Minor7, ChordType::Sus4],
                drum_kit_preference: vec![DrumKit::HipHop, DrumKit::Electronic808],
                bass_style: BassStyle::Synth,
                melody_density: MelodyDensity::Moderate,
                arrangement_style: ArrangementStyle::VerseChorus,
            },
            Genre::ElectroSwing => GenreConfig {
                tempo_min: 120.0,
                tempo_max: 140.0,
                preferred_scales: vec![ScaleType::Major, ScaleType::Mixolydian, ScaleType::Dorian],
                preferred_chord_types: vec![ChordType::Dominant7, ChordType::Major7, ChordType::Minor7],
                drum_kit_preference: vec![DrumKit::Acoustic, DrumKit::Electronic808],
                bass_style: BassStyle::Upright,
                melody_density: MelodyDensity::Moderate,
                arrangement_style: ArrangementStyle::Groove,
            },
        }
    }
}

/// Select a random genre with weighted distribution
/// Can favor lofi but allows variety
pub fn select_random_genre() -> Genre {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..100);
    
    match roll {
        0..=35 => Genre::Lofi,          // 35% - Still favor lofi
        36..=50 => Genre::Rock,          // 15% - Rock
        51..=65 => Genre::Dubstep,       // 15% - Dubstep
        66..=80 => Genre::DnB,           // 15% - DnB
        81..=87 => Genre::Jazz,          // 7% - Jazz
        88..=93 => Genre::Funk,          // 6% - Funk
        94..=97 => Genre::HipHop,        // 4% - HipHop
        _ => Genre::ElectroSwing,        // 3% - ElectroSwing
    }
}

/// Get genre config for a genre
pub fn get_genre_config(genre: Genre) -> GenreConfig {
    genre.config()
}

