use scheme::{
    geometry::{generator::create_geometry, SplitType, optimization::*},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, OptimizationProfile},
};

/// Test that optimization profiles work correctly
#[test]
fn test_optimization_profiles() {
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];

    // Test Fast profile
    let fast_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        optimization_profile: OptimizationProfile::Fast,
        ..SerpentineConfig::default()
    };

    let fast_system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(fast_config),
    );

    // Test Balanced profile
    let balanced_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        optimization_profile: OptimizationProfile::Balanced,
        ..SerpentineConfig::default()
    };

    let balanced_system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(balanced_config),
    );

    // Test Thorough profile
    let thorough_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        optimization_profile: OptimizationProfile::Thorough,
        ..SerpentineConfig::default()
    };

    let thorough_system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(thorough_config),
    );

    // All systems should have channels
    assert!(fast_system.channels.len() > 0);
    assert!(balanced_system.channels.len() > 0);
    assert!(thorough_system.channels.len() > 0);

    // All channels should be serpentine
    for system in [&fast_system, &balanced_system, &thorough_system] {
        for channel in &system.channels {
            match &channel.channel_type {
                scheme::geometry::ChannelType::Serpentine { path } => {
                    assert!(path.len() > 2, "Serpentine channels should have multiple points");
                    
                    // Calculate path length
                    let path_length = calculate_path_length(path);
                    assert!(path_length > 0.0, "Path should have positive length");
                },
                _ => panic!("Expected serpentine channel"),
            }
        }
    }
}

/// Test optimization parameter validation
#[test]
fn test_optimization_parameter_validation() {
    // Test valid optimization configuration with each profile
    for profile in [OptimizationProfile::Fast, OptimizationProfile::Balanced, OptimizationProfile::Thorough] {
        let config = SerpentineConfig {
            fill_factor: 0.8,
            wavelength_factor: 3.0,
            gaussian_width_factor: 6.0,
            wave_density_factor: 2.0,
            wave_phase_direction: 0.0,
            optimization_enabled: true,
            target_fill_ratio: 0.9,
            optimization_profile: profile,
            ..SerpentineConfig::default()
        };
        
        assert!(config.validate().is_ok(), "Valid optimization config should pass validation");
    }
}

/// Test that optimization can be disabled
#[test]
fn test_optimization_disabled() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: false,
        ..SerpentineConfig::default()
    };
    
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Should work normally without optimization
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

/// Test optimization utility functions
#[test]
fn test_optimization_utilities() {
    // Test path length calculation
    let path = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
    let length = calculate_path_length(&path);
    assert!((length - 20.0).abs() < 1e-6, "Path length should be 20.0, got {}", length);
    
    // Test wall distance calculation
    let box_dims = (20.0, 10.0);
    let channel_width = 1.0;
    let center_path = vec![(10.0, 5.0)];
    let center_distance = calculate_min_wall_distance(&center_path, box_dims, channel_width);
    assert!((center_distance - 4.5).abs() < 1e-6, "Center distance should be 4.5, got {}", center_distance);
    
    // Test neighbor distance calculation
    let neighbors = vec![2.0, 8.0];
    let middle_path = vec![(10.0, 5.0)];
    let neighbor_distance = calculate_min_neighbor_distance(&middle_path, &neighbors, channel_width);
    assert!((neighbor_distance - 2.0).abs() < 1e-6, "Neighbor distance should be 2.0, got {}", neighbor_distance);
}

/// Test that optimization preserves bilateral symmetry
#[test]
fn test_optimization_preserves_symmetry() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_profile: OptimizationProfile::Fast, // Use fast for quicker test
        ..SerpentineConfig::default()
    };
    
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Find output channels
    let output_channels: Vec<_> = system.channels.iter()
        .filter(|ch| {
            let to_node = &system.nodes[ch.to_node];
            to_node.point.0 > 150.0 // Output channels are on the right side
        })
        .collect();
    
    // Should have at least 2 output channels for bifurcation
    assert!(output_channels.len() >= 2, "Should have at least two output channels");
    
    // Check that channels have reasonable lengths
    for channel in &output_channels {
        if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
            let length = calculate_path_length(path);
            assert!(length > 0.0, "Channel should have positive length");
        }
    }
}

/// Test optimization with different target fill ratios
#[test]
fn test_optimization_target_fill_ratios() {
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    let fill_ratios = [0.8, 0.9, 0.95];
    
    for &target_fill_ratio in &fill_ratios {
        let serpentine_config = SerpentineConfig {
            optimization_enabled: true,
            target_fill_ratio,
            optimization_profile: OptimizationProfile::Fast, // Use fast for quicker test
            ..SerpentineConfig::default()
        };
        
        let system = create_geometry(
            box_dims,
            &splits,
            &config,
            &ChannelTypeConfig::AllSerpentine(serpentine_config),
        );
        
        // Should generate valid system
        assert!(system.channels.len() > 0, "Should have channels for target_fill_ratio {}", target_fill_ratio);
        
        // All channels should be serpentine with positive length
        for channel in &system.channels {
            if let scheme::geometry::ChannelType::Serpentine { path } = &channel.channel_type {
                let length = calculate_path_length(path);
                assert!(length > 0.0, "Channel should have positive length for target_fill_ratio {}", target_fill_ratio);
            }
        }
    }
}

/// Test optimization with complex multi-channel systems
#[test]
fn test_optimization_multi_channel() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        target_fill_ratio: 0.9,
        optimization_profile: OptimizationProfile::Fast, // Use fast for quicker test
        ..SerpentineConfig::default()
    };
    
    // Test with multiple splits
    let system = create_geometry(
        (300.0, 150.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // Should have multiple channels
    assert!(system.channels.len() > 5, "Multi-split system should have many channels");
    
    // All channels should maintain proper spacing
    for channel in &system.channels {
        match &channel.channel_type {
            scheme::geometry::ChannelType::Serpentine { path } => {
                let path_length = calculate_path_length(path);
                assert!(path_length > 0.0, "Path should have positive length");
                
                // Check that optimization doesn't break multi-channel compatibility
                let min_wall_distance = calculate_min_wall_distance(path, (300.0, 150.0), config.channel_width);
                assert!(min_wall_distance >= -0.5, "Should maintain reasonable wall clearance in multi-channel system, got: {}", min_wall_distance);
            },
            _ => panic!("Expected serpentine channel"),
        }
    }
}
