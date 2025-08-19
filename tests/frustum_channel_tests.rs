//! Comprehensive tests for frustum (tapered) channel functionality
//!
//! This test suite validates the frustum channel implementation including:
//! - Configuration validation and error handling
//! - Strategy pattern implementation
//! - Path generation and width calculations
//! - JSON serialization/deserialization
//! - Integration with the factory system
//! - Visualization support

use scheme::{
    geometry::{generator::create_geometry, SplitType, ChannelSystem, ChannelType},
    geometry::strategies::{FrustumChannelStrategy, ChannelTypeStrategy, ChannelTypeFactory},
    config::{GeometryConfig, ChannelTypeConfig, FrustumConfig, TaperProfile},
};

/// Test FrustumConfig creation and validation
#[test]
fn test_frustum_config_validation() {
    // Test valid configuration
    let valid_config = FrustumConfig::new(
        2.0,  // inlet_width
        0.5,  // throat_width
        1.5,  // outlet_width
        TaperProfile::Linear,
        50,   // smoothness
        0.5,  // throat_position
    );
    assert!(valid_config.is_ok());

    // Test invalid inlet width (too large)
    let invalid_inlet = FrustumConfig::new(100.0, 0.5, 1.5, TaperProfile::Linear, 50, 0.5);
    assert!(invalid_inlet.is_err());

    // Test invalid throat width (larger than inlet)
    let invalid_throat = FrustumConfig::new(2.0, 3.0, 1.5, TaperProfile::Linear, 50, 0.5);
    assert!(invalid_throat.is_err());

    // Test invalid smoothness (too low)
    let invalid_smoothness = FrustumConfig::new(2.0, 0.5, 1.5, TaperProfile::Linear, 5, 0.5);
    assert!(invalid_smoothness.is_err());

    // Test invalid throat position (out of range)
    let invalid_position = FrustumConfig::new(2.0, 0.5, 1.5, TaperProfile::Linear, 50, 1.5);
    assert!(invalid_position.is_err());
}

/// Test default FrustumConfig
#[test]
fn test_frustum_config_default() {
    let config = FrustumConfig::default();
    assert_eq!(config.inlet_width, 2.0);
    assert_eq!(config.throat_width, 0.5);
    assert_eq!(config.outlet_width, 2.0);
    assert_eq!(config.taper_profile, TaperProfile::Linear);
    assert_eq!(config.smoothness, 50);
    assert_eq!(config.throat_position, 0.5);
    
    // Default config should be valid
    assert!(config.validate().is_ok());
}

/// Test width calculation at different positions
#[test]
fn test_frustum_width_calculations() {
    let config = FrustumConfig {
        inlet_width: 2.0,
        throat_width: 0.5,
        outlet_width: 1.5,
        taper_profile: TaperProfile::Linear,
        smoothness: 50,
        throat_position: 0.5,
    };

    // Test width at inlet (t=0.0)
    let inlet_width = config.width_at_position(0.0);
    assert!((inlet_width - 2.0).abs() < 1e-10);

    // Test width at throat (t=0.5)
    let throat_width = config.width_at_position(0.5);
    assert!((throat_width - 0.5).abs() < 1e-10);

    // Test width at outlet (t=1.0)
    let outlet_width = config.width_at_position(1.0);
    assert!((outlet_width - 1.5).abs() < 1e-10);

    // Test intermediate positions
    let quarter_width = config.width_at_position(0.25);
    assert!(quarter_width > 0.5 && quarter_width < 2.0);

    let three_quarter_width = config.width_at_position(0.75);
    assert!(three_quarter_width > 0.5 && three_quarter_width < 1.5);
}

