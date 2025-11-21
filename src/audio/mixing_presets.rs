/// Mixing presets for different styles
#[derive(Debug, Clone, Copy)]
pub enum MixingPreset {
    Clean,
    Warm,
    Punchy,
    Spacious,
}

/// Get mixing parameters for a preset
/// Returns: (drum_vol, drum_eq_l, drum_eq_m, drum_eq_h, bass_vol, bass_eq_l, bass_eq_m, bass_eq_h, melody_vol, melody_eq_l, melody_eq_m, melody_eq_h, pad_vol)
pub fn get_mixing_preset(
    preset: MixingPreset,
) -> (
    f32,
    f32,
    f32,
    f32, // Drum: vol, eq_l, eq_m, eq_h
    f32,
    f32,
    f32,
    f32, // Bass: vol, eq_l, eq_m, eq_h
    f32,
    f32,
    f32,
    f32, // Melody: vol, eq_l, eq_m, eq_h
    f32, // Pad: vol
) {
    match preset {
        MixingPreset::Clean => (
            // Drums
            1.1, 1.2, 1.0, 1.0, // Bass
            0.65, 1.4, 0.9, 0.4, // Melody
            0.40, 0.85, 1.1, 1.05, // Pads
            0.35,
        ),
        MixingPreset::Warm => (
            // Drums
            1.0, 1.5, 0.9, 0.7, // Bass
            0.75, 1.6, 0.85, 0.3, // Melody
            0.36, 0.95, 1.0, 0.9, // Pads
            0.40,
        ),
        MixingPreset::Punchy => (
            // Drums
            1.3, 1.4, 1.1, 0.9, // Bass
            0.70, 1.5, 0.95, 0.4, // Melody
            0.40, 0.9, 1.15, 1.0, // Pads
            0.45,
        ),
        MixingPreset::Spacious => (
            // Drums
            1.0, 1.1, 0.95, 1.1, // Bass
            0.60, 1.3, 0.85, 0.5, // Melody
            0.35, 0.8, 1.0, 1.15, // Pads
            0.50,
        ),
    }
}

/// Select mixing preset from string (for compatibility)
pub fn preset_from_str(s: &str) -> MixingPreset {
    match s {
        "Clean" => MixingPreset::Clean,
        "Warm" => MixingPreset::Warm,
        "Punchy" => MixingPreset::Punchy,
        "Spacious" => MixingPreset::Spacious,
        _ => MixingPreset::Clean,
    }
}
