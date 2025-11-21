pub mod bass;
pub mod drums;
pub mod fx;
pub mod instruments;
pub mod lofi_effects;
pub mod melody;
pub mod pads;
pub mod percussion;
pub mod synthesizer;

pub use bass::{generate_dnb_bassline, generate_dubstep_bassline, generate_rock_bassline};
pub use drums::{
    generate_clap, generate_conga, generate_hihat, generate_kick, generate_shaker, generate_snare,
};
pub use fx::{generate_crash, generate_downlifter, generate_impact, generate_riser};
pub use lofi_effects::LofiProcessor;
pub use melody::InstrumentType;
pub use pads::{generate_drone, generate_pads};
pub use synthesizer::{get_sample_rate, init_sample_rate};
