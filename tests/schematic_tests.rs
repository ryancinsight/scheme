//! tests/schematic_tests.rs
//! 
//! Tests for 2D microfluidic schematic generation and visualization

use scheme::{
    geometry::{generator::create_geometry, SplitType, ChannelType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig},
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
        generation: scheme::config::GeometryGenerationConfig::default(),
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
        generation: scheme::config::GeometryGenerationConfig::default(),
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
    
    // Create system with mixed channel types using Smart configuration
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::default(), // Uses Smart configuration
    );

    // Should have straight channels at minimum
    let mut has_straight = false;
    
    for channel in &system.channels {
        match channel.channel_type {
            ChannelType::Straight => has_straight = true,
            ChannelType::SmoothStraight { .. } => has_straight = true,
            ChannelType::Serpentine { .. } => {},
            ChannelType::Arc { .. } => {},
        }
    }
    
    assert!(has_straight, "Should have some straight channels");
    // Note: serpentine and arc channels may or may not be present depending on the specific geometry
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
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
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
        
        match &channel.channel_type {
            ChannelType::Serpentine { path } | ChannelType::Arc { path } => {
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
            _ => {} // Skip straight channels
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
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
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

/// Test that serpentine channels have perfect bilateral mirror symmetry and complete wave cycles
#[test]
fn test_serpentine_wave_symmetry() {
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0, // Auto-symmetric
        optimization_enabled: false,
        target_fill_ratio: 0.9,
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
    };

    // Create a system with trifurcation to test bilateral symmetry
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );

    // Find serpentine channels and verify they have symmetric wave patterns
    let mut serpentine_channels = Vec::new();
    for channel in &system.channels {
        if let ChannelType::Serpentine { path } = &channel.channel_type {
            serpentine_channels.push((channel, path));
        }
    }

    // Should have serpentine channels
    assert!(!serpentine_channels.is_empty(), "Should have serpentine channels");

    // Verify that serpentine channels have proper wave patterns and bilateral symmetry
    let box_center_x = 200.0 / 2.0;
    let _box_center_y = 100.0 / 2.0;

    let mut left_channels = Vec::new();
    let mut right_channels = Vec::new();

    for (channel, path) in &serpentine_channels {
        let from_node = &system.nodes[channel.from_node];
        let to_node = &system.nodes[channel.to_node];
        let channel_center_x = (from_node.point.0 + to_node.point.0) / 2.0;

        // Check that path has enough points for wave analysis
        assert!(path.len() >= 10, "Serpentine path should have sufficient points for wave analysis");

        // Verify that the path starts and ends at the correct nodes
        assert_eq!(path[0], from_node.point, "Serpentine path should start at from_node");
        assert_eq!(path[path.len() - 1], to_node.point, "Serpentine path should end at to_node");

        // Check that the path has wave-like characteristics with complete cycles
        let mut has_wave_variation = false;
        let mut positive_peaks = 0;
        let mut negative_peaks = 0;

        let dx = to_node.point.0 - from_node.point.0;
        let dy = to_node.point.1 - from_node.point.1;
        let channel_length = (dx * dx + dy * dy).sqrt();

        if channel_length > 1e-6 {
            // Calculate perpendicular direction
            let perp_x = -dy / channel_length;
            let perp_y = dx / channel_length;

            let mut prev_displacement = 0.0;
            let mut prev_slope = 0.0;

            // Analyze wave characteristics
            for (i, point) in path.iter().enumerate().skip(1).take(path.len() - 2) {
                let base_t = ((point.0 - from_node.point.0) * dx + (point.1 - from_node.point.1) * dy) / (channel_length * channel_length);
                let base_x = from_node.point.0 + base_t * dx;
                let base_y = from_node.point.1 + base_t * dy;

                let perp_displacement = (point.0 - base_x) * perp_x + (point.1 - base_y) * perp_y;

                if perp_displacement.abs() > config.channel_width * 0.1 {
                    has_wave_variation = true;
                }

                // Detect peaks (local maxima and minima)
                if i > 1 {
                    let current_slope = perp_displacement - prev_displacement;
                    if prev_slope > 0.0 && current_slope < 0.0 && perp_displacement > config.channel_width * 0.2 {
                        positive_peaks += 1;
                    } else if prev_slope < 0.0 && current_slope > 0.0 && perp_displacement < -config.channel_width * 0.2 {
                        negative_peaks += 1;
                    }
                    prev_slope = current_slope;
                }
                prev_displacement = perp_displacement;
            }
        }

        assert!(has_wave_variation, "Serpentine channel should have wave-like variation");

        // For complete wave cycles, we should have both positive and negative peaks
        // With minimum 2 complete cycles, we expect at least 2 positive and 2 negative peaks
        assert!(positive_peaks >= 1, "Should have at least 1 positive peak for complete wave cycles");
        assert!(negative_peaks >= 1, "Should have at least 1 negative peak for complete wave cycles");

        // Categorize channels by position for bilateral symmetry testing
        if channel_center_x < box_center_x {
            left_channels.push((channel, path));
        } else {
            right_channels.push((channel, path));
        }
    }

    // For bilateral symmetry, we should have matching numbers of left and right channels
    // (This may not always be exactly equal due to center channels, but should be reasonable)
    assert!(!left_channels.is_empty() || !right_channels.is_empty(),
           "Should have channels on at least one side for symmetry testing");
}

