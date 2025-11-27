// Metal-focused synthesis modules
pub mod drums;
pub mod fx;
pub mod filters;         // Basic filters (LowPass, etc.)
pub mod metal_dsp;       // Advanced distortion and noise gate for metal
pub mod karplus_strong;  // String synthesis for guitar/bass
pub mod cabinet;         // Cabinet simulation
pub mod metal_audio_renderer; // Complete metal audio rendering
pub mod mixing;          // Reverb, EQ, and compression

// Core exports
pub use crate::utils::{get_sample_rate, init_sample_rate};
