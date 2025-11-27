/// Audio effects and transitions for production polish
use crate::utils::get_sample_rate;
use rand::Rng;

/// Generate a heavy drop kick for breakdowns
/// This is an extended, aggressive kick drum with massive low-end
pub fn generate_drop_kick() -> Vec<f32> {
    let duration = 2.0; // Longer than a normal kick for dramatic effect
    let num_samples = (duration * get_sample_rate() as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut rng = rand::thread_rng();

    for i in 0..num_samples {
        let time = i as f32 / get_sample_rate() as f32;
        
        // MASSIVE pitch drop: 200Hz -> 35Hz (sub-bass territory)
        let pitch_drop = (-time * 8.0).exp();
        let pitch = 35.0 + (165.0 * pitch_drop);
        
        // Extended amplitude envelope (longer sustain for the drop)
        let amp_env = (-time * 3.5).exp();
        
        // Body: Sine + Triangle blend for maximum weight
        let phase = 2.0 * std::f32::consts::PI * pitch * time;
        let body = (phase.sin() * 0.8 + (phase * 0.5).sin().signum() * 0.2) * amp_env;
        
        // AGGRESSIVE CLICK: Sharp beater attack
        let click_env = (-time * 200.0).exp();
        let click = rng.gen_range(-1.0..1.0) * 1.5 * click_env;
        
        // Mix: Heavy body + sharp click
        let mut sample = body + click * 0.35;
        
        // HARD saturation for that "wall of sound" compression
        sample = (sample * 4.0).tanh();
        
        samples.push(sample * 0.9);
    }

    samples
}
