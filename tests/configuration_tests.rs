//! tests/configuration_tests.rs
//! 
//! Comprehensive tests for the improved configuration management system

use scheme::{
    config::{
        GeometryConfig, SerpentineConfig, ArcConfig, ChannelTypeConfig,
        ChannelTypeConfigBuilder, constants, presets,
    },

};

/// Test GeometryConfig validation
#[test]
fn test_geometry_config_validation() {
    // Test valid configuration
    let config = GeometryConfig::new(1.0, 2.0, 0.5).unwrap();
    assert_eq!(config.wall_clearance, 1.0);
    assert_eq!(config.channel_width, 2.0);
    assert_eq!(config.channel_height, 0.5);
    
    // Test invalid wall clearance
    let result = GeometryConfig::new(-0.1, 1.0, 0.5);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("wall_clearance"));
    assert!(error.to_string().contains("-0.1"));
    
    // Test invalid channel width
    let result = GeometryConfig::new(0.5, 0.0, 0.5);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("channel_width"));
    
    // Test invalid channel height
    let result = GeometryConfig::new(0.5, 1.0, -1.0);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("channel_height"));
}

/// Test SerpentineConfig validation
#[test]
fn test_serpentine_config_validation() {
    // Test valid configuration
    let config = SerpentineConfig::new(0.8, 3.0, 6.0, 2.0).unwrap();
    assert_eq!(config.fill_factor, 0.8);
    assert_eq!(config.wavelength_factor, 3.0);
    assert_eq!(config.gaussian_width_factor, 6.0);
    assert_eq!(config.wave_density_factor, 2.0);
    
    // Test invalid fill factor (too low)
    let result = SerpentineConfig::new(0.05, 3.0, 6.0, 2.0);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("fill_factor"));
    assert!(error.to_string().contains("0.05"));
    
    // Test invalid fill factor (too high)
    let result = SerpentineConfig::new(1.1, 3.0, 6.0, 2.0);
    assert!(result.is_err());
    
    // Test invalid wavelength factor
    let result = SerpentineConfig::new(0.8, 0.5, 6.0, 2.0);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("wavelength_factor"));
    
    // Test invalid gaussian width factor
    let result = SerpentineConfig::new(0.8, 3.0, 1.0, 2.0);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("gaussian_width_factor"));
    
    // Test invalid wave density factor
    let result = SerpentineConfig::new(0.8, 3.0, 6.0, 0.1);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("wave_density_factor"));
}

/// Test ArcConfig validation
#[test]
fn test_arc_config_validation() {
    // Test valid configuration
    let config = ArcConfig::new(0.5, 30).unwrap();
    assert_eq!(config.curvature_factor, 0.5);
    assert_eq!(config.smoothness, 30);
    
    // Test invalid curvature factor (too low)
    let result = ArcConfig::new(-0.1, 20);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("curvature_factor"));
    
    // Test invalid curvature factor (too high)
    let result = ArcConfig::new(3.0, 20);
    assert!(result.is_err());
    
    // Test invalid smoothness (too low)
    let result = ArcConfig::new(0.5, 2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("smoothness"));
    
    // Test invalid smoothness (too high)
    let result = ArcConfig::new(0.5, 2000);
    assert!(result.is_err());
}

