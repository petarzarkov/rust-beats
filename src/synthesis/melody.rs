use super::synthesizer::*;
use crate::composition::music_theory::{Key, Chord, midi_to_freq, MidiNote};
use rand::Rng;

/// Generate a melody following the key and chord progression
pub fn generate_melody(
    key: &Key,
    chords: &[Chord],
    tempo: f32,
    bars: usize,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let bar_duration = beat_duration * 4.0;
    
    let scale_notes = key.get_scale_notes_range(3); // 3 octaves
    let mut melody = Vec::new();
    let mut rng = rand::thread_rng();
    
    for bar_idx in 0..bars {
        let chord = &chords[bar_idx % chords.len()];
        
        // Decide if this bar has melody (60% chance)
        if rng.gen_range(0..100) < 60 {
            let pattern = generate_melody_phrase(
                &scale_notes,
                chord,
                bar_duration,
                &mut rng,
            );
            melody.extend(pattern);
        } else {
            // Rest bar
            let silence_samples = (bar_duration * SAMPLE_RATE as f32) as usize;
            melody.extend(vec![0.0; silence_samples]);
        }
    }
    
    melody
}

/// Generate a melodic phrase for one bar
fn generate_melody_phrase(
    scale_notes: &[MidiNote],
    chord: &Chord,
    bar_duration: f32,
    rng: &mut impl Rng,
) -> Vec<f32> {
    let mut phrase = vec![0.0; (bar_duration * SAMPLE_RATE as f32) as usize];
    
    // Get chord tones for this chord (emphasize these)
    let chord_tones = chord.get_notes();
    
    // Generate 2-4 notes per bar
    let num_notes = rng.gen_range(2..=4);
    
    for note_idx in 0..num_notes {
        // Position in the bar (prefer on-beat)
        let beat_position = if rng.gen_range(0..100) < 70 {
            // On beat
            (note_idx as f32) * (4.0 / num_notes as f32)
        } else {
            // Syncopated
            (note_idx as f32) * (4.0 / num_notes as f32) + 0.5
        };
        
        let start_time = (beat_position / 4.0) * bar_duration;
        let duration = bar_duration / (num_notes as f32 * 1.5);
        
        // Choose a note (70% chord tone, 30% scale tone)
        let midi_note = if rng.gen_range(0..100) < 70 && !chord_tones.is_empty() {
            // Chord tone
            chord_tones[rng.gen_range(0..chord_tones.len())] + 12 // Octave up
        } else {
            // Scale tone
            scale_notes[rng.gen_range(0..scale_notes.len())]
        };
        
        let frequency = midi_to_freq(midi_note);
        let note = generate_melody_note(frequency, duration, 0.3);
        
        // Add to phrase
        let start_sample = (start_time * SAMPLE_RATE as f32) as usize;
        for (i, &sample) in note.iter().enumerate() {
            let idx = start_sample + i;
            if idx < phrase.len() {
                phrase[idx] += sample;
            }
        }
    }
    
    phrase
}

/// Generate a single melody note with vibrato
fn generate_melody_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut rng = rand::thread_rng();
    
    let envelope = Envelope::lead();
    let note_off_time = duration * 0.8;
    
    // Vary the waveform for different melodic timbres
    let waveform = match rng.gen_range(0..100) {
        0..=55 => Waveform::Saw,      // Bright, cutting lead (most common)
        56..=75 => Waveform::Square,  // Retro, video game sound
        _ => Waveform::Triangle,      // Smooth, flute-like
    };
    
    // Lead synth with varied timbre
    let mut lead_osc = Oscillator::new(waveform, frequency);
    let mut filter = LowPassFilter::new(2000.0, 0.5);
    let mut vibrato_lfo = LFO::new(5.0, 0.02); // 5Hz vibrato
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));
        
        // Apply vibrato
        let vibrato = vibrato_lfo.next_value();
        lead_osc.frequency = frequency * (1.0 + vibrato);
        
        let mut sample = lead_osc.next_sample();
        
        // Filter sweep
        filter.cutoff = 1500.0 + env_amp * 1500.0;
        sample = filter.process(sample);
        
        samples.push(sample * env_amp * velocity);
    }
    
    samples
}

/// Generate an arpeggio from a chord
pub fn generate_arpeggio(
    chord: &Chord,
    tempo: f32,
    duration: f32,
) -> Vec<f32> {
    let beat_duration = 60.0 / tempo;
    let sixteenth_duration = beat_duration / 4.0;
    
    let chord_notes = chord.get_notes();
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut arpeggio = vec![0.0; num_samples];
    
    let mut note_idx = 0;
    let mut time = 0.0;
    
    while time < duration {
        let midi_note = chord_notes[note_idx % chord_notes.len()] + 24; // Two octaves up
        let frequency = midi_to_freq(midi_note);
        
        let note = generate_arp_note(frequency, sixteenth_duration * 0.8, 0.2);
        
        let start_sample = (time * SAMPLE_RATE as f32) as usize;
        for (i, &sample) in note.iter().enumerate() {
            let idx = start_sample + i;
            if idx < arpeggio.len() {
                arpeggio[idx] += sample;
            }
        }
        
        time += sixteenth_duration;
        note_idx += 1;
    }
    
    arpeggio
}

/// Generate a single arpeggio note
fn generate_arp_note(frequency: f32, duration: f32, velocity: f32) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    
    let envelope = Envelope {
        attack: 0.001,
        decay: 0.1,
        sustain: 0.3,
        release: 0.05,
    };
    
    let note_off_time = duration * 0.9;
    let mut square_osc = Oscillator::new(Waveform::Square, frequency);
    
    for i in 0..num_samples {
        let time = i as f32 / SAMPLE_RATE as f32;
        let env_amp = envelope.get_amplitude(time, Some(note_off_time));
        
        let sample = square_osc.next_sample() * env_amp * velocity;
        samples.push(sample);
    }
    
    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::music_theory::{Key, generate_chord_progression};
    
    #[test]
    fn test_melody_generation() {
        let key = Key::random_funky();
        let chords = generate_chord_progression(&key, 4);
        let melody = generate_melody(&key, &chords, 110.0, 2);
        assert!(!melody.is_empty());
    }
}

