//! tests/strategy_tests.rs
//! 
//! Tests for the channel type strategy pattern implementation

use scheme::{
    geometry::{ChannelType, Point2D, strategies::*},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig},
};

/// Test that StraightChannelStrategy always returns straight channels
#[test]
fn test_straight_channel_strategy() {
    let strategy = StraightChannelStrategy;
    let config = GeometryConfig::default();
    
    let channel_type = strategy.create_channel(
        (0.0, 0.0),
        (10.0, 5.0),
        &config,
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type {
        ChannelType::Straight => {}, // Expected
        _ => panic!("StraightChannelStrategy should always return Straight channels"),
    }
}

/// Test that SerpentineChannelStrategy returns serpentine channels with paths
#[test]
fn test_serpentine_channel_strategy() {
    let serpentine_config = SerpentineConfig::default();
    let strategy = SerpentineChannelStrategy::new(serpentine_config);
    let geometry_config = GeometryConfig::default();
    
    let channel_type = strategy.create_channel(
        (0.0, 2.0),
        (10.0, 3.0),
        &geometry_config,
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type {
        ChannelType::Serpentine { path } => {
            assert!(!path.is_empty(), "Serpentine channel should have a path");
            assert_eq!(path.len(), 100, "Serpentine path should have 100 points");
            
            // Check endpoint alignment
            assert_eq!(path[0], (0.0, 2.0), "First point should match start");
            assert_eq!(path[path.len() - 1], (10.0, 3.0), "Last point should match end");
        },
        _ => panic!("SerpentineChannelStrategy should return Serpentine channels"),
    }
}

/// Test that ArcChannelStrategy returns arc channels with paths
#[test]
fn test_arc_channel_strategy() {
    let arc_config = ArcConfig::default();
    let strategy = ArcChannelStrategy::new(arc_config);
    let geometry_config = GeometryConfig::default();
    
    let channel_type = strategy.create_channel(
        (0.0, 2.0),
        (10.0, 8.0),
        &geometry_config,
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type {
        ChannelType::Arc { path } => {
            assert!(!path.is_empty(), "Arc channel should have a path");
            assert!(path.len() >= 2, "Arc path should have at least start and end points");
            
            // Check endpoint alignment
            assert_eq!(path[0], (0.0, 2.0), "First point should match start");
            assert_eq!(path[path.len() - 1], (10.0, 8.0), "Last point should match end");
        },
        _ => panic!("ArcChannelStrategy should return Arc channels"),
    }
}

/// Test ChannelTypeFactory with AllStraight configuration
#[test]
fn test_factory_all_straight() {
    let config = ChannelTypeConfig::AllStraight;
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
        ChannelType::Straight => {}, // Expected
        _ => panic!("AllStraight config should produce straight channels"),
    }
}

/// Test ChannelTypeFactory with AllSerpentine configuration
#[test]
fn test_factory_all_serpentine() {
    let serpentine_config = SerpentineConfig::default();
    let config = ChannelTypeConfig::AllSerpentine(serpentine_config);
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
        ChannelType::Serpentine { .. } => {}, // Expected
        _ => panic!("AllSerpentine config should produce serpentine channels"),
    }
}

/// Test ChannelTypeFactory with AllArcs configuration
#[test]
fn test_factory_all_arcs() {
    let arc_config = ArcConfig::default();
    let config = ChannelTypeConfig::AllArcs(arc_config);
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
        ChannelType::Arc { .. } => {}, // Expected
        _ => panic!("AllArcs config should produce arc channels"),
    }
}