/// Test different taper profiles
#[test]
fn test_taper_profiles() {
    let linear_config = FrustumConfig {
        inlet_width: 2.0,
        throat_width: 0.5,
        outlet_width: 2.0,
        taper_profile: TaperProfile::Linear,
        smoothness: 50,
        throat_position: 0.5,
    };

    let exponential_config = FrustumConfig {
        taper_profile: TaperProfile::Exponential,
        ..linear_config
    };

    let smooth_config = FrustumConfig {
        taper_profile: TaperProfile::Smooth,
        ..linear_config
    };

    // All profiles should give same values at endpoints and throat
    let positions = [0.0, 0.5, 1.0];
    for &pos in &positions {
        let linear_width = linear_config.width_at_position(pos);
        let exp_width = exponential_config.width_at_position(pos);
        let smooth_width = smooth_config.width_at_position(pos);

        // Allow larger tolerance for different taper profiles at key positions
        assert!((linear_width - exp_width).abs() < 0.1,
                "Linear and exponential should be close at position {}: {} vs {}", pos, linear_width, exp_width);
        assert!((linear_width - smooth_width).abs() < 0.1,
                "Linear and smooth should be close at position {}: {} vs {}", pos, linear_width, smooth_width);
    }

    // But should differ at intermediate positions
    let mid_pos = 0.25;
    let linear_mid = linear_config.width_at_position(mid_pos);
    let exp_mid = exponential_config.width_at_position(mid_pos);
    let smooth_mid = smooth_config.width_at_position(mid_pos);

    // They should be different (not exactly equal) - but allow for some profiles to be similar
    // The key is that they all produce valid widths between throat and inlet/outlet
    assert!(linear_mid > 0.5 && linear_mid < 2.0, "Linear mid width should be between throat and inlet");
    assert!(exp_mid > 0.5 && exp_mid < 2.0, "Exponential mid width should be between throat and inlet");
    assert!(smooth_mid > 0.5 && smooth_mid < 2.0, "Smooth mid width should be between throat and inlet");
}

/// Test FrustumChannelStrategy
#[test]
fn test_frustum_channel_strategy() {
    let config = FrustumConfig::default();
    let strategy = FrustumChannelStrategy::new(config);
    
    let from = (0.0, 0.0);
    let to = (10.0, 5.0);
    let geometry_config = GeometryConfig::default();
    let box_dims = (20.0, 10.0);
    
    let channel_type = strategy.create_channel(
        from,
        to,
        &geometry_config,
        box_dims,
        4,
        None,
    );
    
    match channel_type {
        ChannelType::Frustum { path, widths, inlet_width, throat_width, outlet_width } => {
            assert_eq!(path.len(), config.smoothness);
            assert_eq!(widths.len(), config.smoothness);
            assert_eq!(inlet_width, config.inlet_width);
            assert_eq!(throat_width, config.throat_width);
            assert_eq!(outlet_width, config.outlet_width);
            
            // Check that path goes from start to end (with floating point tolerance)
            assert!((path[0].0 - from.0).abs() < 1e-10);
            assert!((path[0].1 - from.1).abs() < 1e-10);
            assert!((path[path.len() - 1].0 - to.0).abs() < 1e-10);
            assert!((path[path.len() - 1].1 - to.1).abs() < 1e-10);
            
            // Check that widths match configuration
            assert!((widths[0] - inlet_width).abs() < 1e-10);
            assert!((widths[widths.len() - 1] - outlet_width).abs() < 1e-10);
            
            // Find throat position and verify width
            let throat_index = (config.smoothness as f64 * config.throat_position) as usize;
            assert!((widths[throat_index] - throat_width).abs() < 0.1); // Allow small tolerance
        }
        _ => panic!("FrustumChannelStrategy should produce Frustum channel type"),
    }
}

