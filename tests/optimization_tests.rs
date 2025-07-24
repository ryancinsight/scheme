use scheme::{
    geometry::{generator::create_geometry, SplitType, optimization::*},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig},
};

/// Test path length calculation utility
#[test]
fn test_path_length_calculation() {
    // Test empty path
    let empty_path = vec![];
    assert_eq!(calculate_path_length(&empty_path), 0.0);
    
    // Test single point
    let single_point = vec![(0.0, 0.0)];
    assert_eq!(calculate_path_length(&single_point), 0.0);
    
    // Test straight line
    let straight_line = vec![(0.0, 0.0), (10.0, 0.0)];
    assert_eq!(calculate_path_length(&straight_line), 10.0);
    
    // Test right triangle
    let triangle = vec![(0.0, 0.0), (3.0, 0.0), (3.0, 4.0)];
    assert_eq!(calculate_path_length(&triangle), 7.0); // 3 + 4
    
    // Test diagonal line
    let diagonal = vec![(0.0, 0.0), (3.0, 4.0)];
    assert_eq!(calculate_path_length(&diagonal), 5.0); // sqrt(3^2 + 4^2)
}

/// Test wall distance calculation
#[test]
fn test_wall_distance_calculation() {
    let box_dims = (20.0, 10.0);
    let channel_width = 1.0;
    
    // Test center point
    let center_path = vec![(10.0, 5.0)];
    let center_distance = calculate_min_wall_distance(&center_path, box_dims, channel_width);
    assert_eq!(center_distance, 4.5); // min(10-0.5, 20-10-0.5, 5-0.5, 10-5-0.5)
    
    // Test point near left wall
    let left_path = vec![(1.0, 5.0)];
    let left_distance = calculate_min_wall_distance(&left_path, box_dims, channel_width);
    assert_eq!(left_distance, 0.5); // 1.0 - 0.5
    
    // Test point near bottom wall
    let bottom_path = vec![(10.0, 1.0)];
    let bottom_distance = calculate_min_wall_distance(&bottom_path, box_dims, channel_width);
    assert_eq!(bottom_distance, 0.5); // 1.0 - 0.5
}

/// Test neighbor distance calculation
#[test]
fn test_neighbor_distance_calculation() {
    let channel_width = 1.0;
    let neighbors = vec![2.0, 8.0];
    
    // Test path between neighbors
    let middle_path = vec![(10.0, 5.0)];
    let middle_distance = calculate_min_neighbor_distance(&middle_path, &neighbors, channel_width);
    assert_eq!(middle_distance, 2.0); // min(|5-2|-1, |5-8|-1) = min(2, 2)
    
    // Test path close to neighbor
    let close_path = vec![(10.0, 2.5)];
    let close_distance = calculate_min_neighbor_distance(&close_path, &neighbors, channel_width);
    assert_eq!(close_distance, -0.5); // |2.5-2|-1 = -0.5 (overlap)
    
    // Test no neighbors
    let no_neighbors = vec![];
    let no_neighbor_distance = calculate_min_neighbor_distance(&middle_path, &no_neighbors, channel_width);
    assert_eq!(no_neighbor_distance, f64::INFINITY);
}

/// Test optimization with disabled optimization (backward compatibility)
#[test]
fn test_optimization_disabled() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: false,
        ..SerpentineConfig::default()
    };
    
    // Create system with optimization disabled
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Should work normally without optimization
    // Note: The actual number of channels depends on the geometry generation algorithm
    assert!(system.channels.len() > 0, "Should have at least one channel");
    
    // All channels should be serpentine
    for channel in &system.channels {
        match &channel.channel_type {
            scheme::geometry::ChannelType::Serpentine { path } => {
                assert!(path.len() > 2, "Serpentine channels should have multiple points");
            },
            _ => panic!("Expected serpentine channel"),
        }
    }
}

