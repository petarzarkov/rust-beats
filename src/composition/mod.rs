// Metal-focused composition modules
pub mod music_theory;
pub mod song_names;
pub mod tuning;         // Guitar tunings for metal
pub mod rhythm;         // Euclidean and polymetric rhythms
pub mod riff_generator; // Markov chains and pedal point riff generation
pub mod fretboard;      // Fretboard pathfinding for playable riffs
pub mod drum_humanizer; // Drum humanization for realistic metal drums
pub mod metal_song_generator; // Complete metal song generation
pub mod bass_generator; // Bass line generation for metal

// Core exports used by main
pub use music_theory::{Key, Chord, ScaleType};
pub use song_names::{generate_genre_tags, generate_song_name};
pub use metal_song_generator::MetalSong;
pub mod rhythm_generator;

// Legacy export for compatibility (maps to metal)
pub use metal_song_generator::Genre;