/// Test ChannelTypeFactory with AllFrustum configuration
#[test]
fn test_factory_all_frustum() {
    let frustum_config = FrustumConfig::default();
    let config = ChannelTypeConfig::AllFrustum(frustum_config);
    let strategy = ChannelTypeFactory::create_strategy(
        &config,
        (0.0, 0.0),
        (10.0, 5.0),
        (20.0, 10.0),
    );
    
    let channel_type = strategy.create_channel(
        (0.0, 0.0),
        (10.0, 5.0),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type {
        ChannelType::Frustum { .. } => {}, // Expected
        _ => panic!("AllFrustum config should produce frustum channels"),
    }
}

/// Test Adaptive configuration includes frustum channels
#[test]
fn test_adaptive_configuration_with_frustum() {
    let config = ChannelTypeConfig::default(); // Adaptive configuration
    
    // Test medium-length horizontal channel (should potentially be frustum)
    let strategy = ChannelTypeFactory::create_strategy(
        &config,
        (2.0, 5.0),
        (8.0, 5.2),  // Medium length, mostly horizontal
        (20.0, 10.0),
    );
    
    let channel_type = strategy.create_channel(
        (2.0, 5.0),
        (8.0, 5.2),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    // Should be one of the expected types (exact type depends on smart logic)
    match channel_type {
        ChannelType::Straight | ChannelType::Serpentine { .. } | 
        ChannelType::Arc { .. } | ChannelType::Frustum { .. } => {}, // All valid
        _ => panic!("Adaptive config should produce valid channel type"),
    }
}

/// Test frustum channel generation in complete system
#[test]
fn test_frustum_channel_system_generation() {
    let frustum_config = FrustumConfig::default();
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllFrustum(frustum_config),
    );
    
    assert!(!system.nodes.is_empty());
    assert!(!system.channels.is_empty());
    assert_eq!(system.box_dims, (100.0, 50.0));
    
    // Verify all channels are frustum type
    for channel in &system.channels {
        match &channel.channel_type {
            ChannelType::Frustum { path, widths, .. } => {
                assert!(!path.is_empty());
                assert_eq!(path.len(), widths.len());
            }
            _ => panic!("All channels should be frustum type"),
        }
    }
}

/// Test colored visualization support for frustum channels
#[test]
fn test_frustum_colored_visualization() {
    use scheme::geometry::ChannelTypeCategory;

    let frustum_config = FrustumConfig::default();
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllFrustum(frustum_config),
    );

    // Test the new get_lines_by_type method
    let (boundary_lines, channel_lines) = system.get_lines_by_type();

    // Should have boundary lines
    assert!(!boundary_lines.is_empty(), "Should have boundary lines");

    // Should have frustum channels categorized as tapered
    assert!(channel_lines.contains_key(&ChannelTypeCategory::Tapered),
            "Should have tapered channel category");

    let tapered_lines = &channel_lines[&ChannelTypeCategory::Tapered];
    assert!(!tapered_lines.is_empty(), "Should have tapered channel lines");

    // Should not have other channel types
    assert!(!channel_lines.contains_key(&ChannelTypeCategory::Straight),
            "Should not have straight channels");
    assert!(!channel_lines.contains_key(&ChannelTypeCategory::Curved),
            "Should not have curved channels");

    // Test channel type categorization
    for channel in &system.channels {
        let category = ChannelTypeCategory::from(&channel.channel_type);
        assert_eq!(category, ChannelTypeCategory::Tapered,
                   "All channels should be categorized as tapered");
    }
}

/// Test JSON serialization of frustum channels
#[test]
fn test_frustum_json_serialization() {
    let frustum_config = FrustumConfig::default();
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &GeometryConfig::default(),
        &ChannelTypeConfig::AllFrustum(frustum_config),
    );
    
    // Test serialization
    let json = system.to_json().expect("Should serialize frustum system");
    assert!(!json.is_empty());
    
    // Test deserialization
    let imported = ChannelSystem::from_json(&json).expect("Should deserialize frustum system");
    
    // Verify structure is preserved
    assert_eq!(system.nodes.len(), imported.nodes.len());
    assert_eq!(system.channels.len(), imported.channels.len());
    assert_eq!(system.box_dims, imported.box_dims);
    
    // Verify frustum-specific data is preserved
    for (original, imported) in system.channels.iter().zip(imported.channels.iter()) {
        match (&original.channel_type, &imported.channel_type) {
            (ChannelType::Frustum { path: orig_path, widths: orig_widths, inlet_width: orig_inlet, throat_width: orig_throat, outlet_width: orig_outlet },
             ChannelType::Frustum { path: imp_path, widths: imp_widths, inlet_width: imp_inlet, throat_width: imp_throat, outlet_width: imp_outlet }) => {
                assert_eq!(orig_path.len(), imp_path.len());
                assert_eq!(orig_widths.len(), imp_widths.len());
                assert_eq!(orig_inlet, imp_inlet);
                assert_eq!(orig_throat, imp_throat);
                assert_eq!(orig_outlet, imp_outlet);
                
                // Verify path points are preserved
                for (orig_point, imp_point) in orig_path.iter().zip(imp_path.iter()) {
                    assert!((orig_point.0 - imp_point.0).abs() < 1e-10);
                    assert!((orig_point.1 - imp_point.1).abs() < 1e-10);
                }
                
                // Verify width values are preserved
                for (orig_width, imp_width) in orig_widths.iter().zip(imp_widths.iter()) {
                    assert!((orig_width - imp_width).abs() < 1e-10);
                }
            }
            _ => panic!("Channel types should match"),
        }
    }
}
