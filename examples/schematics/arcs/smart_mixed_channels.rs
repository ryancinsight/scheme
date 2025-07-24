use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GeometryConfig {
        wall_clearance: 4.0,
        channel_width: 3.0,
        channel_height: 3.0,
    };

    let serpentine_config = SerpentineConfig {
        fill_factor: 0.7,
        wavelength_factor: 2.5,
        gaussian_width_factor: 8.0,
        wave_density_factor: 3.0,
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    let arc_config = ArcConfig {
        curvature_factor: 0.35,
        smoothness: 20,
        curvature_direction: 0.0, // Auto-determine for symmetric appearance
    };

    let system = create_geometry(
        (300.0, 150.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::Smart {
            serpentine_config,
            arc_config,
        },
    );

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("outputs/arcs")?;

    // Plot and save the diagram
    plot_geometry(&system, "outputs/arcs/smart_mixed_channels.png")?;
    
    println!("Generated smart mixed channel design: outputs/arcs/smart_mixed_channels.png");
    println!("Number of channels: {}", system.channels.len());
    println!("Number of nodes: {}", system.nodes.len());
    
    // Count different channel types
    let mut straight_count = 0;
    let mut serpentine_count = 0;
    let mut arc_count = 0;
    
    for channel in &system.channels {
        match &channel.channel_type {
            scheme::geometry::ChannelType::Straight => straight_count += 1,
            scheme::geometry::ChannelType::Serpentine { .. } => serpentine_count += 1,
            scheme::geometry::ChannelType::Arc { .. } => arc_count += 1,
        }
    }
    
    println!("Channel type distribution:");
    println!("  Straight: {}", straight_count);
    println!("  Serpentine: {}", serpentine_count);
    println!("  Arc: {}", arc_count);
    
    Ok(())
} 