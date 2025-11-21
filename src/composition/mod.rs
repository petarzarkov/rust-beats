pub mod arranger;
pub mod beat_maker;
pub mod genre;
pub mod music_theory;
pub mod song_generator;
pub mod song_names;

pub use arranger::{Arrangement, Section};
pub use beat_maker::{select_preferred_drum_kit, GrooveStyle};
pub use genre::{get_genre_config, select_random_genre, Genre};
pub use music_theory::{generate_chord_progression, Key, Tempo};
pub use song_generator::SongGenerator;
pub use song_names::{generate_genre_tags, generate_song_name};