/// Test configuration constants are within expected ranges
#[test]
fn test_configuration_constants() {
    // Test that default values are within valid ranges
    assert!(constants::DEFAULT_WALL_CLEARANCE >= constants::MIN_WALL_CLEARANCE);
    assert!(constants::DEFAULT_WALL_CLEARANCE <= constants::MAX_WALL_CLEARANCE);
    
    assert!(constants::DEFAULT_CHANNEL_WIDTH >= constants::MIN_CHANNEL_WIDTH);
    assert!(constants::DEFAULT_CHANNEL_WIDTH <= constants::MAX_CHANNEL_WIDTH);
    
    assert!(constants::DEFAULT_FILL_FACTOR >= constants::MIN_FILL_FACTOR);
    assert!(constants::DEFAULT_FILL_FACTOR <= constants::MAX_FILL_FACTOR);
    
    assert!(constants::DEFAULT_CURVATURE_FACTOR >= constants::MIN_CURVATURE_FACTOR);
    assert!(constants::DEFAULT_CURVATURE_FACTOR <= constants::MAX_CURVATURE_FACTOR);
    
    assert!(constants::DEFAULT_SMOOTHNESS >= constants::MIN_SMOOTHNESS);
    assert!(constants::DEFAULT_SMOOTHNESS <= constants::MAX_SMOOTHNESS);
    
    // Test strategy thresholds are reasonable
    assert!(constants::strategy_thresholds::LONG_HORIZONTAL_THRESHOLD > 0.0);
    assert!(constants::strategy_thresholds::LONG_HORIZONTAL_THRESHOLD < 1.0);
    
    assert!(constants::strategy_thresholds::MIN_ARC_LENGTH_THRESHOLD > 0.0);
    assert!(constants::strategy_thresholds::MIN_ARC_LENGTH_THRESHOLD < 1.0);
}

/// Test default configurations use constants
#[test]
fn test_default_configurations_use_constants() {
    let geometry_config = GeometryConfig::default();
    assert_eq!(geometry_config.wall_clearance, constants::DEFAULT_WALL_CLEARANCE);
    assert_eq!(geometry_config.channel_width, constants::DEFAULT_CHANNEL_WIDTH);
    assert_eq!(geometry_config.channel_height, constants::DEFAULT_CHANNEL_HEIGHT);
    
    let serpentine_config = SerpentineConfig::default();
    assert_eq!(serpentine_config.fill_factor, constants::DEFAULT_FILL_FACTOR);
    assert_eq!(serpentine_config.wavelength_factor, constants::DEFAULT_WAVELENGTH_FACTOR);
    assert_eq!(serpentine_config.gaussian_width_factor, constants::DEFAULT_GAUSSIAN_WIDTH_FACTOR);
    assert_eq!(serpentine_config.wave_density_factor, constants::DEFAULT_WAVE_DENSITY_FACTOR);
    
    let arc_config = ArcConfig::default();
    assert_eq!(arc_config.curvature_factor, constants::DEFAULT_CURVATURE_FACTOR);
    assert_eq!(arc_config.smoothness, constants::DEFAULT_SMOOTHNESS);
}

/// Test configuration presets
#[test]
fn test_configuration_presets() {
    let fine_features = presets::fine_features();
    assert!(fine_features.wall_clearance < constants::DEFAULT_WALL_CLEARANCE);
    assert!(fine_features.channel_width < constants::DEFAULT_CHANNEL_WIDTH);
    assert!(fine_features.channel_height < constants::DEFAULT_CHANNEL_HEIGHT);
    
    let standard = presets::standard();
    assert_eq!(standard.wall_clearance, constants::DEFAULT_WALL_CLEARANCE);
    assert_eq!(standard.channel_width, constants::DEFAULT_CHANNEL_WIDTH);
    assert_eq!(standard.channel_height, constants::DEFAULT_CHANNEL_HEIGHT);
    
    let large_scale = presets::large_scale();
    assert!(large_scale.wall_clearance > constants::DEFAULT_WALL_CLEARANCE);
    assert!(large_scale.channel_width > constants::DEFAULT_CHANNEL_WIDTH);
    assert!(large_scale.channel_height > constants::DEFAULT_CHANNEL_HEIGHT);
    
    let high_density = presets::high_density_serpentine();
    assert!(high_density.fill_factor > constants::DEFAULT_FILL_FACTOR);
    assert!(high_density.wave_density_factor > constants::DEFAULT_WAVE_DENSITY_FACTOR);
    
    let smooth = presets::smooth_serpentine();
    assert!(smooth.fill_factor < constants::DEFAULT_FILL_FACTOR);
    assert!(smooth.wave_density_factor < constants::DEFAULT_WAVE_DENSITY_FACTOR);
    
    let subtle_arcs = presets::subtle_arcs();
    assert!(subtle_arcs.curvature_factor < constants::DEFAULT_CURVATURE_FACTOR);
    
    let pronounced_arcs = presets::pronounced_arcs();
    assert!(pronounced_arcs.curvature_factor > constants::DEFAULT_CURVATURE_FACTOR);
}