/// Test ChannelTypeFactory with MixedByPosition configuration
#[test]
fn test_factory_mixed_by_position() {
    let serpentine_config = SerpentineConfig::default();
    let arc_config = ArcConfig::default();
    let config = ChannelTypeConfig::MixedByPosition {
        middle_zone_fraction: 0.4,
        serpentine_config,
        arc_config,
    };
    
    // Test channel in middle zone (should be serpentine)
    let strategy_middle = ChannelTypeFactory::create_strategy(
        &config,
        (9.0, 2.0),  // Middle of 20-unit wide box
        (11.0, 3.0),
        (20.0, 10.0),
    );
    
    let channel_type_middle = strategy_middle.create_channel(
        (9.0, 2.0),
        (11.0, 3.0),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type_middle {
        ChannelType::Serpentine { .. } => {}, // Expected for middle zone
        _ => panic!("Middle zone should produce serpentine channels"),
    }
    
    // Test angled channel outside middle zone (should be arc)
    let strategy_side = ChannelTypeFactory::create_strategy(
        &config,
        (2.0, 2.0),  // Side of box
        (4.0, 8.0),  // Angled channel
        (20.0, 10.0),
    );
    
    let channel_type_side = strategy_side.create_channel(
        (2.0, 2.0),
        (4.0, 8.0),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type_side {
        ChannelType::Arc { .. } => {}, // Expected for angled channel outside middle
        _ => panic!("Angled channel outside middle should produce arc channels"),
    }
}

/// Test ChannelTypeFactory with Smart configuration
#[test]
fn test_factory_smart() {
    let serpentine_config = SerpentineConfig::default();
    let arc_config = ArcConfig::default();
    let config = ChannelTypeConfig::Smart {
        serpentine_config,
        arc_config,
    };
    
    // Test long horizontal channel (should be serpentine)
    let strategy_long = ChannelTypeFactory::create_strategy(
        &config,
        (2.0, 5.0),
        (15.0, 5.2),  // Long, mostly horizontal
        (20.0, 10.0),
    );
    
    let channel_type_long = strategy_long.create_channel(
        (2.0, 5.0),
        (15.0, 5.2),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type_long {
        ChannelType::Serpentine { .. } => {}, // Expected for long horizontal
        _ => panic!("Long horizontal channel should produce serpentine"),
    }
    
    // Test short channel (should be straight)
    let strategy_short = ChannelTypeFactory::create_strategy(
        &config,
        (5.0, 5.0),
        (6.0, 5.5),  // Short channel
        (20.0, 10.0),
    );
    
    let channel_type_short = strategy_short.create_channel(
        (5.0, 5.0),
        (6.0, 5.5),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type_short {
        ChannelType::Straight => {}, // Expected for short channel
        _ => panic!("Short channel should produce straight"),
    }
}

/// Test ChannelTypeFactory with Custom configuration
#[test]
fn test_factory_custom() {
    let custom_func = |from: Point2D, to: Point2D, _box_dims: (f64, f64)| {
        let dx = to.0 - from.0;
        if dx > 5.0 {
            ChannelType::Serpentine { path: vec![from, to] }
        } else {
            ChannelType::Straight
        }
    };
    
    let config = ChannelTypeConfig::Custom(custom_func);
    
    // Test long channel (should be serpentine per custom logic)
    let strategy_long = ChannelTypeFactory::create_strategy(
        &config,
        (0.0, 0.0),
        (10.0, 0.0),  // dx = 10 > 5
        (20.0, 10.0),
    );
    
    let channel_type_long = strategy_long.create_channel(
        (0.0, 0.0),
        (10.0, 0.0),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type_long {
        ChannelType::Serpentine { .. } => {}, // Expected per custom logic
        _ => panic!("Custom function should produce serpentine for long channels"),
    }
    
    // Test short channel (should be straight per custom logic)
    let strategy_short = ChannelTypeFactory::create_strategy(
        &config,
        (0.0, 0.0),
        (3.0, 0.0),  // dx = 3 < 5
        (20.0, 10.0),
    );
    
    let channel_type_short = strategy_short.create_channel(
        (0.0, 0.0),
        (3.0, 0.0),
        &GeometryConfig::default(),
        (20.0, 10.0),
        4,
        None,
    );
    
    match channel_type_short {
        ChannelType::Straight => {}, // Expected per custom logic
        _ => panic!("Custom function should produce straight for short channels"),
    }
}

/// Test serpentine amplitude calculation with neighbor information
#[test]
fn test_serpentine_amplitude_with_neighbors() {
    let serpentine_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
    };
    let strategy = SerpentineChannelStrategy::new(serpentine_config);
    let geometry_config = GeometryConfig::default();
    
    // Test with neighbor information
    let neighbors = vec![2.0, 6.0]; // Neighbors at y=2 and y=6
    let channel_type = strategy.create_channel(
        (0.0, 4.0),  // Channel at y=4 (between neighbors)
        (10.0, 4.0),
        &geometry_config,
        (20.0, 10.0),
        4,
        Some(&neighbors),
    );
    
    match channel_type {
        ChannelType::Serpentine { path } => {
            assert!(!path.is_empty(), "Should have a path");
            // The amplitude should be constrained by the neighbors
            // This is more of a smoke test to ensure it doesn't panic
        },
        _ => panic!("Should produce serpentine channel"),
    }
}
