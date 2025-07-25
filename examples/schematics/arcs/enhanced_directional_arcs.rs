use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, ArcConfig},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating enhanced directional flow arcs demonstration...");

    // Enhanced arc configuration for demonstration
    let arc_config = ArcConfig {
        curvature_factor: 0.8, // Higher curvature to show enhanced flow patterns
        smoothness: 25,        // Smooth curves for better visualization
        curvature_direction: 0.0, // Auto-determine for symmetric appearance
    };

    let config = GeometryConfig {
        wall_clearance: 15.0,
        channel_width: 4.0,
        channel_height: 1.0,
        generation: scheme::config::GeometryGenerationConfig::default(),
    };

    // Complex multi-level branching to test enhanced logic
    let splits = vec![
        SplitType::Bifurcation,   // First bifurcation  
        SplitType::Trifurcation,  // Then trifurcation
        SplitType::Bifurcation,   // Another bifurcation
        SplitType::Trifurcation,  // Final trifurcation
    ];

    let channel_type_config = ChannelTypeConfig::AllArcs(arc_config);

    let system = create_geometry(
        (400.0, 200.0),
        &splits,
        &config,
        &channel_type_config,
    );

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("outputs/arcs")?;

    let output_path = "outputs/arcs/enhanced_directional_arcs.png";
    plot_geometry(&system, output_path)?;

    println!("Enhanced directional flow arcs saved to: {}", output_path);
    println!("Number of channels: {}", system.channels.len());
    println!("Number of nodes: {}", system.nodes.len());

    // Count arc channels
    let arc_count = system.channels.iter()
        .filter(|c| matches!(c.channel_type, scheme::geometry::ChannelType::Arc { .. }))
        .count();
    println!("Arc channels: {}", arc_count);

    println!("\nEnhanced Flow Features:");
    println!("  • Junction-aware curvature: Reduced curvature near junctions for smooth transitions");
    println!("  • Enhanced flow phase detection: Better diverging/converging determination");
    println!("  • Local position analysis: Improved center/upper/lower channel detection");
    println!("  • Multi-level branching support: Context-aware handling of complex networks");
    println!("  • Reduced curvature for horizontal channels while maintaining flow naturalness");
    println!("  • Enhanced curvature for junction connectors with significant vertical displacement");

    println!("\nFlow Pattern Improvements:");
    println!("  • First ~35% (left): Enhanced diverging flow - channels spread outward naturally");
    println!("  • Middle ~30%: Context-sensitive flow handling");  
    println!("  • Last ~35% (right): Enhanced converging flow - channels merge smoothly");
    println!("  • Junction transitions: Smoother curvature near inlet/outlet junctions");
    println!("  • Complex branching: Better handling of multi-level split patterns");

    Ok(())
} 