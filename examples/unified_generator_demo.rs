use scheme::{
    geometry::{
        generator::{create_geometry, create_geometry_with_metadata, MetadataConfig},
        builders::ChannelExt,
        metadata::PerformanceMetadata,
        SplitType,
    },
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("outputs/unified")?;

    println!("Unified Generator Demo");
    println!("====================");
    println!();

    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig::default();

    // 1. Standard generation (no metadata)
    println!("1. Standard Generation (No Metadata)");
    let start_time = std::time::Instant::now();
    let standard_system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    let standard_time = start_time.elapsed();
    
    println!("   Generated {} channels and {} nodes in {:?}", 
        standard_system.channels.len(), 
        standard_system.nodes.len(),
        standard_time
    );
    
    // Verify no metadata
    let has_metadata = standard_system.channels[0].has_metadata::<PerformanceMetadata>();
    println!("   Has performance metadata: {}", has_metadata);
    println!();

    // 2. Generation with metadata
    println!("2. Generation with Metadata");
    let metadata_config = MetadataConfig {
        track_performance: true,
        track_optimization: true,
    };
    
    let start_time = std::time::Instant::now();
    let metadata_system = create_geometry_with_metadata(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
        &metadata_config,
    );
    let metadata_time = start_time.elapsed();
    
    println!("   Generated {} channels and {} nodes in {:?}", 
        metadata_system.channels.len(), 
        metadata_system.nodes.len(),
        metadata_time
    );
    
    // Verify metadata exists
    let has_perf_metadata = metadata_system.channels[0].has_metadata::<PerformanceMetadata>();
    println!("   Has performance metadata: {}", has_perf_metadata);
    
    if has_perf_metadata {
        let perf_data = metadata_system.channels[0].get_metadata::<PerformanceMetadata>().unwrap();
        println!("   Generation time: {}μs, Memory: {} bytes, Path points: {}", 
            perf_data.generation_time_us,
            perf_data.memory_usage_bytes,
            perf_data.path_points_count
        );
    }
    println!();

    // 3. Performance comparison
    println!("3. Performance Comparison");
    println!("   Standard generation: {:?}", standard_time);
    println!("   Metadata generation: {:?}", metadata_time);
    let overhead = metadata_time.as_nanos() as f64 / standard_time.as_nanos() as f64;
    println!("   Metadata overhead: {:.1}x", overhead);
    println!();

    // 4. Generate visualizations
    println!("4. Generating Visualizations");
    plot_geometry(&standard_system, "outputs/unified/standard_system.png")?;
    println!("   ✓ Standard system: outputs/unified/standard_system.png");
    
    plot_geometry(&metadata_system, "outputs/unified/metadata_system.png")?;
    println!("   ✓ Metadata system: outputs/unified/metadata_system.png");
    println!();

    // 5. API Summary
    println!("5. Unified Generator API Summary");
    println!("   • create_geometry() - Fast generation without metadata");
    println!("   • create_geometry_with_metadata() - Generation with optional metadata tracking");
    println!("   • MetadataConfig - Configure what metadata to track");
    println!("   • Zero overhead when metadata is disabled");
    println!("   • Full backward compatibility maintained");
    println!();

    println!("Demo complete! The unified generator provides:");
    println!("• Single source of truth for geometry generation");
    println!("• Optional metadata support with zero overhead when unused");
    println!("• Clean, consistent API");
    println!("• Full backward compatibility");

    Ok(())
}
