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
        curvature_factor: 0.5,  // Strong curvature to show direction clearly
        smoothness: 25,         // Smooth curves for clear visualization
    };

    // Create a bifurcation followed by trifurcation to show directional flow
    let system = create_geometry(
        (250.0, 120.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(arc_config),
    );

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("outputs/arcs")?;

    // Plot and save the diagram
    plot_geometry(&system, "outputs/arcs/directional_flow_arcs.png")?;
    
    println!("Generated directional flow arcs: outputs/arcs/directional_flow_arcs.png");
    println!("Number of channels: {}", system.channels.len());
    println!("Number of nodes: {}", system.nodes.len());
    
    // Count different channel types
    let mut arc_count = 0;
    
    for channel in &system.channels {
        match &channel.channel_type {
            scheme::geometry::ChannelType::Arc { .. } => arc_count += 1,
            _ => {},
        }
    }
    
    println!("Arc channels: {}", arc_count);
    println!("\nFlow pattern:");
    println!("  • First half (left): Channels diverge outward from splits");
    println!("  • Second half (right): Channels converge inward toward merges");
    println!("  • Upper channels curve upward when diverging, downward when converging");
    println!("  • Lower channels curve downward when diverging, upward when converging");
    
    Ok(())
} 