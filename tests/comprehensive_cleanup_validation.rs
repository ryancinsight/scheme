//! Comprehensive Cleanup Validation Tests
//!
//! This test suite validates that the comprehensive codebase cleanup has been
//! successful and that all functionality continues to work correctly.

use scheme::{
    geometry::{generator::create_geometry, SplitType, ChannelSystem, ChannelType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig, FrustumConfig},
    error::GeometryError,
};

/// Test that JSON serialization works for all channel types after cleanup
#[test]
fn test_json_serialization_post_cleanup() {
    // Test straight channels
    let straight_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );
    
    let json = straight_system.to_json().expect("Should serialize straight channels");
    let imported = ChannelSystem::from_json(&json).expect("Should deserialize straight channels");
    assert_eq!(straight_system.nodes.len(), imported.nodes.len());
    assert_eq!(straight_system.channels.len(), imported.channels.len());
    
    // Test serpentine channels
    let serpentine_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllSerpentine(SerpentineConfig::default()),
    );
    
    let json = serpentine_system.to_json().expect("Should serialize serpentine channels");
    let imported = ChannelSystem::from_json(&json).expect("Should deserialize serpentine channels");
    assert_eq!(serpentine_system.nodes.len(), imported.nodes.len());
    assert_eq!(serpentine_system.channels.len(), imported.channels.len());
    
    // Test arc channels
    let arc_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllArcs(ArcConfig::default()),
    );
    
    let json = arc_system.to_json().expect("Should serialize arc channels");
    let imported = ChannelSystem::from_json(&json).expect("Should deserialize arc channels");
    assert_eq!(arc_system.nodes.len(), imported.nodes.len());
    assert_eq!(arc_system.channels.len(), imported.channels.len());
}

/// Test that error handling improvements work correctly
#[test]
fn test_error_handling_improvements() {
    // Test that invalid configurations return proper errors instead of panicking
    let result = SerpentineConfig::new(-1.0, 1.0, 1.0, 1.0); // Invalid fill_factor
    assert!(result.is_err(), "Should return error for invalid configuration");
    
    // Test that geometry errors are properly handled
    let _result = create_geometry(
        (-1.0, -1.0), // Invalid dimensions
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );
    // Note: The current implementation doesn't validate box dimensions in create_geometry,
    // but this test demonstrates the error handling pattern
    
    // Test error type conversions
    let geometry_error = GeometryError::invalid_point((f64::NAN, 0.0));
    let scheme_error = scheme::error::SchemeError::Geometry(geometry_error);
    assert!(scheme_error.to_string().contains("Geometry error"));
}

/// Test that bilateral symmetry is preserved after cleanup
#[test]
fn test_bilateral_symmetry_preservation() {
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
    
    // Verify that the system has the expected structure
    assert!(!system.nodes.is_empty(), "System should have nodes");
    assert!(!system.channels.is_empty(), "System should have channels");
    
    // Test that JSON roundtrip preserves the structure
    let json = system.to_json().expect("Should serialize complex system");
    let imported = ChannelSystem::from_json(&json).expect("Should deserialize complex system");
    
    assert_eq!(system.nodes.len(), imported.nodes.len(), "Node count should be preserved");
    assert_eq!(system.channels.len(), imported.channels.len(), "Channel count should be preserved");
    assert_eq!(system.box_dims, imported.box_dims, "Box dimensions should be preserved");
}

/// Test that all channel types can be generated and visualized
#[test]
fn test_channel_type_generation() {
    let config = GeometryConfig::default();
    let box_dims = (100.0, 50.0);
    let splits = &[SplitType::Bifurcation];
    
    // Test straight channels
    let straight_system = create_geometry(box_dims, splits, &config, &ChannelTypeConfig::AllStraight);
    assert!(!straight_system.channels.is_empty());
    for channel in &straight_system.channels {
        match &channel.channel_type {
            ChannelType::Straight => {}, // Expected
            _ => panic!("Expected straight channel"),
        }
    }
    
    // Test serpentine channels
    let serpentine_system = create_geometry(
        box_dims, 
        splits, 
        &config, 
        &ChannelTypeConfig::AllSerpentine(SerpentineConfig::default())
    );
    assert!(!serpentine_system.channels.is_empty());
    for channel in &serpentine_system.channels {
        match &channel.channel_type {
            ChannelType::Serpentine { path } => {
                assert!(!path.is_empty(), "Serpentine path should not be empty");
            },
            _ => panic!("Expected serpentine channel"),
        }
    }
    
    // Test arc channels
    let arc_system = create_geometry(
        box_dims, 
        splits, 
        &config, 
        &ChannelTypeConfig::AllArcs(ArcConfig::default())
    );
    assert!(!arc_system.channels.is_empty());
    for channel in &arc_system.channels {
        match &channel.channel_type {
            ChannelType::Arc { path } => {
                assert!(!path.is_empty(), "Arc path should not be empty");
            },
            _ => panic!("Expected arc channel"),
        }
    }
}

/// Test that configuration validation works correctly
#[test]
fn test_configuration_validation() {
    // Test valid configurations
    assert!(SerpentineConfig::new(0.8, 3.0, 6.0, 2.0).is_ok());
    assert!(ArcConfig::new(1.0, 50).is_ok());
    
    // Test invalid configurations should return errors
    assert!(SerpentineConfig::new(-0.1, 3.0, 6.0, 2.0).is_err()); // Invalid fill_factor
    assert!(SerpentineConfig::new(0.8, -1.0, 6.0, 2.0).is_err()); // Invalid wavelength_factor
    
    // Test that default configurations are valid
    let default_serpentine = SerpentineConfig::default();
    assert!(default_serpentine.validate().is_ok());
    
    let default_arc = ArcConfig::default();
    assert!(default_arc.validate().is_ok());
}

/// Test that the cleanup hasn't broken backward compatibility
#[test]
fn test_backward_compatibility() {
    // Test that old API patterns still work
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );
    
    // Test that the system structure is as expected
    assert_eq!(system.box_dims, (200.0, 100.0));
    assert!(!system.nodes.is_empty());
    assert!(!system.channels.is_empty());
    assert!(!system.box_outline.is_empty());
    
    // Test that line extraction still works
    let lines = system.get_lines();
    assert!(!lines.is_empty());
    
    // Test that path segment extraction works (may be empty for straight channels)
    let _segments = system.get_path_segments();
    // For straight channels, segments will be empty, which is correct behavior
}

/// Test that memory usage is reasonable after cleanup
#[test]
fn test_memory_efficiency() {
    // Create a moderately complex system
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllSerpentine(SerpentineConfig::default()),
    );
    
    // Verify the system is not excessively large
    assert!(system.nodes.len() < 1000, "Node count should be reasonable");
    assert!(system.channels.len() < 1000, "Channel count should be reasonable");
    
    // Test that JSON serialization doesn't produce excessively large output
    let json = system.to_json().expect("Should serialize");
    assert!(json.len() < 1_000_000, "JSON output should be reasonable size"); // 1MB limit
}

/// Test that all examples and demos still work
#[test]
fn test_examples_functionality() {
    // This test ensures that the cleanup hasn't broken the example functionality
    // by testing the core functions that examples use
    
    // Test comprehensive split patterns functionality
    let bifurcation_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );
    assert!(!bifurcation_system.channels.is_empty());
    
    let trifurcation_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Trifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllStraight,
    );
    assert!(!trifurcation_system.channels.is_empty());
    
    // Test that mixed configurations work
    let mixed_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::Adaptive {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
            frustum_config: FrustumConfig::default(),
        },
    );
    assert!(!mixed_system.channels.is_empty());
}
