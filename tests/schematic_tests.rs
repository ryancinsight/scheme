//! tests/schematic_tests.rs
//! 
//! Tests for 2D microfluidic schematic generation and visualization

use scheme::{
    geometry::{create_geometry, SplitType},
    config::GeometryConfig,
    visualizations::plot_geometry,
};
use std::fs;
use std::path::Path;

/// Test that basic schematic generation works
#[test]
fn test_basic_schematic_generation() {
    let config = GeometryConfig {
        wall_clearance: 4.0,
        channel_width: 6.0,
        channel_height: 6.0,
    };
    
    let system = create_geometry(
        (200.0, 100.0),  // box dimensions
        &[SplitType::Bifurcation],  // splits array
        &config,  // geometry config
    );

    // Verify the system has the expected structure
    assert!(!system.nodes.is_empty(), "System should have nodes");
    assert!(!system.channels.is_empty(), "System should have channels");
    assert_eq!(system.box_dims, (200.0, 100.0), "Box dimensions should match");
}

/// Test bifurcation pattern generation
#[test]
fn test_bifurcation_pattern() {
    let config = GeometryConfig::default();
    
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation, SplitType::Bifurcation],  // 2 levels of bifurcation
        &config,
    );

    // Bifurcation should create more nodes and channels
    assert!(system.nodes.len() >= 3, "Bifurcation should create multiple nodes");
    assert!(system.channels.len() >= 2, "Bifurcation should create multiple channels");
}

/// Test trifurcation pattern generation
#[test]
fn test_trifurcation_pattern() {
    let config = GeometryConfig::default();
    
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Trifurcation],  // 1 level of trifurcation
        &config,
    );

    // Trifurcation should create appropriate structure
    assert!(system.nodes.len() >= 4, "Trifurcation should create multiple nodes");
    assert!(system.channels.len() >= 3, "Trifurcation should create multiple channels");
}

/// Test that visualization generates output files
#[test]
fn test_schematic_visualization() {
    let config = GeometryConfig::default();
    
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
    );

    let output_dir = "test_outputs";
    fs::create_dir_all(output_dir).expect("Should create output directory");
    
    let plot_path = format!("{}/test_schematic.png", output_dir);
    
    // This should not panic
    let result = plot_geometry(&system, &plot_path);
    assert!(result.is_ok(), "Plotting should succeed");
    
    // Verify file was created
    assert!(Path::new(&plot_path).exists(), "Plot file should be created");
    
    // Clean up
    fs::remove_file(&plot_path).ok();
    fs::remove_dir(output_dir).ok();
}

/// Test split type functionality
#[test]
fn test_split_type_branch_count() {
    assert_eq!(SplitType::Bifurcation.branch_count(), 2);
    assert_eq!(SplitType::Trifurcation.branch_count(), 3);
}

/// Test channel system line extraction
#[test]
fn test_channel_system_lines() {
    let config = GeometryConfig {
        wall_clearance: 4.0,
        channel_width: 3.0,
        channel_height: 3.0,
    };
    
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &config,
    );

    let lines = system.get_lines();
    assert!(!lines.is_empty(), "Should extract lines from channels");
    assert_eq!(lines.len(), system.channels.len(), "Should have one line per channel");
}
