use super::instruments::{generate_heavy_rhythm_guitar, generate_lead_guitar, generate_electric_guitar};
use super::synthesizer::{get_sample_rate, init_sample_rate}; // Fix imports
use crate::composition::genre::MelodyDensity;
use crate::composition::music_theory::{midi_to_freq, Chord, Key, MidiNote};
use crate::composition::{Genre, Section};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MelodyStyle {
    Chug,         // 0-0-0-0
    SludgeRiff,   // Slow chromatic movement
    Drone,        // Sustained feedback
}

#[derive(Clone, Copy)]
pub enum InstrumentType {
    ElectricGuitar,
    HeavyRhythmGuitar,
    LeadGuitar,
}

pub fn generate_melody_with_style_and_instrument(
    key: &Key,
    chords: &[Chord],
    tempo: f32,
    bars: usize,
    _melody_cfg: &crate::config::MelodyConfig,
    _melody_density: MelodyDensity,
    section: Option<Section>,
    _genre: Option<Genre>,
    instrument_preference: Option<InstrumentType>,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;
    let mut melody = Vec::new();
    let mut rng = rand::thread_rng();

    // Determine style based on section
    let style = match section {
        Some(Section::Chorus) => MelodyStyle::SludgeRiff,
        Some(Section::Verse) => MelodyStyle::Chug,
        Some(Section::Intro) => MelodyStyle::Drone,
        _ => MelodyStyle::SludgeRiff,
    };

    let instrument = instrument_preference.unwrap_or(InstrumentType::HeavyRhythmGuitar);

    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        let root = chord.root;

        let pattern = match style {
            MelodyStyle::Chug => {
                // Generate rhythmically interesting 0s (Root notes)
                // E.g., "DUM ... DUM-DUM ... DUM"
                generate_chug_pattern(root, bar_duration, &mut rng, instrument)
            },
            MelodyStyle::SludgeRiff => {
                // Slow moving power chords: Root -> b2 -> b5
                generate_sludge_riff(root, bar_duration, &mut rng, instrument)
            },
            MelodyStyle::Drone => {
                // Single held note with feedback feel
                let freq = midi_to_freq(root);
                // Use Lead for drone to get feedback sustain
                let samples = generate_lead_guitar(freq, bar_duration, 0.7);
                samples
            }
        };

        melody.extend(pattern);
    }
    melody
}

fn generate_chug_pattern(root: u8, duration: f32, rng: &mut impl Rng, inst: InstrumentType) -> Vec<f32> {
    let mut bar = vec![0.0; (duration * get_sample_rate() as f32) as usize];
    let freq = midi_to_freq(root - 12); // Drop octave for heaviness

    // Divide bar into 16ths
    let step = duration / 16.0;
    
    // Create a rhythmic pattern (1 = Hit, 0 = Rest)
    // Common metal gallops or straight 8ths
    let pattern = if rng.gen_bool(0.5) {
        vec![1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0] // Gallop-ish
    } else {
        vec![1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0] // Slow chug
    };

    for (i, &hit) in pattern.iter().enumerate() {
        if hit == 1 {
            let note_dur = step * 1.5; // Slight overlap
            // Randomize velocity for "human" palm mute variation
            let vel = rng.gen_range(0.6..1.0); 
            
            // Generate note
            let samples = match inst {
                InstrumentType::LeadGuitar => generate_lead_guitar(freq, note_dur, vel),
                _ => generate_heavy_rhythm_guitar(freq, note_dur, vel),
            };

            let start_sample = (i as f32 * step * get_sample_rate() as f32) as usize;
            for (j, s) in samples.iter().enumerate() {
                if start_sample + j < bar.len() {
                    bar[start_sample + j] += s;
                }
            }
        }
    }
    bar
}

fn generate_sludge_riff(root: u8, duration: f32, rng: &mut impl Rng, inst: InstrumentType) -> Vec<f32> {
    let mut bar = vec![0.0; (duration * get_sample_rate() as f32) as usize];
    
    // Sludge/Doom relies on the "Devil's Interval" (Tritone/b5) and b2
    let intervals = vec![0, 1, 3, 5, 6, 7]; 
    
    // Generate 2 or 4 notes per bar (Slow)
    let steps = 4;
    let step_dur = duration / steps as f32;

    for i in 0..steps {
        // High chance to return to root
        let interval = if rng.gen_bool(0.4) { 0 } else { intervals[rng.gen_range(0..intervals.len())] };
        let note = root.saturating_sub(12) + interval; // Drop octave
        let freq = midi_to_freq(note);

        let samples = match inst {
            InstrumentType::LeadGuitar => generate_lead_guitar(freq, step_dur, 0.9),
            _ => generate_heavy_rhythm_guitar(freq, step_dur, 0.95),
        };

        let start_sample = (i as f32 * step_dur * get_sample_rate() as f32) as usize;
        for (j, s) in samples.iter().enumerate() {
            if start_sample + j < bar.len() {
                bar[start_sample + j] += s;
            }
        }
    }
    bar
}