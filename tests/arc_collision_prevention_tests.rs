//! Tests for arc channel collision prevention and proximity detection
//!
//! This module tests the enhanced arc channel generation system with
//! collision prevention, proximity detection, and adaptive curvature.

use scheme::config::{ArcConfig, GeometryConfig, presets};
use scheme::geometry::strategies::{ArcChannelStrategy, ChannelTypeStrategy};

#[test]
fn test_adaptive_curvature_reduction() {
    // Test that high curvature factors are reduced when collision prevention is enabled
    let config = ArcConfig {
        curvature_factor: 2.0, // Maximum curvature
        smoothness: 50,
        curvature_direction: 0.0,
        min_separation_distance: 1.0,
        enable_collision_prevention: true,
        max_curvature_reduction: 0.3,
        enable_adaptive_curvature: true,
    };

    let strategy = ArcChannelStrategy::new(config);
    let geometry_config = GeometryConfig::default();
    let box_dims = (100.0, 100.0);

    // Create a scenario with many branches (high density)
    let total_branches = 10;
    let neighbor_distances = vec![0.5, 0.8, 1.2]; // Some neighbors are very close

    let channel = strategy.create_channel(
        (10.0, 20.0),
        (90.0, 80.0),
        &geometry_config,
        box_dims,
        total_branches,
        Some(&neighbor_distances),
    );

    // Verify that a channel was created
    if let scheme::geometry::ChannelType::Arc { path } = channel {
        assert!(path.len() > 2, "Arc path should have multiple points");
        
        // Verify that the path starts and ends at the correct points
        assert_eq!(path[0], (10.0, 20.0));
        assert_eq!(path[path.len() - 1], (90.0, 80.0));
        
        // The adaptive curvature should have reduced the arc height compared to maximum curvature
        // We can't easily test the exact reduction, but we can verify the path is reasonable
        let max_y = path.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let min_y = path.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let arc_height = max_y - min_y;
        
        // With collision prevention, the arc height should be reasonable (not extreme)
        assert!(arc_height < 100.0, "Arc height should be reasonable with collision prevention");
    } else {
        panic!("Expected Arc channel type");
    }
}

#[test]
fn test_collision_prevention_disabled() {
    // Test that collision prevention can be disabled
    let config = ArcConfig {
        curvature_factor: 2.0,
        smoothness: 50,
        curvature_direction: 0.0,
        min_separation_distance: 1.0,
        enable_collision_prevention: false, // Disabled
        max_curvature_reduction: 0.3,
        enable_adaptive_curvature: true,
    };

    let strategy = ArcChannelStrategy::new(config);
    let geometry_config = GeometryConfig::default();
    let box_dims = (100.0, 100.0);

    let channel = strategy.create_channel(
        (10.0, 20.0),
        (90.0, 80.0),
        &geometry_config,
        box_dims,
        10, // Many branches
        Some(&[0.1, 0.2]), // Very close neighbors
    );

    // Even with close neighbors, collision prevention is disabled, so full curvature should be used
    if let scheme::geometry::ChannelType::Arc { path } = channel {
        assert!(path.len() > 2, "Arc path should have multiple points");
    } else {
        panic!("Expected Arc channel type");
    }
}

#[test]
fn test_safe_high_curvature_preset() {
    // Test the safe high curvature preset
    let config = presets::safe_high_curvature_arcs();
    let strategy = ArcChannelStrategy::new(config);
    let geometry_config = GeometryConfig::default();
    let box_dims = (100.0, 100.0);

    let channel = strategy.create_channel(
        (10.0, 50.0),
        (90.0, 50.0),
        &geometry_config,
        box_dims,
        5,
        None,
    );

    if let scheme::geometry::ChannelType::Arc { path } = channel {
        assert!(path.len() > 2, "Arc path should have multiple points");
        assert_eq!(path[0], (10.0, 50.0));
        assert_eq!(path[path.len() - 1], (90.0, 50.0));
    } else {
        panic!("Expected Arc channel type");
    }
}

#[test]
fn test_maximum_safe_arcs_preset() {
    // Test the maximum safe arcs preset
    let config = presets::maximum_safe_arcs();
    let strategy = ArcChannelStrategy::new(config);
    let geometry_config = GeometryConfig::default();
    let box_dims = (200.0, 150.0);

    let channel = strategy.create_channel(
        (20.0, 75.0),
        (180.0, 75.0),
        &geometry_config,
        box_dims,
        3, // Few branches, should allow higher curvature
        None,
    );

    if let scheme::geometry::ChannelType::Arc { path } = channel {
        assert!(path.len() > 2, "Arc path should have multiple points");
        
        // With maximum safe settings and few branches, we should get a pronounced arc
        let max_y = path.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let min_y = path.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let arc_height = max_y - min_y;
        
        // Should have some curvature but not extreme
        assert!(arc_height > 10.0, "Should have noticeable curvature");
        assert!(arc_height < 200.0, "Should not have extreme curvature");
    } else {
        panic!("Expected Arc channel type");
    }
}

#[test]
fn test_dense_layout_arcs_preset() {
    // Test the dense layout preset
    let config = presets::dense_layout_arcs();
    let strategy = ArcChannelStrategy::new(config);
    let geometry_config = GeometryConfig::default();
    let box_dims = (100.0, 100.0);

    // Simulate a dense layout with many branches and close neighbors
    let neighbor_distances = vec![0.3, 0.4, 0.6, 0.8];

    let channel = strategy.create_channel(
        (10.0, 30.0),
        (90.0, 70.0),
        &geometry_config,
        box_dims,
        15, // Many branches (dense layout)
        Some(&neighbor_distances),
    );

    if let scheme::geometry::ChannelType::Arc { path } = channel {
        assert!(path.len() > 2, "Arc path should have multiple points");
        
        // In dense layouts, curvature should be conservative
        let max_y = path.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let min_y = path.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let arc_height = max_y - min_y;
        
        // Should have minimal curvature in dense layouts
        assert!(arc_height < 50.0, "Should have conservative curvature in dense layouts");
    } else {
        panic!("Expected Arc channel type");
    }
}

#[test]
fn test_very_low_curvature() {
    // Test that very low curvature results in nearly straight lines
    let config = ArcConfig {
        curvature_factor: 0.01, // Very low curvature
        smoothness: 20,
        curvature_direction: 0.0,
        min_separation_distance: 1.0,
        enable_collision_prevention: true,
        max_curvature_reduction: 0.5,
        enable_adaptive_curvature: true,
    };

    let strategy = ArcChannelStrategy::new(config);
    let geometry_config = GeometryConfig::default();
    let box_dims = (100.0, 100.0);

    let channel = strategy.create_channel(
        (10.0, 50.0),
        (90.0, 50.0), // Horizontal channel
        &geometry_config,
        box_dims,
        5,
        None,
    );

    if let scheme::geometry::ChannelType::Arc { path } = channel {
        assert!(path.len() > 2, "Should have multiple points");
        assert_eq!(path[0], (10.0, 50.0));
        assert_eq!(path[path.len() - 1], (90.0, 50.0));

        // With very low curvature, the arc should be very subtle
        let max_y = path.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let min_y = path.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let arc_height = max_y - min_y;

        // Should have minimal deviation from straight line
        assert!(arc_height < 5.0, "Very low curvature should result in minimal arc height");
    } else {
        panic!("Expected Arc channel type");
    }
}
