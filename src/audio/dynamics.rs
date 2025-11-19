/// Volume automation and dynamics processing
use crate::composition::{Arrangement, Section};
use crate::synthesis::SAMPLE_RATE;

/// Apply volume automation based on arrangement sections
/// Intro/Outro: quieter, Verse: normal, Chorus: louder, Bridge: medium
/// Works with interleaved stereo buffer (L, R, L, R, ...)
pub fn apply_arrangement_dynamics(stereo_buffer: &mut Vec<f32>, arrangement: &Arrangement, bpm: f32) {
    let samples_per_bar = (SAMPLE_RATE() as f32 * 60.0 / bpm * 4.0) as usize * 2; // *2 for stereo
    let mut current_sample = 0;
    
    for (section_type, bars) in &arrangement.sections {
        let section_samples = samples_per_bar * bars;
        let section_end = (current_sample + section_samples).min(stereo_buffer.len());
        
        // Determine volume multiplier for this section
        let base_volume = match section_type {
            Section::Intro => 0.75,   // Quieter intro
            Section::Verse => 0.95,   // Standard verse
            Section::Chorus => 1.15,  // Louder chorus
            Section::Bridge => 0.90,  // Medium bridge
            Section::Outro => 0.70,   // Quiet outro
        };
        
        // Apply volume to section with smooth transitions
        let fade_samples = samples_per_bar / 4; // Quarter bar fade
        
        for i in (current_sample..section_end).step_by(2) {
            if i + 1 >= stereo_buffer.len() {
                break;
            }
            
            let progress_in_section = i - current_sample;
            let samples_remaining = section_end - i;
            
            // Smooth fade in/out
            let mut volume = base_volume;
            
            // Fade in at section start
            if progress_in_section < fade_samples {
                let fade_in = progress_in_section as f32 / fade_samples as f32;
                volume *= fade_in;
            }
            
            // Fade out at section end (especially for outro)
            if *section_type == Section::Outro && samples_remaining < section_samples / 2 {
                let fade_out = samples_remaining as f32 / (section_samples as f32 / 2.0);
                volume *= fade_out.max(0.3); // Don't fade completely to silence
            }
            
            // Apply volume to both channels
            stereo_buffer[i] *= volume;
            stereo_buffer[i + 1] *= volume;
        }
        
        current_sample = section_end;
    }
}

