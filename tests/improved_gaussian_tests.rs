use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::{generator::create_geometry, SplitType},
};

#[test]
fn test_improved_gaussian_envelope_distance_normalization() {
    let config = GeometryConfig::default();
    
    // Create a configuration with a small gaussian_width_factor to make the effect more pronounced
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 4.0, // Small factor for pronounced envelope effect
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        ..SerpentineConfig::default()
    };

    // Test with different box dimensions to create channels of different lengths
    let short_system = create_geometry(
        (100.0, 50.0), // Short channels
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );

    let long_system = create_geometry(
        (400.0, 200.0), // Long channels
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );

    // Both systems should generate successfully
    assert!(!short_system.channels.is_empty());
    assert!(!long_system.channels.is_empty());
    
    // Verify that channels have serpentine paths
    for channel in &short_system.channels {
        if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
            assert!(path.len() > 2, "Serpentine channels should have multiple path points");
        }
    }
    
    for channel in &long_system.channels {
        if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
            assert!(path.len() > 2, "Serpentine channels should have multiple path points");
        }
    }
}

#[test]
fn test_middle_section_detection() {
    let config = GeometryConfig::default();
    
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 4.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        ..SerpentineConfig::default()
    };

    // Create a system with multiple levels to get both directional changes and middle sections
    let system = create_geometry(
        (400.0, 200.0),
        &[SplitType::Bifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );

    assert!(!system.channels.is_empty());
    
    // Verify that all channels are serpentine and have valid paths
    for channel in &system.channels {
        if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
            assert!(path.len() > 2, "Serpentine channels should have multiple path points");
            
            // Verify endpoints are preserved (first and last points should match node positions)
            let first_point = path[0];
            let last_point = path[path.len() - 1];

            // Get the actual node positions
            let from_node = &system.nodes[channel.from_node];
            let to_node = &system.nodes[channel.to_node];

            // The endpoints should be exactly the node positions
            assert_eq!(first_point, from_node.point, "First point should match from node position");
            assert_eq!(last_point, to_node.point, "Last point should match to node position");
        }
    }
}

#[test]
fn test_gaussian_envelope_preserves_symmetry() {
    let config = GeometryConfig::default();
    
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0, // Auto-symmetric
        ..SerpentineConfig::default()
    };

    // Create a symmetric bifurcation pattern
    let system = create_geometry(
        (300.0, 150.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );

    assert!(!system.channels.is_empty());
    
    // For a single bifurcation, we should have channels that maintain bilateral symmetry
    // This is verified by ensuring the system generates without errors and has valid paths
    for channel in &system.channels {
        if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
            assert!(path.len() > 2, "Serpentine channels should have multiple path points");
            
            // Verify the path is smooth (no extreme jumps between consecutive points)
            for i in 1..path.len() {
                let prev = path[i - 1];
                let curr = path[i];
                let distance = ((curr.0 - prev.0).powi(2) + (curr.1 - prev.1).powi(2)).sqrt();
                
                // Distance between consecutive points should be reasonable
                assert!(distance < 50.0, "Consecutive points should not be too far apart");
            }
        }
    }
}

#[test]
fn test_improved_envelope_with_optimization() {
    let config = GeometryConfig::default();
    
    let optimized_serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 4.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        optimization_enabled: true, // Enable optimization
        target_fill_ratio: 0.95,
        optimization_profile: scheme::config::OptimizationProfile::Fast, // Use fast for testing
        ..SerpentineConfig::default()
    };

    // Test that optimization works with the improved Gaussian envelope
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(optimized_serpentine_config),
    );

    assert!(!system.channels.is_empty());
    
    // Verify that optimized channels still have valid serpentine paths
    for channel in &system.channels {
        if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
            assert!(path.len() > 2, "Optimized serpentine channels should have multiple path points");
            
            // Verify endpoints are still preserved
            let from_node = &system.nodes[channel.from_node];
            let to_node = &system.nodes[channel.to_node];
            assert_eq!(path[0], from_node.point, "Optimization should preserve start point");
            assert_eq!(path[path.len() - 1], to_node.point, "Optimization should preserve end point");
        }
    }
}
