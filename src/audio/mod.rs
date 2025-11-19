pub mod renderer;
pub mod mixer;
pub mod encoder;
pub mod renderers;
pub mod dynamics;
pub mod mixing_presets;
pub mod track_builder;

pub use renderer::{render_to_wav_with_metadata, SongMetadata};
pub use mixer::{Track, mix_tracks, master_lofi, stereo_to_mono, normalize_loudness};
pub use encoder::encode_to_mp3;
pub use renderers::{
    render_arranged_drums, render_arranged_bass, render_arranged_melody,
    generate_pads_with_arrangement, render_fx_track, add_percussion_track,
};
pub use dynamics::apply_arrangement_dynamics;
pub use mixing_presets::{MixingPreset, preset_from_str, get_mixing_preset};
pub use track_builder::build_tracks;

