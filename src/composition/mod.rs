pub mod song_names;
pub mod music_theory;
pub mod beat_maker;
pub mod arranger;

pub use song_names::{generate_song_name, generate_genre_tags};
pub use music_theory::{Key, Chord, Tempo, generate_chord_progression};
pub use beat_maker::{DrumHit, GrooveStyle, generate_drum_pattern, random_groove_style, select_random_drum_kit};
pub use arranger::{Arrangement, Section};

