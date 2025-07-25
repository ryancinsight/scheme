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

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("outputs/arcs")?;

    // Test 1: Auto-determined curvature (all inward)
    let auto_arc_config = ArcConfig {
        curvature_factor: 0.5,
        smoothness: 25,
        curvature_direction: 0.0, // Auto-determine (should be all inward)
    };

    let auto_system = create_geometry(
        (250.0, 120.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(auto_arc_config),
    );

    plot_geometry(&auto_system, "outputs/arcs/curvature_auto.png")?;
    println!("Generated auto-curvature arcs: outputs/arcs/curvature_auto.png");

    // Test 2: Forced inward curvature
    let inward_arc_config = ArcConfig {
        curvature_factor: 0.5,
        smoothness: 25,
        curvature_direction: -1.0, // Force all arcs to curve inward
    };

    let inward_system = create_geometry(
        (250.0, 120.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(inward_arc_config),
    );

    plot_geometry(&inward_system, "outputs/arcs/curvature_inward.png")?;
    println!("Generated inward-curvature arcs: outputs/arcs/curvature_inward.png");

    // Test 3: Forced outward curvature
    let outward_arc_config = ArcConfig {
        curvature_factor: 0.5,
        smoothness: 25,
        curvature_direction: 1.0, // Force all arcs to curve outward
    };

    let outward_system = create_geometry(
        (250.0, 120.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(outward_arc_config),
    );

    plot_geometry(&outward_system, "outputs/arcs/curvature_outward.png")?;
    println!("Generated outward-curvature arcs: outputs/arcs/curvature_outward.png");

    // Count arc channels in each system
    let count_arcs = |system: &scheme::geometry::ChannelSystem| {
        system.channels.iter().filter(|ch| matches!(ch.channel_type, scheme::geometry::ChannelType::Arc { .. })).count()
    };

    println!("\nArc channel counts:");
    println!("  Auto: {}", count_arcs(&auto_system));
    println!("  Inward: {}", count_arcs(&inward_system));
    println!("  Outward: {}", count_arcs(&outward_system));

    Ok(())
}