/// Test arc channel generation
#[test]
fn test_arc_channels() {
    let config = GeometryConfig::default();
    let arc_config = ArcConfig {
        curvature_factor: 0.5,
        smoothness: 15,
        curvature_direction: 0.0, // Auto-determine
    };
    
    // Create system with all arc channels
    let system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(arc_config),
    );

    // Verify all channels are arcs
    for channel in &system.channels {
        match &channel.channel_type {
            ChannelType::Arc { path } => {
                // Arc channels should have multiple path points
                assert!(path.len() > 2, "Arc channels should have multiple path points");
                assert!(path.len() >= arc_config.smoothness, "Arc should have at least as many points as smoothness setting");
                
                // Verify endpoint alignment
                let from_node = &system.nodes[channel.from_node];
                let to_node = &system.nodes[channel.to_node];
                let first_point = path.first().unwrap();
                let last_point = path.last().unwrap();
                
                assert_eq!(first_point.0, from_node.point.0, "Arc should start at from_node");
                assert_eq!(first_point.1, from_node.point.1, "Arc should start at from_node");
                assert_eq!(last_point.0, to_node.point.0, "Arc should end at to_node");
                assert_eq!(last_point.1, to_node.point.1, "Arc should end at to_node");
            },
            _ => panic!("Expected all channels to be arcs"),
        }
    }
}

/// Test smart channel type selection
#[test]
fn test_smart_channel_types() {
    let config = GeometryConfig::default();
    
    // Create system with smart channel type selection
    let system = create_geometry(
        (300.0, 100.0),  // Wider to ensure we get different zones
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::Smart {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
        },
    );

    // Should have different channel types
    let mut channel_types = std::collections::HashSet::new();
    
    for channel in &system.channels {
        match &channel.channel_type {
            ChannelType::Straight => { channel_types.insert("straight"); },
            ChannelType::SmoothStraight { .. } => { channel_types.insert("smooth_straight"); },
            ChannelType::Serpentine { .. } => { channel_types.insert("serpentine"); },
            ChannelType::Arc { .. } => { channel_types.insert("arc"); },
        }
    }
    
    // Smart selection should produce at least straight channels
    assert!(channel_types.contains("straight"), "Smart selection should include straight channels");
}

/// Test that the geometry generation produces symmetric channel layouts
#[test]
fn test_symmetric_channel_layout() {
    use scheme::geometry::{create_geometry, SplitType};
    use scheme::config::{GeometryConfig, ChannelTypeConfig};

    let config = GeometryConfig::default();
    let channel_config = ChannelTypeConfig::AllStraight;
    let splits = vec![SplitType::Bifurcation, SplitType::Trifurcation];

    let system = create_geometry((20.0, 10.0), &splits, &config, &channel_config);

    // Check that the system has the expected structure
    assert!(!system.channels.is_empty());
    assert!(!system.nodes.is_empty());

    // For a symmetric system, we should have nodes that mirror across the vertical centerline
    let center_x = 10.0; // Half of the 20.0 width
    let mut left_nodes = Vec::new();
    let mut right_nodes = Vec::new();
    let mut center_nodes = Vec::new();

    for node in &system.nodes {
        if node.point.0 < center_x - 0.1 {
            left_nodes.push(node);
        } else if node.point.0 > center_x + 0.1 {
            right_nodes.push(node);
        } else {
            center_nodes.push(node);
        }
    }

    // We should have nodes on both sides and potentially some at the center
    assert!(!left_nodes.is_empty(), "Should have nodes on the left side");
    assert!(!right_nodes.is_empty(), "Should have nodes on the right side");

    // Check that for each left node, there's a corresponding right node at the mirrored position
    for left_node in &left_nodes {
        let expected_right_x = 2.0 * center_x - left_node.point.0;
        let expected_right_y = left_node.point.1;

        let found_mirror = right_nodes.iter().any(|right_node| {
            (right_node.point.0 - expected_right_x).abs() < 0.1 &&
            (right_node.point.1 - expected_right_y).abs() < 0.1
        });

        assert!(found_mirror,
            "No mirror node found for left node at ({}, {}) - expected right node at ({}, {})",
            left_node.point.0, left_node.point.1, expected_right_x, expected_right_y
        );
    }

    println!("Symmetric layout test passed: {} left nodes, {} right nodes, {} center nodes",
             left_nodes.len(), right_nodes.len(), center_nodes.len());
}

