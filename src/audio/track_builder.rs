/// Build audio tracks from generated audio sources
use super::{mixing_presets::MixingPreset, Track};

/// Build all tracks from audio sources with mixing preset
pub fn build_tracks(
    drums: Vec<f32>,
    bassline: Vec<f32>,
    melody_l: Vec<f32>,
    melody_r: Vec<f32>,
    pads_l: Vec<f32>,
    pads_r: Vec<f32>,
    fx_track: Vec<f32>,
    percussion: Option<Vec<f32>>,
    preset: MixingPreset,
    pad_intensity: &str,
) -> Vec<Track> {
    use super::mixing_presets::get_mixing_preset;

    let (
        drum_vol,
        drum_eq_l,
        drum_eq_m,
        drum_eq_h,
        bass_vol,
        bass_eq_l,
        bass_eq_m,
        bass_eq_h,
        melody_vol,
        melody_eq_l,
        melody_eq_m,
        melody_eq_h,
        _pad_vol_base,
    ) = get_mixing_preset(preset);

    // Pad volume based on intensity
    let pad_vol = match pad_intensity {
        "Subtle" => 0.25,
        "Medium" => 0.40,
        "Prominent" => 0.55,
        _ => 0.35,
    };

    let mut tracks = vec![
        // Drums
        Track::new(drums)
            .with_volume(drum_vol)
            .with_pan(0.0)
            .with_eq(drum_eq_l, drum_eq_m, drum_eq_h),
        // Bass
        Track::new(bassline)
            .with_volume(bass_vol)
            .with_pan(0.0)
            .with_eq(bass_eq_l, bass_eq_m, bass_eq_h),
        // Melody (stereo doubled, double-tracked for authentic width)
        Track::new(melody_l)
            .with_volume(melody_vol)
            .with_pan(-0.20) // Slightly wider
            .with_eq(melody_eq_l, melody_eq_m, melody_eq_h),
        Track::new(melody_r)
            .with_volume(melody_vol)
            .with_pan(0.20) // Slightly wider
            .with_eq(melody_eq_l, melody_eq_m, melody_eq_h),
        // Pads (stereo wide, double-tracked for authentic width)
        Track::new(pads_l)
            .with_volume(pad_vol)
            .with_pan(-0.6) // Wider stereo image
            .with_eq(0.8, 0.9, 0.8),
        Track::new(pads_r)
            .with_volume(pad_vol)
            .with_pan(0.6) // Wider stereo image
            .with_eq(0.8, 0.9, 0.8),
    ];

    // Add percussion track if enabled
    if let Some(perc_track) = percussion {
        tracks.push(
            Track::new(perc_track)
                .with_volume(0.25)
                .with_pan(0.0)
                .with_eq(0.9, 1.0, 1.1),
        );
    }

    // Add transition FX track
    tracks.push(
        Track::new(fx_track)
            .with_volume(0.25) // Reduced from 0.35 to 0.25 for less intrusive FX
            .with_pan(0.0) // Center
            .with_eq(0.7, 1.0, 1.2), // Emphasize highs for sweeps
    );

    tracks
}
