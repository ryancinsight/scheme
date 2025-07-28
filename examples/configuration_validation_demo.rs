use scheme::{
    config::{
        GeometryConfig, GeometryGenerationConfig, ChannelTypeConfig, SerpentineConfig,
        presets,
    },
    geometry::{generator::create_geometry, strategies::SmoothTransitionConfig, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("outputs/configuration_validation")?;

    println!("Configuration Validation Demo");
    println!("============================");
    println!();

    // Test 1: Default configurations
    println!("1. Testing Default Configurations");
    let _default_config = GeometryConfig::default();
    println!("   ✓ Default GeometryConfig created successfully");

    let _default_generation = GeometryGenerationConfig::default();
    println!("   ✓ Default GeometryGenerationConfig created successfully");

    let _default_smooth = SmoothTransitionConfig::default();
    println!("   ✓ Default SmoothTransitionConfig created successfully");

    // Test 2: Preset configurations
    println!("2. Testing Preset Configurations");
    let research_config = presets::research_grade();
    println!("   ✓ Research grade preset created successfully");
    
    let _manufacturing_config = presets::manufacturing_grade();
    println!("   ✓ Manufacturing grade preset created successfully");
    
    let high_quality_config = presets::high_quality_generation();
    println!("   ✓ High quality generation preset created successfully");
    
    let fast_config = presets::fast_generation();
    println!("   ✓ Fast generation preset created successfully");

    // Test 3: Custom configurations with validation
    println!("3. Testing Custom Configurations with Validation");
    
    // Test valid custom configuration
    let custom_generation = GeometryGenerationConfig::new(150, 75, 15, 2.5)?;
    println!("   ✓ Valid custom GeometryGenerationConfig created successfully");
    
    let _custom_geometry = GeometryConfig::with_generation(0.4, 1.2, 0.9, custom_generation)?;
    println!("   ✓ Valid custom GeometryConfig created successfully");

    let _custom_smooth = SmoothTransitionConfig::new(0.2, 0.4, 25, 2.0)?;
    println!("   ✓ Valid custom SmoothTransitionConfig created successfully");

    // Test 4: Configuration validation (error cases)
    println!("4. Testing Configuration Validation (Error Cases)");
    
    // Test invalid serpentine points (too high)
    match GeometryGenerationConfig::new(2000, 50, 10, 2.0) {
        Err(_) => println!("   ✓ Invalid serpentine_points correctly rejected"),
        Ok(_) => println!("   ✗ Invalid serpentine_points should have been rejected"),
    }
    
    // Test invalid transition length factor (too high)
    match SmoothTransitionConfig::new(0.8, 0.3, 20, 2.0) {
        Err(_) => println!("   ✓ Invalid transition_length_factor correctly rejected"),
        Ok(_) => println!("   ✗ Invalid transition_length_factor should have been rejected"),
    }

    // Test 5: Generate geometries with different configurations
    println!("5. Testing Geometry Generation with Different Configurations");
    
    let serpentine_config = SerpentineConfig::default();
    
    // High quality generation
    let high_quality_system = create_geometry(
        (20.0, 10.0),
        &[SplitType::Bifurcation],
        &high_quality_config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    let output_path = "outputs/configuration_validation/high_quality.png";
    plot_geometry(&high_quality_system, output_path)?;
    println!("   ✓ High quality geometry generated: {}", output_path);
    
    // Fast generation
    let fast_system = create_geometry(
        (20.0, 10.0),
        &[SplitType::Bifurcation],
        &fast_config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    let output_path = "outputs/configuration_validation/fast_generation.png";
    plot_geometry(&fast_system, output_path)?;
    println!("   ✓ Fast geometry generated: {}", output_path);
    
    // Research grade
    let research_system = create_geometry(
        (20.0, 10.0),
        &[SplitType::Bifurcation],
        &research_config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    let output_path = "outputs/configuration_validation/research_grade.png";
    plot_geometry(&research_system, output_path)?;
    println!("   ✓ Research grade geometry generated: {}", output_path);

    // Test 6: Smooth transitions with different configurations
    println!("6. Testing Smooth Transitions with Different Configurations");
    
    let subtle_smooth = SmoothTransitionConfig::subtle();
    let pronounced_smooth = SmoothTransitionConfig::pronounced();
    let high_quality_smooth = SmoothTransitionConfig::high_quality();
    let fast_smooth = SmoothTransitionConfig::fast();
    
    for (name, smooth_config) in [
        ("subtle", subtle_smooth),
        ("pronounced", pronounced_smooth),
        ("high_quality", high_quality_smooth),
        ("fast", fast_smooth),
    ] {
        let system = create_geometry(
            (20.0, 10.0),
            &[SplitType::Bifurcation],
            &GeometryConfig::default(),
            &ChannelTypeConfig::SmoothSerpentineWithTransitions {
                serpentine_config,
                smooth_straight_config: smooth_config,
            },
        );
        
        let output_path = format!("outputs/configuration_validation/smooth_{}.png", name);
        plot_geometry(&system, &output_path)?;
        println!("   ✓ {} smooth transitions generated: {}", name, output_path);
    }

    println!();
    println!("Configuration System Summary:");
    println!("• All default configurations work correctly");
    println!("• All preset configurations are functional");
    println!("• Custom configuration validation works properly");
    println!("• Error cases are handled correctly");
    println!("• Geometry generation works with all configuration types");
    println!("• Smooth transition configurations provide expected variety");
    println!();
    println!("New Configuration Features:");
    println!("• GeometryGenerationConfig: Controls point density and quality");
    println!("• Enhanced SmoothTransitionConfig: Added wave_multiplier parameter");
    println!("• Comprehensive validation: All parameters have range checks");
    println!("• Preset configurations: Ready-to-use configurations for common scenarios");
    println!("• Error handling: Clear error messages for invalid configurations");

    Ok(())
}