/// Test optimization with enabled optimization
#[test]
fn test_optimization_enabled() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        ..SerpentineConfig::default()
    };
    
    // Create system with optimization enabled
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Should work with optimization
    assert!(system.channels.len() > 0, "Should have at least one channel");
    
    // All channels should be serpentine
    for channel in &system.channels {
        match &channel.channel_type {
            scheme::geometry::ChannelType::Serpentine { path } => {
                assert!(path.len() > 2, "Serpentine channels should have multiple points");
                
                // Calculate path length
                let path_length = calculate_path_length(path);
                assert!(path_length > 0.0, "Path should have positive length");
                
                // Check wall clearance - optimization should not violate basic constraints
                let min_wall_distance = calculate_min_wall_distance(path, (200.0, 100.0), config.channel_width);
                // If optimization fails to find valid parameters, it should fall back to original parameters
                // which should maintain clearance, so we allow some tolerance but not major violations
                assert!(min_wall_distance >= -1.0, "Should maintain reasonable wall clearance, got: {}", min_wall_distance);
            },
            _ => panic!("Expected serpentine channel"),
        }
    }
}

/// Test optimization parameter validation
#[test]
fn test_optimization_parameter_validation() {
    // Test valid optimization configuration
    let valid_config = SerpentineConfig::new_with_optimization(
        0.8, 3.0, 6.0, 2.0, 0.9
    );
    assert!(valid_config.is_ok());
    
    // Test invalid target_fill_ratio (too low)
    let invalid_low = SerpentineConfig::new_with_optimization(
        0.8, 3.0, 6.0, 2.0, 0.7
    );
    assert!(invalid_low.is_err());
    
    // Test invalid target_fill_ratio (too high)
    let invalid_high = SerpentineConfig::new_with_optimization(
        0.8, 3.0, 6.0, 2.0, 1.0
    );
    assert!(invalid_high.is_err());
}

/// Test optimization with multi-channel systems
#[test]
fn test_optimization_multi_channel() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        ..SerpentineConfig::default()
    };
    
    // Create system with multiple splits
    let system = create_geometry(
        (400.0, 200.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Should have multiple channels for multi-split system
    assert!(system.channels.len() > 5, "Multi-split system should have many channels");
    
    // All channels should maintain proper spacing
    for channel in &system.channels {
        match &channel.channel_type {
            scheme::geometry::ChannelType::Serpentine { path } => {
                let path_length = calculate_path_length(path);
                assert!(path_length > 0.0, "Path should have positive length");
                
                // Check that optimization doesn't break multi-channel compatibility
                let min_wall_distance = calculate_min_wall_distance(path, (400.0, 200.0), config.channel_width);
                assert!(min_wall_distance >= -1.0, "Should maintain reasonable wall clearance in multi-channel system, got: {}", min_wall_distance);
            },
            _ => panic!("Expected serpentine channel"),
        }
    }
}

/// Test optimization preset configuration
#[test]
fn test_optimization_preset() {
    use scheme::config::presets;
    
    let optimized_config = presets::optimized_serpentine();
    assert!(optimized_config.optimization_enabled);
    assert_eq!(optimized_config.target_fill_ratio, 0.95);
    
    // Test that preset works in geometry generation
    let config = GeometryConfig::default();
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(optimized_config),
    );
    
    assert!(system.channels.len() > 0, "Should have at least one channel");
}

/// Test that optimization maintains bilateral symmetry
#[test]
fn test_optimization_maintains_symmetry() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        wave_phase_direction: 0.0, // Auto-symmetric
        ..SerpentineConfig::default()
    };
    
    // Create system with bifurcation to test symmetry
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Find output channels (channels that end near the right side)
    let output_channels: Vec<_> = system.channels.iter()
        .filter(|ch| {
            let to_node = &system.nodes[ch.to_node];
            to_node.point.0 > 150.0 // Output channels are on the right side
        })
        .collect();

    // Should have at least 2 output channels for bifurcation
    assert!(output_channels.len() >= 2, "Should have at least two output channels, found {}", output_channels.len());
    
    // Check that at least the first two output channels have similar path lengths (within tolerance)
    if output_channels.len() >= 2 {
        if let (
            scheme::geometry::ChannelType::Serpentine { path: path1 },
            scheme::geometry::ChannelType::Serpentine { path: path2 }
        ) = (&output_channels[0].channel_type, &output_channels[1].channel_type) {
            let length1 = calculate_path_length(path1);
            let length2 = calculate_path_length(path2);
            let length_diff = (length1 - length2).abs();
            let avg_length = (length1 + length2) / 2.0;

            if avg_length > 0.0 {
                let relative_diff = length_diff / avg_length;
                assert!(relative_diff < 0.2, "Symmetric channels should have similar lengths: {} vs {} (diff: {:.1}%)",
                    length1, length2, relative_diff * 100.0);
            }
        }
    }
}
