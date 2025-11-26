use crate::composition::{
    drum_humanizer::{DrumHumanizer, BlastBeatStyle, generate_blast_beat, blast_beat_velocity},
    fretboard::{FretboardPathfinder, calculate_playability_score},
    music_theory::{Key, ScaleType, MidiNote},
    riff_generator::{PedalPointGenerator, MetalMarkovPresets},
    tuning::GuitarTuning,
};
use rand::Rng;

/// Legacy genre enum for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    SwampMetal,
}


/// Metal song structure sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetalSection {
    Intro,
    Verse,
    Chorus,
    Breakdown,
    Solo,
    Outro,
}

/// Metal subgenre for style-specific generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetalSubgenre {
    HeavyMetal,     // Traditional heavy metal
    ThrashMetal,    // Fast, aggressive
    DeathMetal,     // Brutal, low-tuned
    DoomMetal,      // Slow, heavy
    ProgressiveMetal, // Complex, technical
}

impl MetalSubgenre {
    /// Get appropriate guitar tuning for subgenre
    pub fn default_tuning(&self) -> GuitarTuning {
        match self {
            MetalSubgenre::HeavyMetal => GuitarTuning::EStandard,
            MetalSubgenre::ThrashMetal => GuitarTuning::EStandard,
            MetalSubgenre::DeathMetal => GuitarTuning::DStandard,
            MetalSubgenre::DoomMetal => GuitarTuning::CStandard,
            MetalSubgenre::ProgressiveMetal => GuitarTuning::DropC,
        }
    }

    /// Get appropriate scale for subgenre
    pub fn default_scale(&self) -> ScaleType {
        match self {
            MetalSubgenre::HeavyMetal => ScaleType::MinorPentatonic,
            MetalSubgenre::ThrashMetal => ScaleType::Phrygian,
            MetalSubgenre::DeathMetal => ScaleType::Phrygian,
            MetalSubgenre::DoomMetal => ScaleType::Dorian,
            MetalSubgenre::ProgressiveMetal => ScaleType::HarmonicMinor,
        }
    }

    /// Get tempo range for subgenre (min, max BPM)
    pub fn tempo_range(&self) -> (u16, u16) {
        match self {
            MetalSubgenre::HeavyMetal => (120, 160),
            MetalSubgenre::ThrashMetal => (160, 220),
            MetalSubgenre::DeathMetal => (140, 200),
            MetalSubgenre::DoomMetal => (60, 100),
            MetalSubgenre::ProgressiveMetal => (100, 180),
        }
    }
}

/// A complete metal riff with notes and palm muting
#[derive(Debug, Clone)]
pub struct MetalRiff {
    pub notes: Vec<MidiNote>,
    pub palm_muted: Vec<bool>,
    pub playability_score: f32,
}

/// A complete metal song structure
#[derive(Debug, Clone)]
pub struct MetalSong {
    pub subgenre: MetalSubgenre,
    pub key: Key,
    pub tempo: u16,
    pub tuning: GuitarTuning,
    pub sections: Vec<(MetalSection, MetalRiff)>,
    pub drum_humanizer: DrumHumanizer,
}

/// Metal song generator - integrates all components
pub struct MetalSongGenerator {
    subgenre: MetalSubgenre,
    tuning: GuitarTuning,
    key: Key,
    tempo: u16,
}

impl MetalSongGenerator {
    /// Create a new metal song generator
    pub fn new(subgenre: MetalSubgenre) -> Self {
        let mut rng = rand::thread_rng();
        
        // Choose tuning and scale based on subgenre
        let tuning = subgenre.default_tuning();
        let scale_type = subgenre.default_scale();
        
        // Choose root note based on tuning
        let root = tuning.lowest_note();
        let key = Key {
            root,
            scale_type,
        };
        
        // Choose tempo within subgenre range
        let (min_tempo, max_tempo) = subgenre.tempo_range();
        let tempo = rng.gen_range(min_tempo..=max_tempo);
        
        MetalSongGenerator {
            subgenre,
            tuning,
            key,
            tempo,
        }
    }

    /// Generate a complete metal riff for a section
    pub fn generate_riff(&self, section: MetalSection) -> MetalRiff {
        let (bars, notes_per_bar) = match section {
            MetalSection::Intro => (1, 8),
            MetalSection::Verse => (2, 8),
            MetalSection::Chorus => (2, 8),
            MetalSection::Breakdown => (1, 8),
            MetalSection::Solo => (4, 8),
            MetalSection::Outro => (1, 8),
        };

        // Generate riff using pedal point generator
        let pedal_gen = PedalPointGenerator::from_key(&self.key);
        let riff_pattern = pedal_gen.generate_riff_pattern(bars, notes_per_bar);

        // Extract notes and palm muting
        let (notes, palm_muted): (Vec<_>, Vec<_>) = riff_pattern.into_iter().unzip();

        // Validate playability
        let pathfinder = FretboardPathfinder::new(self.tuning);
        let fret_positions = pathfinder.find_playable_path(&notes);
        let playability_score = calculate_playability_score(&fret_positions);

        MetalRiff {
            notes,
            palm_muted,
            playability_score,
        }
    }

