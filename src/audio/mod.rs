pub mod dynamics;
pub mod encoder;
pub mod mixer;
pub mod mixing_presets;
pub mod renderer;
pub mod renderers;
pub mod track_builder;

pub use dynamics::apply_arrangement_dynamics;
pub use encoder::encode_to_mp3;
pub use mixer::{master_lofi, mix_tracks, normalize_loudness, stereo_to_mono, Track};
pub use mixing_presets::preset_from_str;
pub use renderer::{render_to_wav_with_metadata, SongMetadata};
pub use renderers::{
    add_percussion_track, generate_pads_with_arrangement, render_arranged_bass,
    render_arranged_drums, render_arranged_melody_with_instrument, render_fx_track,
};
pub use track_builder::build_tracks;