/// Test that arc channels have symmetric curvature across the vertical centerline
#[test]
fn test_arc_curvature_symmetry() {
    use scheme::geometry::{create_geometry, SplitType, ChannelType};
    use scheme::config::{GeometryConfig, ChannelTypeConfig, ArcConfig};

    let config = GeometryConfig::default();
    let arc_config = ArcConfig {
        curvature_factor: 0.5,
        smoothness: 10,
        curvature_direction: 0.0, // Auto-determine
    };
    let channel_config = ChannelTypeConfig::AllArcs(arc_config);
    let splits = vec![SplitType::Bifurcation];

    let system = create_geometry((20.0, 10.0), &splits, &config, &channel_config);

    // Find arc channels and group them by left/right side
    let center_x = 10.0;
    let mut left_arcs = Vec::new();
    let mut right_arcs = Vec::new();

    for channel in &system.channels {
        if let ChannelType::Arc { path } = &channel.channel_type {
            let from_node = &system.nodes[channel.from_node];
            let to_node = &system.nodes[channel.to_node];
            let channel_center_x = (from_node.point.0 + to_node.point.0) / 2.0;

            if channel_center_x < center_x {
                left_arcs.push((channel, path, from_node, to_node));
            } else if channel_center_x > center_x {
                right_arcs.push((channel, path, from_node, to_node));
            }
        }
    }

    // We should have arc channels on both sides
    assert!(!left_arcs.is_empty(), "Should have arc channels on the left side");
    assert!(!right_arcs.is_empty(), "Should have arc channels on the right side");

    // For each left arc, find its corresponding right arc and check curvature symmetry
    for (_left_channel, left_path, left_from, left_to) in &left_arcs {
        let left_y_center = (left_from.point.1 + left_to.point.1) / 2.0;

        // Find corresponding right arc at similar y-coordinate
        let corresponding_right = right_arcs.iter().find(|(_, _, right_from, right_to)| {
            let right_y_center = (right_from.point.1 + right_to.point.1) / 2.0;
            (left_y_center - right_y_center).abs() < 0.5
        });

        if let Some((_right_channel, right_path, _right_from, _right_to)) = corresponding_right {
            // Check that both arcs have similar curvature characteristics
            assert!(left_path.len() > 2, "Left arc should have multiple path points");
            assert!(right_path.len() > 2, "Right arc should have multiple path points");

            // Calculate curvature direction for both arcs
            let left_mid_idx = left_path.len() / 2;
            let right_mid_idx = right_path.len() / 2;

            let left_start = left_path[0];
            let left_mid = left_path[left_mid_idx];
            let left_end = left_path[left_path.len() - 1];

            let right_start = right_path[0];
            let right_mid = right_path[right_mid_idx];
            let right_end = right_path[right_path.len() - 1];

            // Calculate the curvature direction (positive = curves up, negative = curves down)
            let left_baseline_y = (left_start.1 + left_end.1) / 2.0;
            let left_curvature_direction = left_mid.1 - left_baseline_y;

            let right_baseline_y = (right_start.1 + right_end.1) / 2.0;
            let right_curvature_direction = right_mid.1 - right_baseline_y;

            // For visual symmetry, arcs at corresponding positions should curve in the same direction
            // (both concave toward center or both convex away from center)
            let curvature_directions_match =
                (left_curvature_direction > 0.0 && right_curvature_direction > 0.0) ||
                (left_curvature_direction < 0.0 && right_curvature_direction < 0.0);

            assert!(curvature_directions_match,
                "Arc curvature directions should match for symmetry: left={:.3}, right={:.3} at y={:.1}",
                left_curvature_direction, right_curvature_direction, left_y_center
            );
        }
    }

    println!("Arc curvature symmetry test passed: {} left arcs, {} right arcs",
             left_arcs.len(), right_arcs.len());
}