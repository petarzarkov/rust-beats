pub mod song_names;
pub mod music_theory;
pub mod beat_maker;
pub mod arranger;
pub mod genre;
pub mod song_generator;

pub use song_names::{generate_song_name, generate_genre_tags};
pub use music_theory::{Key, Chord, Tempo, generate_chord_progression};
pub use beat_maker::{DrumHit, GrooveStyle, generate_drum_pattern, select_random_drum_kit};
pub use arranger::{Arrangement, Section};
pub use genre::{Genre, BassStyle, select_random_genre, get_genre_config};
pub use song_generator::{SongGenerator, SongParams};

