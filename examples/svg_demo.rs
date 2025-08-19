use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create organized output directory
    fs::create_dir_all("outputs/svg")?;

    println!("SVG Output Demo - Generating microfluidic schematics in SVG format");
    
    let config = GeometryConfig::default();
    
    // Create a simple bifurcation pattern
    let simple_system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    // Generate SVG output
    plot_geometry(&simple_system, "outputs/svg/simple_bifurcation.svg")?;
    println!("✓ Generated outputs/svg/simple_bifurcation.svg");

    // Create a complex pattern with serpentine channels
    let complex_system = create_geometry(
        (400.0, 200.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(SerpentineConfig::default()),
    );

    // Generate SVG output
    plot_geometry(&complex_system, "outputs/svg/complex_serpentine.svg")?;
    println!("✓ Generated outputs/svg/complex_serpentine.svg");
    
    // Create a mixed pattern with adaptive channel selection
    let adaptive_system = create_geometry(
        (300.0, 150.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::Adaptive {
            serpentine_config: SerpentineConfig::default(),
            arc_config: scheme::config::ArcConfig::default(),
            frustum_config: scheme::config::FrustumConfig::default(),
        },
    );

    // Generate both PNG and SVG for comparison
    plot_geometry(&adaptive_system, "outputs/svg/adaptive_mixed.png")?;
    plot_geometry(&adaptive_system, "outputs/svg/adaptive_mixed.svg")?;
    println!("✓ Generated outputs/svg/adaptive_mixed.png and outputs/svg/adaptive_mixed.svg for comparison");
    
    println!("\nSVG Demo completed successfully!");
    println!("SVG files are vector graphics that can be scaled without quality loss.");
    println!("They can be opened in web browsers or vector graphics editors.");
    
    Ok(())
}
