//! Comprehensive Arc Channel Collision Prevention Demonstration
//!
//! This example showcases the enhanced arc channel generation system with
//! collision prevention, proximity detection, and adaptive curvature features.
//! It demonstrates how the system prevents channel overlaps while maintaining
//! visual appeal and design principles.

use scheme::config::{GeometryConfig, ChannelTypeConfig, ArcConfig, presets};
use scheme::geometry::{create_geometry, SplitType};
use scheme::visualizations::plot_geometry;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Comprehensive Arc Channel Collision Prevention Demonstration");
    println!("================================================================");

    // Create output directory
    let output_dir = "outputs/arcs/collision_prevention";
    fs::create_dir_all(output_dir)?;

    // Test 1: Maximum curvature with collision prevention OFF vs ON
    println!("\n🔥 Maximum Curvature Comparison:");
    
    // Without collision prevention (dangerous)
    let dangerous_config = ArcConfig {
        curvature_factor: 2.0, // Maximum curvature
        smoothness: 50,
        curvature_direction: 0.0,
        min_separation_distance: 1.0,
        enable_collision_prevention: false, // DISABLED
        max_curvature_reduction: 0.5,
        enable_adaptive_curvature: false,
    };
    
    let geometry_config = GeometryConfig::default();
    let channel_config = ChannelTypeConfig::AllArcs(dangerous_config);
    let splits = vec![SplitType::Bifurcation, SplitType::Bifurcation, SplitType::Trifurcation];
    
    let system = create_geometry((300.0, 200.0), &splits, &geometry_config, &channel_config);
    let output_path = format!("{}/dangerous_maximum_curvature.png", output_dir);
    plot_geometry(&system, &output_path)?;
    println!("   ⚠️  dangerous: Maximum curvature without collision prevention -> {}", output_path);

    // With collision prevention (safe)
    let safe_config = presets::maximum_safe_arcs();
    let channel_config = ChannelTypeConfig::AllArcs(safe_config);
    
    let system = create_geometry((300.0, 200.0), &splits, &geometry_config, &channel_config);
    let output_path = format!("{}/safe_maximum_curvature.png", output_dir);
    plot_geometry(&system, &output_path)?;
    println!("   ✅ safe: Maximum curvature with collision prevention -> {}", output_path);

    // Test 2: Dense layout scenarios
    println!("\n🏗️  Dense Layout Scenarios:");
    
    // Dense layout with many splits
    let dense_splits = vec![
        SplitType::Trifurcation, 
        SplitType::Bifurcation, 
        SplitType::Trifurcation, 
        SplitType::Bifurcation
    ];
    
    // Without adaptive curvature
    let non_adaptive_config = ArcConfig {
        curvature_factor: 1.0,
        smoothness: 30,
        curvature_direction: 0.0,
        min_separation_distance: 1.0,
        enable_collision_prevention: true,
        max_curvature_reduction: 0.5,
        enable_adaptive_curvature: false, // DISABLED
    };
    
    let channel_config = ChannelTypeConfig::AllArcs(non_adaptive_config);
    let system = create_geometry((250.0, 150.0), &dense_splits, &geometry_config, &channel_config);
    let output_path = format!("{}/dense_non_adaptive.png", output_dir);
    plot_geometry(&system, &output_path)?;
    println!("   📊 non_adaptive: Dense layout without adaptive curvature -> {}", output_path);

    // With adaptive curvature
    let adaptive_config = presets::dense_layout_arcs();
    let channel_config = ChannelTypeConfig::AllArcs(adaptive_config);
    
    let system = create_geometry((250.0, 150.0), &dense_splits, &geometry_config, &channel_config);
    let output_path = format!("{}/dense_adaptive.png", output_dir);
    plot_geometry(&system, &output_path)?;
    println!("   🧠 adaptive: Dense layout with adaptive curvature -> {}", output_path);

    // Test 3: Progressive curvature reduction demonstration
    println!("\n📉 Progressive Curvature Reduction:");
    
    let curvature_factors = vec![0.5, 1.0, 1.5, 2.0];
    let reduction_factors = vec![0.8, 0.5, 0.3, 0.1];
    
    for (i, (&curvature, &reduction)) in curvature_factors.iter().zip(reduction_factors.iter()).enumerate() {
        let config = ArcConfig {
            curvature_factor: curvature,
            smoothness: 40,
            curvature_direction: 0.0,
            min_separation_distance: 1.5,
            enable_collision_prevention: true,
            max_curvature_reduction: reduction,
            enable_adaptive_curvature: true,
        };
        
        let channel_config = ChannelTypeConfig::AllArcs(config);
        let test_splits = vec![SplitType::Bifurcation, SplitType::Trifurcation];
        
        let system = create_geometry((200.0, 120.0), &test_splits, &geometry_config, &channel_config);
        let output_path = format!("{}/progressive_reduction_{}.png", output_dir, i + 1);
        plot_geometry(&system, &output_path)?;
        println!("   📈 step_{}: Curvature {:.1}, reduction {:.1} -> {}", i + 1, curvature, reduction, output_path);
    }

    // Test 4: Separation distance effects
    println!("\n📏 Separation Distance Effects:");
    
    let separation_distances = vec![0.5, 1.0, 2.0, 3.0];
    
    for (_i, &separation) in separation_distances.iter().enumerate() {
        let config = ArcConfig {
            curvature_factor: 1.2,
            smoothness: 35,
            curvature_direction: 0.0,
            min_separation_distance: separation,
            enable_collision_prevention: true,
            max_curvature_reduction: 0.4,
            enable_adaptive_curvature: true,
        };
        
        let channel_config = ChannelTypeConfig::AllArcs(config);
        let test_splits = vec![SplitType::Trifurcation, SplitType::Bifurcation];
        
        let system = create_geometry((180.0, 100.0), &test_splits, &geometry_config, &channel_config);
        let output_path = format!("{}/separation_distance_{:.1}.png", output_dir, separation);
        plot_geometry(&system, &output_path)?;
        println!("   📐 separation_{:.1}: Min separation {:.1}mm -> {}", separation, separation, output_path);
    }

    // Test 5: Preset comparison
    println!("\n⚙️  Safety Preset Comparison:");
    
    let presets_to_test = vec![
        ("subtle", presets::subtle_arcs()),
        ("pronounced", presets::pronounced_arcs()),
        ("safe_high", presets::safe_high_curvature_arcs()),
        ("maximum_safe", presets::maximum_safe_arcs()),
        ("dense_layout", presets::dense_layout_arcs()),
    ];
    
    for (name, config) in presets_to_test {
        let channel_config = ChannelTypeConfig::AllArcs(config);
        let test_splits = vec![SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation];
        
        let system = create_geometry((220.0, 130.0), &test_splits, &geometry_config, &channel_config);
        let output_path = format!("{}/preset_{}.png", output_dir, name);
        plot_geometry(&system, &output_path)?;
        println!("   🎛️  {}: Curvature {:.1}, separation {:.1} -> {}", name, config.curvature_factor, config.min_separation_distance, output_path);
    }

    println!("\n📊 Feature Summary:");
    println!("   • Collision Prevention: Automatically reduces curvature when overlaps detected");
    println!("   • Adaptive Curvature: Dynamically adjusts based on neighbor proximity");
    println!("   • Separation Control: Configurable minimum distance between channels");
    println!("   • Safety Presets: Pre-configured settings for different scenarios");
    println!("   • Progressive Reduction: Gradual curvature reduction for optimal balance");
    println!("   • Dense Layout Support: Specialized handling for high-density designs");

    println!("\n✅ Comprehensive collision prevention demonstration complete!");
    println!("   All outputs organized in {}/", output_dir);
    println!("   Compare 'dangerous_maximum_curvature.png' vs 'safe_maximum_curvature.png' to see the difference!");

    Ok(())
}