    /// Generate a complete metal song structure
    pub fn generate_song(&self) -> MetalSong {
        let mut sections = Vec::new();

        // Standard metal song structure
        sections.push((MetalSection::Intro, self.generate_riff(MetalSection::Intro)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Verse, self.generate_riff(MetalSection::Verse)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Breakdown, self.generate_riff(MetalSection::Breakdown)));
        sections.push((MetalSection::Solo, self.generate_riff(MetalSection::Solo)));
        sections.push((MetalSection::Chorus, self.generate_riff(MetalSection::Chorus)));
        sections.push((MetalSection::Outro, self.generate_riff(MetalSection::Outro)));

        // Choose drum humanizer based on subgenre
        let drum_humanizer = match self.subgenre {
            MetalSubgenre::HeavyMetal => DrumHumanizer::new(),
            MetalSubgenre::ThrashMetal => DrumHumanizer::thrash(),
            MetalSubgenre::DeathMetal => DrumHumanizer::blast_beat(),
            MetalSubgenre::DoomMetal => DrumHumanizer::breakdown(),
            MetalSubgenre::ProgressiveMetal => DrumHumanizer::new(),
        };

        MetalSong {
            subgenre: self.subgenre,
            key: self.key,
            tempo: self.tempo,
            tuning: self.tuning,
            sections,
            drum_humanizer,
        }
    }

    /// Generate drum pattern for a section
    pub fn generate_drums(&self, section: MetalSection, humanizer: &DrumHumanizer) -> Vec<(u8, i32)> {
        let mut drum_hits = Vec::new();
        let subdivisions = match section {
            MetalSection::Breakdown => 8,  // Slower, heavier
            _ => 16, // Standard 16th notes
        };

        // Generate blast beat for appropriate sections
        let use_blast = matches!(
            (self.subgenre, section),
            (MetalSubgenre::DeathMetal, MetalSection::Verse) |
            (MetalSubgenre::DeathMetal, MetalSection::Chorus)
        );

        if use_blast {
            let (kicks, snares) = generate_blast_beat(BlastBeatStyle::Traditional, subdivisions);
            
            for i in 0..subdivisions {
                if kicks[i] || snares[i] {
                    let base_velocity = blast_beat_velocity(100, i == 0);
                    let (velocity, timing) = humanizer.humanize_hit(base_velocity, i == 0);
                    drum_hits.push((velocity, timing));
                }
            }
        } else {
            // Standard pattern
            for i in 0..subdivisions {
                let is_accent = i % 4 == 0; // Accent on beats
                let base_velocity = if is_accent { 110 } else { 95 };
                let (velocity, timing) = humanizer.humanize_hit(base_velocity, is_accent);
                drum_hits.push((velocity, timing));
            }
        }

        drum_hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metal_subgenre_tuning() {
        assert_eq!(MetalSubgenre::HeavyMetal.default_tuning(), GuitarTuning::EStandard);
        assert_eq!(MetalSubgenre::DeathMetal.default_tuning(), GuitarTuning::DStandard);
    }

    #[test]
    fn test_metal_subgenre_tempo() {
        let (min, max) = MetalSubgenre::ThrashMetal.tempo_range();
        assert!(min >= 160);
        assert!(max <= 220);
    }

    #[test]
    fn test_song_generator_creation() {
        let generator = MetalSongGenerator::new(MetalSubgenre::HeavyMetal);
        assert_eq!(generator.subgenre, MetalSubgenre::HeavyMetal);
        assert!(generator.tempo >= 120 && generator.tempo <= 160);
    }

    #[test]
    fn test_generate_riff() {
        let generator = MetalSongGenerator::new(MetalSubgenre::DeathMetal);
        let riff = generator.generate_riff(MetalSection::Verse);
        
        assert_eq!(riff.notes.len(), 16);
        assert_eq!(riff.palm_muted.len(), 16);
        assert!(riff.playability_score >= 0.0 && riff.playability_score <= 1.0);
    }

    #[test]
    fn test_generate_song() {
        let generator = MetalSongGenerator::new(MetalSubgenre::ThrashMetal);
        let song = generator.generate_song();
        
        assert_eq!(song.subgenre, MetalSubgenre::ThrashMetal);
        assert!(song.sections.len() > 0);
        assert!(song.tempo >= 160 && song.tempo <= 220);
    }

    #[test]
    fn test_generate_drums() {
        let generator = MetalSongGenerator::new(MetalSubgenre::DeathMetal);
        let humanizer = DrumHumanizer::blast_beat();
        let drums = generator.generate_drums(MetalSection::Verse, &humanizer);
        
        assert!(drums.len() > 0);
        // Check velocity and timing are within valid ranges
        for (velocity, timing) in drums {
            assert!(velocity >= 1 && velocity <= 127);
            assert!(timing.abs() < 100); // Reasonable timing offset
        }
    }

    #[test]
    fn test_all_subgenres() {
        let subgenres = vec![
            MetalSubgenre::HeavyMetal,
            MetalSubgenre::ThrashMetal,
            MetalSubgenre::DeathMetal,
            MetalSubgenre::DoomMetal,
            MetalSubgenre::ProgressiveMetal,
        ];

        for subgenre in subgenres {
            let generator = MetalSongGenerator::new(subgenre);
            let song = generator.generate_song();
            assert_eq!(song.subgenre, subgenre);
        }
    }

    #[test]
    fn test_riff_playability() {
        let generator = MetalSongGenerator::new(MetalSubgenre::ProgressiveMetal);
        let riff = generator.generate_riff(MetalSection::Solo);
        
        // Solo should be longer
        assert_eq!(riff.notes.len(), 32);
        // Should have a playability score
        assert!(riff.playability_score > 0.0);
    }
}
