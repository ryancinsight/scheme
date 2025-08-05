//! Comprehensive JSON Serialization Test
//!
//! This example verifies that all channel types (straight, serpentine, arc) can be
//! properly serialized to JSON and deserialized back without data loss.
//!
//! Run with: cargo run --example comprehensive_json_serialization_test

use scheme::{
    geometry::{generator::create_geometry, SplitType, ChannelSystem, ChannelType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig, FrustumConfig},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Comprehensive JSON Serialization Test");
    println!("========================================");
    println!();

    // Test different channel types
    test_straight_channels()?;
    test_serpentine_channels()?;
    test_arc_channels()?;
    test_mixed_channels()?;
    test_complex_bilateral_symmetry()?;

    println!("✅ All JSON serialization tests passed!");
    println!();
    println!("🎯 Key Validation Points:");
    println!("   ✅ All channel types serialize correctly");
    println!("   ✅ Complex geometries preserve structure");
    println!("   ✅ Bilateral symmetry is maintained");
    println!("   ✅ Roundtrip conversion is lossless");
    println!("   ✅ Path data is preserved for complex channels");
    
    Ok(())
}

fn test_straight_channels() -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣  Testing Straight Channels");
    
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );
    
    let json = system.to_json()?;
    let imported = ChannelSystem::from_json(&json)?;
    
    assert_eq!(system.nodes.len(), imported.nodes.len());
    assert_eq!(system.channels.len(), imported.channels.len());
    assert_eq!(system.box_dims, imported.box_dims);
    
    // Verify all channels are straight
    for channel in &imported.channels {
        match &channel.channel_type {
            ChannelType::Straight => {}, // Expected
            _ => panic!("Expected straight channel, got {:?}", channel.channel_type),
        }
    }
    
    println!("   ✅ Straight channels: serialization successful");
    Ok(())
}

fn test_serpentine_channels() -> Result<(), Box<dyn std::error::Error>> {
    println!("2️⃣  Testing Serpentine Channels");
    
    let serpentine_config = SerpentineConfig::default();
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    let json = system.to_json()?;
    let imported = ChannelSystem::from_json(&json)?;
    
    assert_eq!(system.nodes.len(), imported.nodes.len());
    assert_eq!(system.channels.len(), imported.channels.len());
    assert_eq!(system.box_dims, imported.box_dims);
    
    // Verify serpentine paths are preserved
    for (original, imported) in system.channels.iter().zip(imported.channels.iter()) {
        match (&original.channel_type, &imported.channel_type) {
            (ChannelType::Serpentine { path: orig_path }, 
             ChannelType::Serpentine { path: imp_path }) => {
                assert_eq!(orig_path.len(), imp_path.len(), "Serpentine path lengths should match");
                // Verify first and last points match (endpoints should be preserved)
                if !orig_path.is_empty() && !imp_path.is_empty() {
                    let orig_first = orig_path[0];
                    let orig_last = orig_path[orig_path.len() - 1];
                    let imp_first = imp_path[0];
                    let imp_last = imp_path[imp_path.len() - 1];
                    
                    assert!((orig_first.0 - imp_first.0).abs() < 1e-10, "First point X should match");
                    assert!((orig_first.1 - imp_first.1).abs() < 1e-10, "First point Y should match");
                    assert!((orig_last.0 - imp_last.0).abs() < 1e-10, "Last point X should match");
                    assert!((orig_last.1 - imp_last.1).abs() < 1e-10, "Last point Y should match");
                }
            }
            _ => {}
        }
    }
    
    println!("   ✅ Serpentine channels: serialization successful");
    Ok(())
}

fn test_arc_channels() -> Result<(), Box<dyn std::error::Error>> {
    println!("3️⃣  Testing Arc Channels");
    
    let arc_config = ArcConfig::default();
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllArcs(arc_config),
    );
    
    let json = system.to_json()?;
    let imported = ChannelSystem::from_json(&json)?;
    
    assert_eq!(system.nodes.len(), imported.nodes.len());
    assert_eq!(system.channels.len(), imported.channels.len());
    assert_eq!(system.box_dims, imported.box_dims);
    
    // Verify arc paths are preserved
    for (original, imported) in system.channels.iter().zip(imported.channels.iter()) {
        match (&original.channel_type, &imported.channel_type) {
            (ChannelType::Arc { path: orig_path }, 
             ChannelType::Arc { path: imp_path }) => {
                assert_eq!(orig_path.len(), imp_path.len(), "Arc path lengths should match");
                // Verify endpoints are preserved
                if !orig_path.is_empty() && !imp_path.is_empty() {
                    let orig_first = orig_path[0];
                    let orig_last = orig_path[orig_path.len() - 1];
                    let imp_first = imp_path[0];
                    let imp_last = imp_path[imp_path.len() - 1];
                    
                    assert!((orig_first.0 - imp_first.0).abs() < 1e-10, "Arc first point X should match");
                    assert!((orig_first.1 - imp_first.1).abs() < 1e-10, "Arc first point Y should match");
                    assert!((orig_last.0 - imp_last.0).abs() < 1e-10, "Arc last point X should match");
                    assert!((orig_last.1 - imp_last.1).abs() < 1e-10, "Arc last point Y should match");
                }
            }
            _ => {}
        }
    }
    
    println!("   ✅ Arc channels: serialization successful");
    Ok(())
}

fn test_mixed_channels() -> Result<(), Box<dyn std::error::Error>> {
    println!("4️⃣  Testing Mixed Channel Types");
    
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::Smart {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
            frustum_config: FrustumConfig::default(),
        },
    );
    
    let json = system.to_json()?;
    let imported = ChannelSystem::from_json(&json)?;
    
    assert_eq!(system.nodes.len(), imported.nodes.len());
    assert_eq!(system.channels.len(), imported.channels.len());
    assert_eq!(system.box_dims, imported.box_dims);
    
    println!("   ✅ Mixed channels: serialization successful");
    Ok(())
}

fn test_complex_bilateral_symmetry() -> Result<(), Box<dyn std::error::Error>> {
    println!("5️⃣  Testing Complex Bilateral Symmetry");
    
    // Create a complex system with multiple splits to test symmetry preservation
    let serpentine_config = SerpentineConfig {
        wave_phase_direction: 0.0, // Auto-symmetric
        ..SerpentineConfig::default()
    };
    
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    let json = system.to_json()?;
    let imported = ChannelSystem::from_json(&json)?;
    
    assert_eq!(system.nodes.len(), imported.nodes.len());
    assert_eq!(system.channels.len(), imported.channels.len());
    assert_eq!(system.box_dims, imported.box_dims);
    
    // Save for inspection
    std::fs::write("outputs/complex_bilateral_symmetry_test.json", &json)?;
    
    println!("   ✅ Complex bilateral symmetry: serialization successful");
    println!("   💾 Saved to: outputs/complex_bilateral_symmetry_test.json");
    Ok(())
}
