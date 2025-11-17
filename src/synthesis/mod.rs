pub mod synthesizer;
pub mod drums;
pub mod bass;
pub mod melody;
pub mod pads;

pub use synthesizer::{Oscillator, Envelope, Waveform, LowPassFilter, SAMPLE_RATE, generate_note, mix_buffers};
pub use drums::{generate_kick, generate_snare, generate_hihat, generate_clap, generate_conga, generate_shaker};
pub use bass::{generate_bassline, generate_sub_bass};
pub use melody::{generate_melody, generate_arpeggio};
pub use pads::{generate_pads, generate_drone};

