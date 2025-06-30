//! tests/schematic_tests.rs
//! 
//! Tests for 2D microfluidic schematic generation and visualization

use scheme::{
    geometry::{generator::create_geometry, SplitType, ChannelType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig},
    visualizations::schematic::plot_geometry,
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
        &ChannelTypeConfig::AllStraight,  // channel type config
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
        &ChannelTypeConfig::AllStraight,
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
        &ChannelTypeConfig::AllStraight,
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
        &ChannelTypeConfig::AllStraight,
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
        &ChannelTypeConfig::AllStraight,
    );

    let lines = system.get_lines();
    assert!(!lines.is_empty(), "Should extract lines from channels");
    // For straight channels, should have box outline (4 lines) + one line per channel
    let expected_lines = 4 + system.channels.len(); // box outline + channel lines
    assert_eq!(lines.len(), expected_lines, "Should have box outline plus one line per straight channel");
}

/// Test serpentine channel generation
#[test]
fn test_serpentine_channels() {
    let config = GeometryConfig::default();
    
    // Create system with all serpentine channels
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(Default::default()),
    );

    // Verify all channels are serpentine
    for channel in &system.channels {
        match &channel.channel_type {
            ChannelType::Serpentine { path } => {
                // Serpentine channels should have multiple path points
                assert!(path.len() > 2, "Serpentine channels should have multiple path points");
            },
            _ => panic!("Expected all channels to be serpentine"),
        }
    }
}

/// Test mixed channel types
#[test]
fn test_mixed_channel_types() {
    let config = GeometryConfig::default();
    
    // Create system with mixed channel types
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::default(), // Uses mixed by position
    );

    // Should have both straight and serpentine channels
    let mut has_straight = false;
    let mut has_serpentine = false;
    
    for channel in &system.channels {
        match channel.channel_type {
            ChannelType::Straight => has_straight = true,
            ChannelType::Serpentine { .. } => has_serpentine = true,
        }
    }
    
    assert!(has_straight, "Should have some straight channels");
    assert!(has_serpentine, "Should have some serpentine channels");
}

/// Test that serpentine channels start and end exactly at node positions
#[test]
fn test_serpentine_path_endpoints() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 2.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 1.5,
    };
    
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Verify that serpentine channels start and end exactly at node positions
    for channel in &system.channels {
        let from_node = &system.nodes[channel.from_node];
        let to_node = &system.nodes[channel.to_node];
        
        if let ChannelType::Serpentine { path } = &channel.channel_type {
            if !path.is_empty() {
                let first_point = path.first().unwrap();
                let last_point = path.last().unwrap();
                
                // Check that first point exactly matches from_node (should be exact now)
                assert_eq!(first_point.0, from_node.point.0, 
                    "First point X: {} != Node X: {}", first_point.0, from_node.point.0);
                assert_eq!(first_point.1, from_node.point.1,
                    "First point Y: {} != Node Y: {}", first_point.1, from_node.point.1);
                
                // Check that last point exactly matches to_node (should be exact now)
                assert_eq!(last_point.0, to_node.point.0,
                    "Last point X: {} != Node X: {}", last_point.0, to_node.point.0);
                assert_eq!(last_point.1, to_node.point.1,
                    "Last point Y: {} != Node Y: {}", last_point.1, to_node.point.1);
            }
        }
    }
}

/// Test serpentine path smoothness and point density
#[test]
fn test_serpentine_path_smoothness() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 0.5,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
    };
    
    let system = create_geometry(
        (200.0, 100.0),
        &[],  // Single straight channel
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Should have exactly one channel for empty splits
    assert_eq!(system.channels.len(), 1);
    
    let channel = &system.channels[0];
    
    if let ChannelType::Serpentine { path } = &channel.channel_type {
        assert!(path.len() > 10, "Should have many points for smooth curve");
        
        // Verify path starts and ends at correct positions
        let first_point = path.first().unwrap();
        let last_point = path.last().unwrap();
        
        let epsilon = 1e-10;
        assert!((first_point.0 - 0.0).abs() < epsilon, "Should start at x=0");
        assert!((first_point.1 - 50.0).abs() < epsilon, "Should start at y=height/2");
        assert!((last_point.0 - 200.0).abs() < epsilon, "Should end at x=length");
        assert!((last_point.1 - 50.0).abs() < epsilon, "Should end at y=height/2");
    } else {
        panic!("Expected serpentine channel");
    }
}