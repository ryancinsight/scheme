use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::{generator::create_geometry, strategies::SmoothTransitionConfig, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/serpentine").unwrap();

    let config = GeometryConfig::default();

    let serpentine_config = SerpentineConfig {
        fill_factor: 0.9,
        wavelength_factor: 2.0,
        gaussian_width_factor: 8.0,
        wave_density_factor: 3.5,
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    let smooth_transition_config = SmoothTransitionConfig {
        transition_length_factor: 0.2,
        transition_amplitude_factor: 0.3,
        transition_smoothness: 20,
    };

    // Use the new smooth serpentine with transitions configuration
    let channel_config = ChannelTypeConfig::SmoothSerpentineWithTransitions {
        serpentine_config,
        smooth_straight_config: smooth_transition_config,
    };

    let splits = vec![SplitType::Bifurcation];
    let system = create_geometry((20.0, 10.0), &splits, &config, &channel_config);

    let output_path = "outputs/serpentine/bifurcation_single.png";
    plot_geometry(&system, output_path).unwrap();
    println!("Generated serpentine (single bifurcation) with smooth transitions: {}", output_path);
}