use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("outputs/smooth_serpentine")?;

    println!("Smooth Serpentine Transitions Demo");
    println!("=================================");
    println!();

    let config = GeometryConfig::default();

    // Configuration that demonstrates the smooth endpoint transitions
    let smooth_serpentine_config = SerpentineConfig {
        fill_factor: 0.9,
        wavelength_factor: 2.0,
        gaussian_width_factor: 8.0,
        wave_density_factor: 3.5,
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    // 1. Single bifurcation with smooth transitions
    println!("1. Generating Single Bifurcation with Smooth Serpentine Transitions");
    let single_system = create_geometry(
        (20.0, 10.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(smooth_serpentine_config),
    );

    let output_path = "outputs/smooth_serpentine/single_bifurcation_smooth.png";
    plot_geometry(&single_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    // 2. Double bifurcation for more complex demonstration
    println!("2. Generating Double Bifurcation with Smooth Serpentine Transitions");
    let double_system = create_geometry(
        (30.0, 15.0),
        &[SplitType::Bifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(smooth_serpentine_config),
    );

    let output_path = "outputs/smooth_serpentine/double_bifurcation_smooth.png";
    plot_geometry(&double_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    // 3. Trifurcation with smooth transitions
    println!("3. Generating Trifurcation with Smooth Serpentine Transitions");
    let trifurcation_system = create_geometry(
        (25.0, 15.0),
        &[SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(smooth_serpentine_config),
    );

    let output_path = "outputs/smooth_serpentine/trifurcation_smooth.png";
    plot_geometry(&trifurcation_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    // 4. High-density configuration for detailed view
    println!("4. Generating High-Density Serpentine for Detail View");
    let high_density_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 1.5,
        gaussian_width_factor: 6.0,
        wave_density_factor: 5.0, // More waves for detailed view
        wave_phase_direction: 0.0,
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    let high_density_system = create_geometry(
        (40.0, 20.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(high_density_config),
    );

    let output_path = "outputs/smooth_serpentine/high_density_smooth.png";
    plot_geometry(&high_density_system, output_path)?;
    println!("   ✓ Generated: {}", output_path);

    println!();
    println!("Smooth Serpentine Improvements:");
    println!("• Smooth endpoint envelope using smoothstep function: t²(3-2t)");
    println!("• Wave phase aligned to half-periods: sin(π*n*t) ensures zero at endpoints");
    println!("• Combined with improved Gaussian envelope for middle sections");
    println!("• Eliminates sharp transitions and discontinuities");
    println!("• Maintains exact endpoint precision for node connections");
    println!("• C¹ continuity (smooth first derivative) at endpoints");
    println!();
    println!("Mathematical improvements:");
    println!("• Endpoint envelope: f(0) = 0, f(1) = 1, f'(0) = 0, f'(1) = 0");
    println!("• Wave function: sin(π*half_periods*t) naturally zero at t=0,1");
    println!("• Combined envelope: smooth_envelope * gaussian_envelope");
    println!("• Result: Seamless transitions from serpentine to straight sections");

    Ok(())
}
