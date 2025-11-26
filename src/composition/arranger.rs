use crate::composition::genre::ArrangementStyle;
use rand::Rng;

/// Musical sections in a song
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Section {
    Intro,
    Verse,
    Chorus,
    Bridge,
    Outro,
}

/// Arrangement structure defining the song layout
#[derive(Debug, Clone)]
pub struct Arrangement {
    pub sections: Vec<(Section, usize)>, // Section type and number of bars
    pub total_bars: usize,
}

impl Arrangement {
    /// Generate a standard song arrangement (3 minutes / ~180 seconds) with variance
    pub fn generate_standard() -> Self {
        Self::generate_from_templates(&verse_chorus_templates())
    }

    /// Generate an arrangement that respects the genre's preferred song arc
    pub fn generate_with_style(style: ArrangementStyle) -> Self {
        match style {
            ArrangementStyle::VerseChorus => Self::generate_standard(),
            ArrangementStyle::Linear => Self::generate_from_templates(&consistent_templates()),
        }
    }

    /// Generate an arrangement aiming for a specific duration in seconds
    pub fn generate_for_duration(style: ArrangementStyle, tempo_bpm: f32, target_seconds: f32) -> Self {
        // Calculate roughly how many bars we need
        // bars = (seconds * bpm) / (60 * 4)
        // Assumes 4/4 time
        let target_bars = (target_seconds * tempo_bpm / 240.0).round() as usize;
        
        let mut base_arrangement = Self::generate_with_style(style);
        
        // If we're already close enough (within 10%), just return it
        let diff = (base_arrangement.total_bars as i32 - target_bars as i32).abs();
        if diff < (target_bars / 10) as i32 {
            return base_arrangement;
        }
        
        // If we need more bars, repeat internal sections
        if base_arrangement.total_bars < target_bars {
            let mut new_sections = Vec::new();
            let mut current_bars = 0;
            
            // Always keep intro
            if let Some(first) = base_arrangement.sections.first() {
                new_sections.push(*first);
                current_bars += first.1;
            }
            
            // Middle sections to repeat
            let middle_sections: Vec<(Section, usize)> = base_arrangement.sections
                .iter()
                .skip(1) // Skip intro
                .take(base_arrangement.sections.len() - 2) // Skip outro
                .cloned()
                .collect();
                
            // Repeat middle sections until we're close
            while current_bars < target_bars - 16 { // Reserve space for outro
                for section in &middle_sections {
                    if current_bars >= target_bars - 8 { break; }
                    new_sections.push(*section);
                    current_bars += section.1;
                }
            }
            
            // Always keep outro
            if let Some(last) = base_arrangement.sections.last() {
                new_sections.push(*last);
                current_bars += last.1;
            }
            
            base_arrangement.sections = new_sections;
            base_arrangement.total_bars = current_bars;
        }
        
        base_arrangement
    }

    /// Generate a short song arrangement (30-60 seconds for testing)
    pub fn generate_short() -> Self {
        Arrangement {
            sections: vec![
                (Section::Intro, 2),
                (Section::Verse, 4),
                (Section::Chorus, 4),
                (Section::Outro, 2),
            ],
            total_bars: 12,
        }
    }

    /// Get the section at a specific bar index
    pub fn get_section_at_bar(&self, bar_index: usize) -> Option<Section> {
        let mut current_bar = 0;

        for (section, bars) in &self.sections {
            if bar_index >= current_bar && bar_index < current_bar + bars {
                return Some(*section);
            }
            current_bar += bars;
        }

        None
    }

    /// Get intensity for a section (0.0 = quiet, 1.0 = full)
    pub fn get_section_intensity(section: Section) -> f32 {
        match section {
            Section::Intro => 0.5,
            Section::Verse => 0.7,
            Section::Chorus => 1.0,
            Section::Bridge => 0.8,
            Section::Outro => 0.4,
        }
    }

    /// Should this section have melody?
    pub fn section_has_melody(section: Section) -> bool {
        match section {
            Section::Intro => false, // Drums and bass only
            Section::Verse => true,  // Add melody
            Section::Chorus => true, // Full melody
            Section::Bridge => true, // Melodic variation
            Section::Outro => false, // Fade out with drums/bass
        }
    }

