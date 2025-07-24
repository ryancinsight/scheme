use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("outputs/serpentine")?;

    let config = GeometryConfig::default();

    // Create a configuration that will show the improved Gaussian envelope
    let improved_serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 4.0, // Smaller factor for more pronounced effect
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    // Test with a bifurcation pattern to show both directional changes and middle sections
    let splits = vec![SplitType::Bifurcation, SplitType::Bifurcation];
    let system = create_geometry(
        (400.0, 200.0), // Larger dimensions to better show the effect
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(improved_serpentine_config),
    );

    let output_path = "outputs/serpentine/improved_gaussian_demo.png";
    plot_geometry(&system, output_path)?;
    println!("Generated improved Gaussian envelope demo: {}", output_path);
    
    // Also create a comparison with the old behavior using a high gaussian_width_factor
    let old_style_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 10.0, // Higher factor simulates old uniform behavior
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    let old_system = create_geometry(
        (400.0, 200.0),
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(old_style_config),
    );

    let old_output_path = "outputs/serpentine/old_gaussian_comparison.png";
    plot_geometry(&old_system, old_output_path)?;
    println!("Generated old-style Gaussian comparison: {}", old_output_path);

    println!("\nImproved Gaussian Envelope Features:");
    println!("- Distance-based normalization: shorter channels have more aggressive tapering");
    println!("- Middle section detection: horizontal channels maintain more amplitude in center");
    println!("- Plateau effect: middle sections have a flat region for full amplitude");
    println!("- Directional change handling: nodes with direction changes get full Gaussian tapering");

    Ok(())
}
