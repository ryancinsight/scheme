//! Wave Shape Tests
//! 
//! Comprehensive tests for the wave shape functionality in serpentine channels.
//! Tests both sine and square wave generation, convenience methods, and integration
//! with existing systems.

use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig, WaveShape},
    geometry::{generator::create_geometry, SplitType, ChannelType},
};

/// Test that WaveShape enum has correct default
#[test]
fn test_wave_shape_default() {
    assert_eq!(WaveShape::default(), WaveShape::Sine);
}

/// Test that SerpentineConfig uses sine wave by default
#[test]
fn test_serpentine_config_default_wave_shape() {
    let config = SerpentineConfig::default();
    assert_eq!(config.wave_shape, WaveShape::Sine);
}

/// Test wave shape convenience methods
#[test]
fn test_wave_shape_convenience_methods() {
    let base_config = SerpentineConfig::default();
    
    // Test with_sine_wave method
    let sine_config = base_config.with_sine_wave();
    assert_eq!(sine_config.wave_shape, WaveShape::Sine);
    assert_eq!(sine_config.fill_factor, base_config.fill_factor); // Other fields preserved
    
    // Test with_square_wave method
    let square_config = base_config.with_square_wave();
    assert_eq!(square_config.wave_shape, WaveShape::Square);
    assert_eq!(square_config.fill_factor, base_config.fill_factor); // Other fields preserved
    
    // Test with_wave_shape method
    let explicit_sine = base_config.with_wave_shape(WaveShape::Sine);
    assert_eq!(explicit_sine.wave_shape, WaveShape::Sine);
    
    let explicit_square = base_config.with_wave_shape(WaveShape::Square);
    assert_eq!(explicit_square.wave_shape, WaveShape::Square);
}

/// Test that both wave shapes generate valid geometry
#[test]
fn test_wave_shapes_generate_valid_geometry() {
    let config = GeometryConfig::default();
    let splits = vec![SplitType::Bifurcation];
    
    // Test sine wave generation
    let sine_config = SerpentineConfig::default().with_sine_wave();
    let sine_system = create_geometry(
        (200.0, 100.0),
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(sine_config),
    );
    
    assert!(!sine_system.channels.is_empty());
    assert!(!sine_system.nodes.is_empty());
    
    // Verify channels are serpentine type
    for channel in &sine_system.channels {
        match &channel.channel_type {
            ChannelType::Serpentine { path } => {
                assert!(!path.is_empty());
                assert!(path.len() >= 2); // At least start and end points
            }
            _ => panic!("Expected serpentine channel"),
        }
    }
    
    // Test square wave generation
    let square_config = SerpentineConfig::default().with_square_wave();
    let square_system = create_geometry(
        (200.0, 100.0),
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(square_config),
    );
    
    assert!(!square_system.channels.is_empty());
    assert!(!square_system.nodes.is_empty());
    
    // Verify channels are serpentine type
    for channel in &square_system.channels {
        match &channel.channel_type {
            ChannelType::Serpentine { path } => {
                assert!(!path.is_empty());
                assert!(path.len() >= 2); // At least start and end points
            }
            _ => panic!("Expected serpentine channel"),
        }
    }
}

/// Test wave shape with different configurations
#[test]
fn test_wave_shapes_with_different_configs() {
    let config = GeometryConfig::default();
    
    // Test with high-density configuration
    let high_density_sine = SerpentineConfig {
        fill_factor: 0.9,
        wavelength_factor: 2.0,
        gaussian_width_factor: 8.0,
        wave_density_factor: 4.0,
        ..SerpentineConfig::default()
    }.with_sine_wave();
    
    let high_density_square = SerpentineConfig {
        fill_factor: 0.9,
        wavelength_factor: 2.0,
        gaussian_width_factor: 8.0,
        wave_density_factor: 4.0,
        ..SerpentineConfig::default()
    }.with_square_wave();
    
    // Both should generate valid systems
    let sine_system = create_geometry(
        (150.0, 75.0),
        &[SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(high_density_sine),
    );
    
    let square_system = create_geometry(
        (150.0, 75.0),
        &[SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(high_density_square),
    );
    
    assert!(!sine_system.channels.is_empty());
    assert!(!square_system.channels.is_empty());
    
    // Both systems should have the same number of channels and nodes
    assert_eq!(sine_system.channels.len(), square_system.channels.len());
    assert_eq!(sine_system.nodes.len(), square_system.nodes.len());
}

/// Test wave shape with optimization enabled
#[test]
fn test_wave_shapes_with_optimization() {
    let config = GeometryConfig::default();
    
    let optimized_sine = SerpentineConfig {
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Fast,
        ..SerpentineConfig::default()
    }.with_sine_wave();
    
    let optimized_square = SerpentineConfig {
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Fast,
        ..SerpentineConfig::default()
    }.with_square_wave();
    
    // Both should work with optimization
    let sine_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(optimized_sine),
    );
    
    let square_system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(optimized_square),
    );
    
    assert!(!sine_system.channels.is_empty());
    assert!(!square_system.channels.is_empty());
}

/// Test that wave shape is preserved through configuration methods
#[test]
fn test_wave_shape_preservation() {
    let base_config = SerpentineConfig::default().with_square_wave();
    
    // Test that validation preserves wave shape
    let validated_config = SerpentineConfig {
        fill_factor: 0.7,
        wavelength_factor: 4.0,
        ..base_config
    };
    
    assert_eq!(validated_config.wave_shape, WaveShape::Square);
    assert!(validated_config.validate().is_ok());
}

/// Test wave shape with different phase directions
#[test]
fn test_wave_shapes_with_phase_directions() {
    let config = GeometryConfig::default();
    
    // Test inward phase with both wave shapes
    let inward_sine = SerpentineConfig {
        wave_phase_direction: -1.0,
        ..SerpentineConfig::default()
    }.with_sine_wave();
    
    let inward_square = SerpentineConfig {
        wave_phase_direction: -1.0,
        ..SerpentineConfig::default()
    }.with_square_wave();
    
    // Test outward phase with both wave shapes
    let outward_sine = SerpentineConfig {
        wave_phase_direction: 1.0,
        ..SerpentineConfig::default()
    }.with_sine_wave();
    
    let outward_square = SerpentineConfig {
        wave_phase_direction: 1.0,
        ..SerpentineConfig::default()
    }.with_square_wave();
    
    // All configurations should generate valid systems
    for (name, serpentine_config) in [
        ("inward_sine", inward_sine),
        ("inward_square", inward_square),
        ("outward_sine", outward_sine),
        ("outward_square", outward_square),
    ] {
        let system = create_geometry(
            (120.0, 60.0),
            &[SplitType::Bifurcation],
            &config,
            &ChannelTypeConfig::AllSerpentine(serpentine_config),
        );

        assert!(!system.channels.is_empty(), "Failed for configuration: {}", name);
        assert!(!system.nodes.is_empty(), "Failed for configuration: {}", name);
    }
}
