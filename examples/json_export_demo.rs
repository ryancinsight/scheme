//! JSON Export Demonstration
//!
//! This example demonstrates the new JSON export and import capabilities
//! of the scheme crate. It shows how to:
//!
//! 1. Create channel systems using the existing geometry generator
//! 2. Export them to JSON format for interoperability
//! 3. Import them back from JSON
//! 4. Use the JSON format for integration with other tools like OxiCFD
//!
//! Run with: cargo run --example json_export_demo

use scheme::{
    geometry::{generator::create_geometry, SplitType, ChannelSystem},
    config::{GeometryConfig, ChannelTypeConfig},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Scheme JSON Export/Import Demonstration");
    println!("==========================================");
    println!();

    // Demonstrate JSON export for different channel configurations
    demonstrate_single_channel_export()?;
    demonstrate_bifurcation_export()?;
    demonstrate_trifurcation_export()?;
    demonstrate_complex_system_export()?;
    demonstrate_roundtrip_conversion()?;

    println!("✅ JSON Export/Import demonstration completed successfully!");
    println!();
    println!("🎯 Key Benefits:");
    println!("   ✅ Native JSON serialization support");
    println!("   ✅ Seamless interoperability with OxiCFD");
    println!("   ✅ Easy data persistence and transfer");
    println!("   ✅ Human-readable format for debugging");
    println!("   ✅ Version-safe serialization");

    Ok(())
}

/// Demonstrate single channel JSON export
fn demonstrate_single_channel_export() -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣  Single Channel JSON Export");
    println!("   ============================");

    // Create a simple single channel system
    let system = create_geometry(
        (20.0, 5.0),  // 20mm x 5mm device
        &[],          // No splits = single channel
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );

    println!("   📐 Created single channel system:");
    println!("      Nodes: {}", system.nodes.len());
    println!("      Channels: {}", system.channels.len());
    println!("      Box dimensions: {:?}", system.box_dims);

    // Export to JSON
    let json = system.to_json()?;
    println!("   📄 Exported to JSON ({} characters)", json.len());

    // Save to file
    std::fs::write("single_channel_export.json", &json)?;
    println!("   💾 Saved to: single_channel_export.json");

    println!();
    Ok(())
}

/// Demonstrate bifurcation JSON export
fn demonstrate_bifurcation_export() -> Result<(), Box<dyn std::error::Error>> {
    println!("2️⃣  Bifurcation JSON Export");
    println!("   =========================");

    // Create a bifurcation system
    let system = create_geometry(
        (30.0, 15.0),  // 30mm x 15mm device
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );

    println!("   📐 Created bifurcation system:");
    println!("      Nodes: {}", system.nodes.len());
    println!("      Channels: {}", system.channels.len());
    println!("      Box dimensions: {:?}", system.box_dims);

    // Export to JSON
    let json = system.to_json()?;
    println!("   📄 Exported to JSON ({} characters)", json.len());

    // Save to file
    std::fs::write("bifurcation_export.json", &json)?;
    println!("   💾 Saved to: bifurcation_export.json");

    println!();
    Ok(())
}

/// Demonstrate trifurcation JSON export
fn demonstrate_trifurcation_export() -> Result<(), Box<dyn std::error::Error>> {
    println!("3️⃣  Trifurcation JSON Export");
    println!("   ==========================");

    // Create a trifurcation system
    let system = create_geometry(
        (35.0, 20.0),  // 35mm x 20mm device
        &[SplitType::Trifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );

    println!("   📐 Created trifurcation system:");
    println!("      Nodes: {}", system.nodes.len());
    println!("      Channels: {}", system.channels.len());
    println!("      Box dimensions: {:?}", system.box_dims);

    // Export to JSON
    let json = system.to_json()?;
    println!("   📄 Exported to JSON ({} characters)", json.len());

    // Save to file
    std::fs::write("trifurcation_export.json", &json)?;
    println!("   💾 Saved to: trifurcation_export.json");

    println!();
    Ok(())
}

/// Demonstrate complex system JSON export
fn demonstrate_complex_system_export() -> Result<(), Box<dyn std::error::Error>> {
    println!("4️⃣  Complex System JSON Export");
    println!("   ============================");

    // Create a complex multi-level system
    let system = create_geometry(
        (50.0, 30.0),  // 50mm x 30mm device
        &[
            SplitType::Bifurcation,
            SplitType::Trifurcation,
            SplitType::Bifurcation,
        ],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );

    println!("   📐 Created complex system:");
    println!("      Nodes: {}", system.nodes.len());
    println!("      Channels: {}", system.channels.len());
    println!("      Box dimensions: {:?}", system.box_dims);
    println!("      Split pattern: Bifurcation → Trifurcation → Bifurcation");

    // Export to JSON
    let json = system.to_json()?;
    println!("   📄 Exported to JSON ({} characters)", json.len());

    // Save to file
    std::fs::write("complex_system_export.json", &json)?;
    println!("   💾 Saved to: complex_system_export.json");

    // Show a snippet of the JSON structure
    println!("   📋 JSON structure preview:");
    let lines: Vec<&str> = json.lines().take(10).collect();
    for line in lines {
        println!("      {}", line);
    }
    if json.lines().count() > 10 {
        println!("      ... ({} more lines)", json.lines().count() - 10);
    }

    println!();
    Ok(())
}

/// Demonstrate roundtrip conversion (export → import)
fn demonstrate_roundtrip_conversion() -> Result<(), Box<dyn std::error::Error>> {
    println!("5️⃣  Roundtrip Conversion Test");
    println!("   ===========================");

    // Create original system
    let original_system = create_geometry(
        (25.0, 12.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );

    println!("   📐 Original system:");
    println!("      Nodes: {}", original_system.nodes.len());
    println!("      Channels: {}", original_system.channels.len());
    println!("      Box dimensions: {:?}", original_system.box_dims);

    // Export to JSON
    let json = original_system.to_json()?;
    println!("   📤 Exported to JSON");

    // Import back from JSON
    let imported_system = ChannelSystem::from_json(&json)?;
    println!("   📥 Imported from JSON");

    // Verify roundtrip integrity
    println!("   🔍 Verifying roundtrip integrity:");
    
    let nodes_match = original_system.nodes.len() == imported_system.nodes.len();
    let channels_match = original_system.channels.len() == imported_system.channels.len();
    let box_dims_match = original_system.box_dims == imported_system.box_dims;
    
    println!("      Nodes count match: {}", if nodes_match { "✅" } else { "❌" });
    println!("      Channels count match: {}", if channels_match { "✅" } else { "❌" });
    println!("      Box dimensions match: {}", if box_dims_match { "✅" } else { "❌" });

    if nodes_match && channels_match && box_dims_match {
        println!("   ✅ Roundtrip conversion successful!");
    } else {
        println!("   ❌ Roundtrip conversion failed!");
    }

    // Save the roundtrip test result
    std::fs::write("roundtrip_test.json", &json)?;
    println!("   💾 Saved roundtrip test to: roundtrip_test.json");

    println!();
    Ok(())
}