/// Test ChannelTypeConfigBuilder
#[test]
fn test_channel_type_config_builder() {
    let builder = ChannelTypeConfigBuilder::new();
    
    // Test building smart configuration
    let smart_config = builder.build_smart();
    match smart_config {
        ChannelTypeConfig::Smart { serpentine_config, arc_config } => {
            assert_eq!(serpentine_config.fill_factor, constants::DEFAULT_FILL_FACTOR);
            assert_eq!(arc_config.curvature_factor, constants::DEFAULT_CURVATURE_FACTOR);
        },
        _ => panic!("Expected Smart configuration"),
    }
    
    // Test building mixed by position configuration
    let builder = ChannelTypeConfigBuilder::new()
        .with_serpentine_config(presets::high_density_serpentine())
        .with_arc_config(presets::pronounced_arcs())
        .with_middle_zone_fraction(0.6);
    
    let mixed_config = builder.build_mixed_by_position();
    match mixed_config {
        ChannelTypeConfig::MixedByPosition { 
            middle_zone_fraction, 
            serpentine_config, 
            arc_config 
        } => {
            assert_eq!(middle_zone_fraction, 0.6);
            assert!(serpentine_config.fill_factor > constants::DEFAULT_FILL_FACTOR);
            assert!(arc_config.curvature_factor > constants::DEFAULT_CURVATURE_FACTOR);
        },
        _ => panic!("Expected MixedByPosition configuration"),
    }
}

/// Test configuration validation with edge cases
#[test]
fn test_configuration_edge_cases() {
    // Test exactly at boundaries
    let config = GeometryConfig::new(
        constants::MIN_WALL_CLEARANCE,
        constants::MIN_CHANNEL_WIDTH,
        constants::MIN_CHANNEL_HEIGHT,
    );
    assert!(config.is_ok());
    
    let config = GeometryConfig::new(
        constants::MAX_WALL_CLEARANCE,
        constants::MAX_CHANNEL_WIDTH,
        constants::MAX_CHANNEL_HEIGHT,
    );
    assert!(config.is_ok());
    
    // Test just outside boundaries
    let config = GeometryConfig::new(
        constants::MIN_WALL_CLEARANCE - 0.001,
        constants::MIN_CHANNEL_WIDTH,
        constants::MIN_CHANNEL_HEIGHT,
    );
    assert!(config.is_err());
    
    let config = GeometryConfig::new(
        constants::MAX_WALL_CLEARANCE + 0.001,
        constants::MAX_CHANNEL_WIDTH,
        constants::MAX_CHANNEL_HEIGHT,
    );
    assert!(config.is_err());
}

/// Test that validation is called on construction
#[test]
fn test_validation_on_construction() {
    // Test that invalid configurations cannot be created
    let result = SerpentineConfig::new(2.0, 3.0, 6.0, 2.0); // Invalid fill_factor
    assert!(result.is_err());
    
    let result = ArcConfig::new(0.5, 1); // Invalid smoothness
    assert!(result.is_err());
    
    // Test that default configurations are always valid
    let geometry_config = GeometryConfig::default();
    assert!(geometry_config.validate().is_ok());
    
    let serpentine_config = SerpentineConfig::default();
    assert!(serpentine_config.validate().is_ok());
    
    let arc_config = ArcConfig::default();
    assert!(arc_config.validate().is_ok());
}

/// Test configuration error messages are informative
#[test]
fn test_configuration_error_messages() {
    let result = GeometryConfig::new(-1.0, 1.0, 0.5);
    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("wall_clearance"));
    assert!(error_msg.contains("-1"));
    assert!(error_msg.contains("Must be between"));
    assert!(error_msg.contains(&constants::MIN_WALL_CLEARANCE.to_string()));
    assert!(error_msg.contains(&constants::MAX_WALL_CLEARANCE.to_string()));
}
