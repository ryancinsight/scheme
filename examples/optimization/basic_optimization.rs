use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig, presets},
    geometry::{generator::create_geometry, SplitType, optimization::calculate_path_length},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/optimization").unwrap();

    let config = GeometryConfig::default();

    println!("Basic Serpentine Channel Optimization Demo");
    println!("==========================================");
    println!();

    // Create a simple bifurcation system
    let splits = vec![SplitType::Bifurcation];
    let box_dims = (200.0, 100.0);

    // 1. Standard serpentine configuration (no optimization)
    println!("1. Standard Serpentine Configuration");
    let standard_config = SerpentineConfig::default(); // optimization_enabled = false by default
    
    let standard_system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(standard_config),
    );

    let standard_length = calculate_total_length(&standard_system);
    println!("   Total channel length: {:.2} mm", standard_length);
    
    plot_geometry(&standard_system, "outputs/optimization/basic_standard.png").unwrap();
    println!("   Generated: outputs/optimization/basic_standard.png");
    println!();

    // 2. Optimized serpentine configuration
    println!("2. Optimized Serpentine Configuration");
    let optimized_config = presets::optimized_serpentine(); // Uses optimization_enabled = true
    
    let optimized_system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(optimized_config),
    );

    let optimized_length = calculate_total_length(&optimized_system);
    println!("   Total channel length: {:.2} mm", optimized_length);
    
    plot_geometry(&optimized_system, "outputs/optimization/basic_optimized.png").unwrap();
    println!("   Generated: outputs/optimization/basic_optimized.png");
    println!();

    // 3. Calculate and display improvement
    let improvement = if standard_length > 0.0 {
        ((optimized_length - standard_length) / standard_length) * 100.0
    } else {
        0.0
    };

    println!("3. Optimization Results");
    println!("   Standard length:    {:.2} mm", standard_length);
    println!("   Optimized length:   {:.2} mm", optimized_length);
    println!("   Length improvement: {:.1}%", improvement);
    println!();

    // 4. Custom optimization configuration
    println!("4. Custom Optimization Configuration");
    let custom_config = SerpentineConfig {
        fill_factor: 0.9,
        wavelength_factor: 2.5,
        gaussian_width_factor: 8.0,
        wave_density_factor: 3.0,
        wave_phase_direction: 0.0,
        optimization_enabled: true,
        target_fill_ratio: 0.98, // Very aggressive optimization
        optimization_profile: scheme::config::OptimizationProfile::Thorough,
    };

    let custom_system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(custom_config),
    );

    let custom_length = calculate_total_length(&custom_system);
    let custom_improvement = if standard_length > 0.0 {
        ((custom_length - standard_length) / standard_length) * 100.0
    } else {
        0.0
    };

    println!("   Custom optimized length: {:.2} mm", custom_length);
    println!("   Custom improvement:      {:.1}%", custom_improvement);
    
    plot_geometry(&custom_system, "outputs/optimization/basic_custom.png").unwrap();
    println!("   Generated: outputs/optimization/basic_custom.png");
    println!();

    // 5. Show configuration details
    println!("5. Configuration Details");
    println!("   Standard Config:");
    println!("     optimization_enabled: {}", standard_config.optimization_enabled);
    println!("     target_fill_ratio:    {:.2}", standard_config.target_fill_ratio);
    println!("     fill_factor:          {:.2}", standard_config.fill_factor);
    println!("     wavelength_factor:    {:.2}", standard_config.wavelength_factor);
    println!();
    
    println!("   Optimized Config:");
    println!("     optimization_enabled: {}", optimized_config.optimization_enabled);
    println!("     target_fill_ratio:    {:.2}", optimized_config.target_fill_ratio);
    println!("     fill_factor:          {:.2}", optimized_config.fill_factor);
    println!("     wavelength_factor:    {:.2}", optimized_config.wavelength_factor);
    println!();

    println!("   Custom Config:");
    println!("     optimization_enabled: {}", custom_config.optimization_enabled);
    println!("     target_fill_ratio:    {:.2}", custom_config.target_fill_ratio);
    println!("     fill_factor:          {:.2}", custom_config.fill_factor);
    println!("     wavelength_factor:    {:.2}", custom_config.wavelength_factor);
    println!();

    println!("Demo complete! Check the generated PNG files to see the visual differences.");
}

/// Calculate total length of all serpentine channels in a system
fn calculate_total_length(system: &scheme::geometry::ChannelSystem) -> f64 {
    system.channels.iter()
        .map(|channel| {
            match &channel.channel_type {
                scheme::geometry::ChannelType::Serpentine { path } => {
                    calculate_path_length(path)
                },
                _ => 0.0,
            }
        })
        .sum()
}
