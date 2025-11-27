// Metal-focused composition modules
pub mod music_theory;
pub mod song_names;
pub mod tuning;         // Guitar tunings for metal
pub mod rhythm;         // Euclidean and polymetric rhythms
pub mod riff_generator; // Markov chains and pedal point riff generation
pub mod riff_motifs;    // Riff motif system for asymmetric patterns
pub mod fretboard;      // Fretboard pathfinding for playable riffs
pub mod drum_humanizer; // Drum humanization for realistic metal drums
pub mod drum_articulations; // Drum articulation patterns (ghost notes, flams, etc.)
pub mod breakdown_generator;
pub mod bar_memory;
pub mod phrase_drums;
pub mod metal_song_generator; // Complete metal song generation
pub mod bass_generator; // Bass line generation for metal

// Core exports used by main
pub use song_names::{generate_genre_tags, generate_song_name};
pub mod rhythm_generator;

// Legacy export for compatibility (maps to metal)
pub use metal_song_generator::Genre;
