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
    /// Generate a standard song arrangement (3 minutes / ~180 seconds)
    pub fn generate_standard() -> Self {
        let mut rng = rand::thread_rng();
        
        // Full-length song structures (85-90 bars for ~3 minutes at typical tempo)
        let structures = vec![
            // Classic pop structure with extended sections
            vec![
                (Section::Intro, 8),
                (Section::Verse, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Bridge, 8),
                (Section::Verse, 4),    // Short verse variation
                (Section::Chorus, 8),
                (Section::Chorus, 8),   // Double chorus for finale
                (Section::Outro, 8),
            ],
            // Funk/Jazz structure with instrumental break
            vec![
                (Section::Intro, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Bridge, 12),  // Extended bridge for solo
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Chorus, 8),   // Extra chorus
                (Section::Outro, 8),
            ],
            // Extended groove structure
            vec![
                (Section::Intro, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Bridge, 8),
                (Section::Chorus, 8),
                (Section::Bridge, 8),   // Extended break
                (Section::Chorus, 8),
                (Section::Chorus, 8),   // Extended outro chorus
                (Section::Outro, 8),
            ],
            // Verse-heavy structure
            vec![
                (Section::Intro, 8),
                (Section::Verse, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Verse, 8),
                (Section::Verse, 8),
                (Section::Chorus, 8),
                (Section::Bridge, 8),
                (Section::Chorus, 8),
                (Section::Chorus, 4),   // Half chorus
                (Section::Outro, 8),
            ],
        ];
        
        let chosen = &structures[rng.gen_range(0..structures.len())];
        let total_bars = chosen.iter().map(|(_, bars)| bars).sum();
        
        Arrangement {
            sections: chosen.clone(),
            total_bars,
        }
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
            Section::Intro => false,      // Drums and bass only
            Section::Verse => true,       // Add melody
            Section::Chorus => true,      // Full melody
            Section::Bridge => true,      // Melodic variation
            Section::Outro => false,      // Fade out with drums/bass
        }
    }
    
    /// Should this section have heavy bass?
    pub fn section_has_heavy_bass(section: Section) -> bool {
        match section {
            Section::Intro => true,
            Section::Verse => true,
            Section::Chorus => true,
            Section::Bridge => false,     // Lighter bass for contrast
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
            (Section::Bridge, Section::Chorus) => true,  // Build to finale
            (Section::Intro, Section::Verse) => true,    // Build from intro
            _ => false,
        }
    }
    
    /// Determine if section transition should have a breakdown
    pub fn needs_breakdown(from: Section, to: Section) -> bool {
        match (from, to) {
            (Section::Chorus, Section::Bridge) => true,  // Drop energy
            (Section::Chorus, Section::Verse) => true,   // Return to verse
            (_, Section::Outro) => true,                 // Fade out
            _ => false,
        }
    }
    
    /// Should pads be playing in this section?
    pub fn section_has_pads(section: Section) -> bool {
        match section {
            Section::Intro => true,       // Atmospheric intro
            Section::Verse => false,      // Keep it clean
            Section::Chorus => true,      // Full sound
            Section::Bridge => true,      // Atmospheric break
            Section::Outro => true,       // Fade with pads
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DrumComplexity {
    Simple,   // Basic kick and snare
    Medium,   // Add hi-hats and variations
    Complex,  // Full kit with fills
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arrangement_generation() {
        let arr = Arrangement::generate_standard();
        assert!(arr.total_bars >= 40); // At least ~1 minute at typical tempo
        assert!(arr.total_bars <= 80); // Not too long
        
        // Should have intro and outro
        assert_eq!(arr.sections.first().unwrap().0, Section::Intro);
        assert_eq!(arr.sections.last().unwrap().0, Section::Outro);
    }
    
    #[test]
    fn test_section_lookup() {
        let arr = Arrangement {
            sections: vec![
                (Section::Intro, 4),
                (Section::Verse, 8),
            ],
            total_bars: 12,
        };
        
        assert_eq!(arr.get_section_at_bar(0), Some(Section::Intro));
        assert_eq!(arr.get_section_at_bar(3), Some(Section::Intro));
        assert_eq!(arr.get_section_at_bar(4), Some(Section::Verse));
        assert_eq!(arr.get_section_at_bar(11), Some(Section::Verse));
    }
}

