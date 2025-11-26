// Metal-focused synthesis modules
pub mod drums;
pub mod fx;
pub mod synthesizer;
pub mod metal_dsp;       // Advanced distortion and noise gate for metal
pub mod karplus_strong;  // String synthesis for guitar/bass
pub mod cabinet;         // Cabinet simulation
pub mod metal_audio_renderer; // Complete metal audio rendering
pub mod mixing;          // Reverb, EQ, and compression

// Core exports
pub use synthesizer::{get_sample_rate, init_sample_rate};
pub use metal_audio_renderer::MetalAudioRenderer;