    /// Should this section have heavy bass?
    pub fn section_has_heavy_bass(section: Section) -> bool {
        match section {
            Section::Intro => true,
            Section::Verse => true,
            Section::Chorus => true,
            Section::Bridge => false, // Lighter bass for contrast
            Section::Outro => true,
        }
    }

    /// Get drum complexity for section
    pub fn get_drum_complexity(section: Section) -> DrumComplexity {
        match section {
            Section::Intro => DrumComplexity::Simple,
            Section::Verse => DrumComplexity::Medium,
            Section::Chorus => DrumComplexity::Complex,
            Section::Bridge => DrumComplexity::Medium,
            Section::Outro => DrumComplexity::Simple,
        }
    }

    /// Check if a bar should have a fill (transition to next section)
    pub fn should_add_fill(&self, bar_index: usize) -> bool {
        let mut current_bar = 0;

        for (_, bars) in &self.sections {
            // Add fill on the last bar before a section change
            if bar_index == current_bar + bars - 1 {
                return true;
            }
            current_bar += bars;
        }

        false
    }

    /// Check if this is the start of a new section (for build-ups)
    pub fn is_section_start(&self, bar_index: usize) -> bool {
        let mut current_bar = 0;

        for (_, bars) in &self.sections {
            if bar_index == current_bar {
                return true;
            }
            current_bar += bars;
        }

        false
    }

    /// Get the previous and current section at a bar index (for transitions)
    pub fn get_transition(&self, bar_index: usize) -> Option<(Section, Section)> {
        let mut current_bar = 0;
        let mut prev_section = None;

        for (section, bars) in &self.sections {
            // If we're at the start of this section
            if bar_index == current_bar {
                if let Some(prev) = prev_section {
                    return Some((prev, *section));
                }
            }

            current_bar += bars;
            prev_section = Some(*section);
        }

        None
    }

    /// Determine if section transition should have a build-up
    pub fn needs_buildup(from: Section, to: Section) -> bool {
        match (from, to) {
            (Section::Verse, Section::Chorus) => true,  // Build energy
            (Section::Bridge, Section::Chorus) => true, // Build to finale
            (Section::Intro, Section::Verse) => true,   // Build from intro
            _ => false,
        }
    }

    /// Determine if section transition should have a breakdown
    pub fn needs_breakdown(from: Section, to: Section) -> bool {
        match (from, to) {
            (Section::Chorus, Section::Bridge) => true, // Drop energy
            (Section::Chorus, Section::Verse) => true,  // Return to verse
            (_, Section::Outro) => true,                // Fade out
            _ => false,
        }
    }

    /// Should pads be playing in this section?
    pub fn section_has_pads(section: Section) -> bool {
        match section {
            Section::Intro => true,  // Atmospheric intro
            Section::Verse => false, // Keep it clean
            Section::Chorus => true, // Full sound
            Section::Bridge => true, // Atmospheric break
            Section::Outro => true,  // Fade with pads
        }
    }

    fn generate_from_templates(templates: &[Vec<(Section, usize)>]) -> Self {
        let mut rng = rand::thread_rng();
        let chosen = &templates[rng.gen_range(0..templates.len())];
        let mut sections = Vec::new();

        for (section, bars) in chosen {
            let varied_bars = if rng.gen_range(0..100) < 30 {
                let variation = rng.gen_range(-2..=2);
                ((*bars as i32 + variation).max(2).min(16)) as usize
            } else {
                *bars
            };
            sections.push((*section, varied_bars));
        }

        let total_bars = sections.iter().map(|(_, bars)| bars).sum();

        Arrangement {
            sections,
            total_bars,
        }
    }
}

