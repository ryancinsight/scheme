use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, ArcConfig},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GeometryConfig {
        wall_clearance: 4.0,
        channel_width: 3.0,
        channel_height: 3.0,
        generation: scheme::config::GeometryGenerationConfig::default(),
    };

    let arc_config = ArcConfig {
        curvature_factor: 0.3,  // Subtle curvature
        smoothness: 30,         // Very smooth curves
        curvature_direction: 0.0, // Auto-determine for symmetric appearance
    };

    let system = create_geometry(
        (250.0, 120.0),
        &[SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(arc_config),
    );

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("outputs/arcs")?;

    // Plot and save the diagram
    plot_geometry(&system, "outputs/arcs/trifurcation_arcs.png")?;
    
    println!("Generated trifurcation with arc channels: outputs/arcs/trifurcation_arcs.png");
    println!("Number of channels: {}", system.channels.len());
    println!("Number of nodes: {}", system.nodes.len());
    
    Ok(())
} 