use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, ArcConfig},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GeometryConfig {
        wall_clearance: 3.0,
        channel_width: 2.5,
        channel_height: 2.5,
    };

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("outputs/arcs")?;

    // Generate examples with different curvature factors
    let curvature_factors = vec![0.1, 0.3, 0.5, 0.7];
    
    for (i, &curvature) in curvature_factors.iter().enumerate() {
        let arc_config = ArcConfig {
            curvature_factor: curvature,
            smoothness: 20,
        };

        let system = create_geometry(
            (180.0, 80.0),
            &[SplitType::Bifurcation],
            &config,
            &ChannelTypeConfig::AllArcs(arc_config),
        );

        let filename = format!("outputs/arcs/curvature_{}.png", i + 1);
        plot_geometry(&system, &filename)?;
        
        println!("Generated curvature factor {}: {}", curvature, filename);
    }
    
    println!("\nComparison of different arc curvatures:");
    println!("  curvature_1.png: factor 0.1 (subtle)");
    println!("  curvature_2.png: factor 0.3 (moderate)");
    println!("  curvature_3.png: factor 0.5 (pronounced)");
    println!("  curvature_4.png: factor 0.7 (high)");
    
    Ok(())
} 