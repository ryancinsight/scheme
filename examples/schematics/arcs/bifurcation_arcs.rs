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
    };

    let arc_config = ArcConfig {
        curvature_factor: 0.4,  // Moderate curvature for natural appearance
        smoothness: 25,         // Smooth curves
    };

    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(arc_config),
    );

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("outputs/arcs")?;

    // Plot and save the diagram
    plot_geometry(&system, "outputs/arcs/bifurcation_arcs.png")?;
    
    println!("Generated bifurcation with arc channels: outputs/arcs/bifurcation_arcs.png");
    println!("Number of channels: {}", system.channels.len());
    println!("Number of nodes: {}", system.nodes.len());
    
    Ok(())
} 