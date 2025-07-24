use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::{generator::create_geometry, strategies::SmoothTransitionConfig, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("outputs/smooth_transitions")?;

    println!("Smooth Transitions Demo");
    println!("======================");
    println!();

    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.9,
        wavelength_factor: 2.0,
        gaussian_width_factor: 8.0,
        wave_density_factor: 3.5,
        wave_phase_direction: 0.0,
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    let smooth_transition_config = SmoothTransitionConfig {
        transition_length_factor: 0.2, // 20% of channel length for transitions
        transition_amplitude_factor: 0.4, // 40% of channel width for amplitude
        transition_smoothness: 25, // 25 points per transition zone
    };

    // 1. Original serpentine with sharp transitions
    println!("1. Generating Original Serpentine (for comparison)");
    let original_system = create_geometry(
        (20.0, 10.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );

    let output_path = "outputs/smooth_transitions/original_serpentine.png";
    plot_geometry(&original_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    // 2. Smooth serpentine with smooth straight junction connectors
    println!("2. Generating Smooth Serpentine with Transition Zones");
    let smooth_system = create_geometry(
        (20.0, 10.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::SmoothSerpentineWithTransitions {
            serpentine_config,
            smooth_straight_config: smooth_transition_config,
        },
    );

    let output_path = "outputs/smooth_transitions/smooth_serpentine_transitions.png";
    plot_geometry(&smooth_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    // 3. All smooth straight channels
    println!("3. Generating All Smooth Straight Channels");
    let all_smooth_system = create_geometry(
        (20.0, 10.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSmoothStraight(smooth_transition_config),
    );

    let output_path = "outputs/smooth_transitions/all_smooth_straight.png";
    plot_geometry(&all_smooth_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    // 4. Double bifurcation for more complex demonstration
    println!("4. Generating Double Bifurcation with Smooth Transitions");
    let complex_system = create_geometry(
        (30.0, 15.0),
        &[SplitType::Bifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::SmoothSerpentineWithTransitions {
            serpentine_config,
            smooth_straight_config: smooth_transition_config,
        },
    );

    let output_path = "outputs/smooth_transitions/double_bifurcation_smooth.png";
    plot_geometry(&complex_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    println!();
    println!("Smooth Transition Improvements:");
    println!("• SmoothStraight channel type with transition zones");
    println!("• Gradual amplitude changes using smoothstep functions");
    println!("• Configurable transition length and amplitude");
    println!("• Eliminates sharp corners at junction connections");
    println!("• Maintains exact endpoint precision");
    println!("• Compatible with existing serpentine channels");
    println!();
    println!("Configuration options:");
    println!("• transition_length_factor: Controls length of transition zones");
    println!("• transition_amplitude_factor: Controls amplitude of transition waves");
    println!("• transition_smoothness: Controls number of points in transitions");
    println!("• SmoothSerpentineWithTransitions: Mixed configuration for optimal results");

    Ok(())
}
