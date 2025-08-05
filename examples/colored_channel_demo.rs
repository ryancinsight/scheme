//! Colored Channel Type Demonstration
//!
//! This example demonstrates the new colored visualization system where
//! different channel types are rendered in different colors:
//! - Straight channels: Black
//! - Curved channels (Serpentine, Arc): Blue  
//! - Tapered channels (Frustum): Red
//!
//! Run with: cargo run --example colored_channel_demo

use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig, FrustumConfig},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Colored Channel Type Demonstration");
    println!("====================================");
    println!();
    println!("This demo shows different channel types in different colors:");
    println!("  🖤 Straight channels: Black");
    println!("  🔵 Curved channels (Serpentine, Arc): Blue");
    println!("  🔴 Tapered channels (Frustum): Red");
    println!();

    // Ensure output directory exists
    fs::create_dir_all("outputs")?;

    // Create systems with different channel types
    demonstrate_all_straight()?;
    demonstrate_all_serpentine()?;
    demonstrate_all_frustum()?;
    demonstrate_mixed_smart_system()?;
    demonstrate_custom_colors()?;

    println!("✅ All colored demonstrations completed successfully!");
    println!();
    println!("📁 Output files saved to 'outputs/' directory:");
    println!("   • colored_all_straight.svg (black channels)");
    println!("   • colored_all_serpentine.svg (blue channels)");
    println!("   • colored_all_frustum.svg (red channels)");
    println!("   • colored_mixed_smart.svg (mixed colors)");
    println!("   • colored_custom_colors.svg (custom color scheme)");
    
    Ok(())
}

/// Demonstrate all straight channels (black)
fn demonstrate_all_straight() -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣  All Straight Channels (Black)");
    
    let system = create_geometry(
        (120.0, 60.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );
    
    plot_geometry(&system, "outputs/colored_all_straight.svg")?;
    println!("   ✅ Straight channels: saved to colored_all_straight.svg");
    println!("   📊 {} straight channels (all black)", system.channels.len());
    println!();
    Ok(())
}

/// Demonstrate all serpentine channels (blue)
fn demonstrate_all_serpentine() -> Result<(), Box<dyn std::error::Error>> {
    println!("2️⃣  All Serpentine Channels (Blue)");
    
    let serpentine_config = SerpentineConfig::default();
    
    let system = create_geometry(
        (120.0, 60.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    plot_geometry(&system, "outputs/colored_all_serpentine.svg")?;
    println!("   ✅ Serpentine channels: saved to colored_all_serpentine.svg");
    println!("   📊 {} serpentine channels (all blue)", system.channels.len());
    println!();
    Ok(())
}

/// Demonstrate all frustum channels (red)
fn demonstrate_all_frustum() -> Result<(), Box<dyn std::error::Error>> {
    println!("3️⃣  All Frustum Channels (Red)");
    
    let frustum_config = FrustumConfig {
        inlet_width: 2.5,
        throat_width: 0.6,
        outlet_width: 2.0,
        taper_profile: scheme::config::TaperProfile::Smooth,
        smoothness: 60,
        throat_position: 0.4,
    };
    
    let system = create_geometry(
        (120.0, 60.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllFrustum(frustum_config),
    );
    
    plot_geometry(&system, "outputs/colored_all_frustum.svg")?;
    println!("   ✅ Frustum channels: saved to colored_all_frustum.svg");
    println!("   📊 {} frustum channels (all red)", system.channels.len());
    println!();
    Ok(())
}

/// Demonstrate mixed smart system with multiple channel types
fn demonstrate_mixed_smart_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("4️⃣  Mixed Smart System (Multiple Colors)");
    
    // Create a larger system to encourage different channel types
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::default(), // Smart selection
    );
    
    plot_geometry(&system, "outputs/colored_mixed_smart.svg")?;
    println!("   ✅ Mixed system: saved to colored_mixed_smart.svg");
    
    // Analyze channel types
    let mut channel_counts = std::collections::HashMap::new();
    for channel in &system.channels {
        let channel_type_name = match &channel.channel_type {
            scheme::geometry::ChannelType::Straight => "Straight (Black)",
            scheme::geometry::ChannelType::SmoothStraight { .. } => "SmoothStraight (Black)",
            scheme::geometry::ChannelType::Serpentine { .. } => "Serpentine (Blue)",
            scheme::geometry::ChannelType::Arc { .. } => "Arc (Blue)",
            scheme::geometry::ChannelType::Frustum { .. } => "Frustum (Red)",
        };
        *channel_counts.entry(channel_type_name).or_insert(0) += 1;
    }
    
    println!("   📊 Channel type distribution:");
    for (channel_type, count) in &channel_counts {
        println!("      • {}: {} channels", channel_type, count);
    }
    println!();
    Ok(())
}

/// Demonstrate custom color configuration
fn demonstrate_custom_colors() -> Result<(), Box<dyn std::error::Error>> {
    println!("5️⃣  Custom Color Configuration");
    
    use scheme::visualizations::{RenderConfig, ChannelTypeStyles, LineStyle, Color};
    
    // Create a custom color scheme
    let custom_styles = ChannelTypeStyles {
        straight_style: LineStyle::solid(Color::rgb(100, 100, 100), 1.0), // Gray
        curved_style: LineStyle::solid(Color::rgb(0, 150, 0), 2.0), // Green
        tapered_style: LineStyle::solid(Color::rgb(255, 100, 0), 3.0), // Orange
    };
    
    let mut config = RenderConfig::default();
    config.channel_type_styles = custom_styles;
    config.title = "Custom Color Scheme".to_string();
    
    // Create a mixed system
    let system = create_geometry(
        (150.0, 80.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::Smart {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
            frustum_config: FrustumConfig::default(),
        },
    );
    
    // Use the custom configuration
    scheme::visualizations::schematic::plot_geometry_with_config(
        &system,
        "outputs/colored_custom_colors.svg",
        &config,
    )?;
    
    println!("   ✅ Custom colors: saved to colored_custom_colors.svg");
    println!("   🎨 Custom color scheme:");
    println!("      • Straight channels: Gray");
    println!("      • Curved channels: Green");
    println!("      • Tapered channels: Orange");
    println!();
    Ok(())
}
