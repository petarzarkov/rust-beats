pub mod synthesizer;
pub mod drums;
pub mod bass;
pub mod melody;
pub mod pads;
pub mod lofi_effects;
pub mod instruments;
pub mod percussion;
pub mod fx;

pub use synthesizer::{SAMPLE_RATE, init_sample_rate};
pub use drums::{generate_kick, generate_snare, generate_hihat, generate_clap, generate_conga, generate_shaker};
pub use bass::{generate_bassline, generate_rock_bassline, generate_dubstep_bassline, generate_dnb_bassline};
pub use melody::{generate_melody};
pub use pads::{generate_pads};
pub use lofi_effects::{LofiProcessor};
pub use fx::{generate_riser, generate_downlifter, generate_crash, generate_impact};