fn verse_chorus_templates() -> Vec<Vec<(Section, usize)>> {
    vec![
        // Classic Pop Structure
        vec![
            (Section::Intro, 8),
            (Section::Verse, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Bridge, 8),
            (Section::Verse, 4),
            (Section::Chorus, 8),
            (Section::Chorus, 8),
            (Section::Outro, 8),
        ],
        // Radio Edit
        vec![
            (Section::Intro, 4),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Bridge, 8),
            (Section::Chorus, 8),
            (Section::Outro, 4),
        ],
        // Extended Build
        vec![
            (Section::Intro, 16),
            (Section::Verse, 8),
            (Section::Verse, 8),
            (Section::Chorus, 16),
            (Section::Verse, 8),
            (Section::Chorus, 16),
            (Section::Bridge, 8),
            (Section::Chorus, 16),
            (Section::Outro, 16),
        ],
        // Quick Start
        vec![
            (Section::Intro, 2),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Bridge, 8),
            (Section::Chorus, 8),
            (Section::Outro, 4),
        ],
        // Original template 1
        vec![
            (Section::Intro, 6),
            (Section::Verse, 6),
            (Section::Chorus, 10),
            (Section::Verse, 6),
            (Section::Chorus, 10),
            (Section::Bridge, 8),
            (Section::Chorus, 10),
            (Section::Chorus, 8),
            (Section::Outro, 6),
        ],
    ]
}

fn build_drop_templates() -> Vec<Vec<(Section, usize)>> {
    vec![
        // Standard EDM
        vec![
            (Section::Intro, 16),
            (Section::Verse, 16),
            (Section::Bridge, 8), // Buildup
            (Section::Chorus, 16), // Drop
            (Section::Verse, 16),
            (Section::Bridge, 8), // Buildup
            (Section::Chorus, 16), // Drop
            (Section::Outro, 16),
        ],
        // Short Drop
        vec![
            (Section::Intro, 8),
            (Section::Bridge, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Bridge, 8),
            (Section::Chorus, 8),
            (Section::Outro, 8),
        ],
        // Progressive
        vec![
            (Section::Intro, 32),
            (Section::Verse, 16),
            (Section::Bridge, 16),
            (Section::Chorus, 32),
            (Section::Outro, 32),
        ],
    ]
}

fn consistent_templates() -> Vec<Vec<(Section, usize)>> {
    vec![
        // Flowing
        vec![
            (Section::Intro, 16),
            (Section::Verse, 16),
            (Section::Chorus, 16),
            (Section::Verse, 16),
            (Section::Chorus, 16),
            (Section::Outro, 16),
        ],
        // Intense
        vec![
            (Section::Intro, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Outro, 8),
        ],
        // Liquid DnB style
        vec![
            (Section::Intro, 32),
            (Section::Verse, 32),
            (Section::Bridge, 16),
            (Section::Verse, 32),
            (Section::Outro, 32),
        ],
    ]
}

fn groove_templates() -> Vec<Vec<(Section, usize)>> {
    vec![
        // Standard Groove
        vec![
            (Section::Intro, 8),
            (Section::Verse, 16),
            (Section::Chorus, 8),
            (Section::Verse, 16),
            (Section::Chorus, 8),
            (Section::Bridge, 8),
            (Section::Chorus, 16),
            (Section::Outro, 8),
        ],
        // Jam Session
        vec![
            (Section::Intro, 16),
            (Section::Verse, 16),
            (Section::Verse, 16),
            (Section::Bridge, 16),
            (Section::Verse, 16),
            (Section::Outro, 16),
        ],
        // Short Groove
        vec![
            (Section::Intro, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Verse, 8),
            (Section::Chorus, 8),
            (Section::Outro, 8),
        ],
    ]
}
#[derive(Debug, Clone, Copy)]
pub enum DrumComplexity {
    Simple,  // Basic kick and snare
    Medium,  // Add hi-hats and variations
    Complex, // Full kit with fills
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrangement_generation() {
        let arr = Arrangement::generate_standard();
        assert!(arr.total_bars >= 30); // At least ~45 seconds at typical tempo
        assert!(arr.total_bars <= 120); // Allow for variance (was 80, now up to 120 with variance)

        // Should have intro and outro
        assert_eq!(arr.sections.first().unwrap().0, Section::Intro);
        assert_eq!(arr.sections.last().unwrap().0, Section::Outro);
    }

    #[test]
    fn test_section_lookup() {
        let arr = Arrangement {
            sections: vec![(Section::Intro, 4), (Section::Verse, 8)],
            total_bars: 12,
        };

        assert_eq!(arr.get_section_at_bar(0), Some(Section::Intro));
        assert_eq!(arr.get_section_at_bar(3), Some(Section::Intro));
        assert_eq!(arr.get_section_at_bar(4), Some(Section::Verse));
        assert_eq!(arr.get_section_at_bar(11), Some(Section::Verse));
    }
}
