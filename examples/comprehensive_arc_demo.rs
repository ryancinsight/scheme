//! Comprehensive Arc Channel Demo
//! 
//! This example demonstrates all aspects of arc channel generation:
//! - Different curvature factors
//! - Smoothness levels
//! - Directional control
//! - Integration with split patterns
//! - Smart mixed channel configurations

use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, ArcConfig, presets},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create organized output directories
    fs::create_dir_all("outputs/arcs/curvature")?;
    fs::create_dir_all("outputs/arcs/smoothness")?;
    fs::create_dir_all("outputs/arcs/directions")?;
    fs::create_dir_all("outputs/mixed/smart_selection")?;

    let config = GeometryConfig::default();
    let splits = vec![SplitType::Bifurcation, SplitType::Trifurcation];
    let box_dims = (300.0, 150.0);

    println!("🌈 Comprehensive Arc Channel Demonstration");
    println!("========================================");

    // 1. Curvature Factor Variations
    println!("\n📐 Curvature Factor Variations:");
    
    let curvature_factors = vec![
        (0.1, "subtle", "Very subtle curves"),
        (0.5, "moderate", "Moderate curvature"),
        (1.0, "pronounced", "Pronounced curves"),
        (1.5, "strong", "Strong curvature"),
        (2.0, "maximum", "Maximum curvature"),
    ];

    for (factor, name, description) in curvature_factors {
        let arc_config = ArcConfig {
            curvature_factor: factor,
            smoothness: 50,
            curvature_direction: 0.0, // Auto-determine
            min_separation_distance: 1.0,
            enable_collision_prevention: true,
            max_curvature_reduction: 0.5,
            enable_adaptive_curvature: true,
        };
        
        let system = create_geometry(box_dims, &splits, &config, &ChannelTypeConfig::AllArcs(arc_config));
        let output = format!("outputs/arcs/curvature/{}_curvature.png", name);
        plot_geometry(&system, &output)?;
        println!("   ✓ {}: {} (factor: {}) -> {}", name, description, factor, output);
    }

    // 2. Smoothness Levels
    println!("\n✨ Smoothness Levels:");
    
    let smoothness_levels = vec![
        (10, "low", "Low resolution (10 points)"),
        (25, "medium", "Medium resolution (25 points)"),
        (50, "high", "High resolution (50 points)"),
        (100, "ultra", "Ultra-high resolution (100 points)"),
    ];

    for (smoothness, name, description) in smoothness_levels {
        let arc_config = ArcConfig {
            curvature_factor: 1.0,
            smoothness,
            curvature_direction: 0.0,
            min_separation_distance: 1.0,
            enable_collision_prevention: true,
            max_curvature_reduction: 0.5,
            enable_adaptive_curvature: true,
        };
        
        let system = create_geometry(box_dims, &splits, &config, &ChannelTypeConfig::AllArcs(arc_config));
        let output = format!("outputs/arcs/smoothness/{}_smoothness.png", name);
        plot_geometry(&system, &output)?;
        println!("   ✓ {}: {} -> {}", name, description, output);
    }

    // 3. Directional Control
    println!("\n🧭 Directional Control:");
    
    let direction_configs = vec![
        ("auto", 0.0, "Auto-determine direction"),
        ("inward", -1.0, "Force inward curvature"),
        ("outward", 1.0, "Force outward curvature"),
    ];

    for (name, direction, description) in direction_configs {
        let arc_config = ArcConfig {
            curvature_factor: 1.0,
            smoothness: 50,
            curvature_direction: direction,
            min_separation_distance: 1.0,
            enable_collision_prevention: true,
            max_curvature_reduction: 0.5,
            enable_adaptive_curvature: true,
        };
        
        let system = create_geometry(box_dims, &splits, &config, &ChannelTypeConfig::AllArcs(arc_config));
        let output = format!("outputs/arcs/directions/{}_direction.png", name);
        plot_geometry(&system, &output)?;
        println!("   ✓ {}: {} -> {}", name, description, output);
    }

    // 4. Configuration Presets
    println!("\n⚙️  Configuration Presets:");
    
    let preset_configs = vec![
        ("default", ArcConfig::default(), "Standard arc configuration"),
        ("subtle", presets::subtle_arcs(), "Subtle, gentle curves"),
        ("pronounced", presets::pronounced_arcs(), "Pronounced, dramatic curves"),
    ];

    for (name, preset_config, description) in preset_configs {
        let system = create_geometry(box_dims, &splits, &config, &ChannelTypeConfig::AllArcs(preset_config));
        let output = format!("outputs/arcs/curvature/{}_preset.png", name);
        plot_geometry(&system, &output)?;
        println!("   ✓ {}: {} -> {}", name, description, output);
    }

    // 5. Smart Mixed Channel Selection
    println!("\n🧠 Smart Mixed Channel Selection:");
    
    let adaptive_configs = vec![
        ("adaptive_default", ChannelTypeConfig::Adaptive {
            serpentine_config: scheme::config::SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
            frustum_config: scheme::config::FrustumConfig::default(),
        }, "Default adaptive selection"),
        ("adaptive_pronounced", ChannelTypeConfig::Adaptive {
            serpentine_config: presets::smooth_serpentine(),
            arc_config: presets::pronounced_arcs(),
            frustum_config: scheme::config::FrustumConfig::default(),
        }, "Pronounced curves with smooth serpentines"),
        ("adaptive_mixed_position", ChannelTypeConfig::MixedByPosition {
            middle_zone_fraction: 0.4,
            serpentine_config: scheme::config::SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
        }, "Position-based channel selection"),
    ];

    for (name, adaptive_config, description) in adaptive_configs {
        let system = create_geometry(box_dims, &splits, &config, &adaptive_config);
        let output = format!("outputs/mixed/adaptive_selection/{}.png", name);
        plot_geometry(&system, &output)?;
        println!("   ✓ {}: {} -> {}", name, description, output);
    }

    // 6. Single Arc Demonstration
    println!("\n🔗 Single Arc Examples:");
    
    let single_configs = vec![
        ("basic", ArcConfig::default()),
        ("subtle", presets::subtle_arcs()),
        ("pronounced", presets::pronounced_arcs()),
        ("high_smoothness", ArcConfig {
            curvature_factor: 1.0,
            smoothness: 100,
            curvature_direction: 0.0,
            min_separation_distance: 1.0,
            enable_collision_prevention: true,
            max_curvature_reduction: 0.5,
            enable_adaptive_curvature: true,
        }),
    ];

    for (name, single_config) in single_configs {
        let system = create_geometry((200.0, 100.0), &[], &config, &ChannelTypeConfig::AllArcs(single_config));
        let output = format!("outputs/arcs/curvature/single_{}.png", name);
        plot_geometry(&system, &output)?;
        println!("   ✓ Single arc {}: -> {}", name, output);
    }

    // Summary
    println!("\n📊 Feature Summary:");
    println!("   • Curvature Control: From subtle (0.1) to maximum (2.0) curvature factors");
    println!("   • Smoothness Options: 10-100+ points for different resolution needs");
    println!("   • Directional Control: Auto, inward, and outward curvature directions");
    println!("   • Smart Selection: Automatic channel type selection based on geometry");
    println!("   • Mixed Configurations: Combine arcs and serpentines intelligently");
    println!("   • Performance Optimized: Efficient generation for all smoothness levels");

    println!("\n✅ Comprehensive arc demonstration complete!");
    println!("   All outputs organized in outputs/arcs/ and outputs/mixed/ subdirectories");
    
    Ok(())
}
