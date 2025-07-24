use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    plot_geometry(&simple_system, "outputs/simple_bifurcation.svg")?;
    println!("✓ Generated simple_bifurcation.svg");
    
    // Create a complex pattern with serpentine channels
    let complex_system = create_geometry(
        (400.0, 200.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(SerpentineConfig::default()),
    );
    
    // Generate SVG output
    plot_geometry(&complex_system, "outputs/complex_serpentine.svg")?;
    println!("✓ Generated complex_serpentine.svg");
    
    // Create a mixed pattern with smart channel selection
    let smart_system = create_geometry(
        (300.0, 150.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::Smart {
            serpentine_config: SerpentineConfig::default(),
            arc_config: scheme::config::ArcConfig::default(),
        },
    );
    
    // Generate both PNG and SVG for comparison
    plot_geometry(&smart_system, "outputs/smart_mixed.png")?;
    plot_geometry(&smart_system, "outputs/smart_mixed.svg")?;
    println!("✓ Generated smart_mixed.png and smart_mixed.svg for comparison");
    
    println!("\nSVG Demo completed successfully!");
    println!("SVG files are vector graphics that can be scaled without quality loss.");
    println!("They can be opened in web browsers or vector graphics editors.");
    
    Ok(())
}
