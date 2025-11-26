pub mod bass;
pub mod drums;
pub mod fx;
pub mod instruments;
pub mod lofi_effects;
pub mod melody;
pub mod pads;
pub mod percussion;
pub mod synthesizer;

pub use bass::generate_metal_bassline;
pub use drums::{
    generate_hihat, generate_kick, generate_snare, generate_china, generate_ride, generate_tom, generate_crash,
};
pub use fx::{generate_downlifter, generate_impact, generate_riser};
pub use lofi_effects::LofiProcessor;
pub use melody::InstrumentType;
pub use pads::{generate_drone, generate_pads};
pub use synthesizer::{get_sample_rate, init_sample_rate};
