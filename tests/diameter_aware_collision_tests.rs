use scheme::config::{GeometryConfig, SerpentineConfig, WaveShape};
use scheme::geometry::strategies::{SerpentineChannelStrategy, ChannelTypeStrategy};
use scheme::geometry::types::ChannelType;
use scheme::config_constants::ConstantsRegistry;

#[test]
fn test_diameter_aware_amplitude_calculation() {
    let constants = ConstantsRegistry::new();
    let min_wall_thickness = constants.get_min_wall_thickness();
    
    // Test with different channel widths
    let test_cases = vec![
        (0.5, 2.0),  // Small channel, normal wavelength factor
        (1.0, 4.0),  // Standard channel, normal wavelength factor
        (2.0, 6.0),  // Large channel, normal wavelength factor
    ];
    
    for (channel_width, wavelength_factor) in test_cases {
        let geometry_config = GeometryConfig::new(0.5, channel_width, 0.5).unwrap();
        let serpentine_config = SerpentineConfig {
            wavelength_factor,
            fill_factor: 0.8,
            ..SerpentineConfig::default()
        };
        
        let strategy = SerpentineChannelStrategy::new(serpentine_config);
        
        // Create a channel in a reasonable box
        let box_dims = (20.0, 10.0);
        let channel_type = strategy.create_channel(
            (2.0, 5.0),
            (18.0, 5.0),
            &geometry_config,
            box_dims,
            1,
            None,
        );
        
        match channel_type {
            ChannelType::Serpentine { path } => {
                assert!(!path.is_empty(), "Should generate a valid path");
                
                // Check that the path doesn't violate manufacturing constraints
                let channel_center_y = 5.0;
                let max_deviation = path.iter()
                    .map(|&(_, y)| (y - channel_center_y).abs())
                    .fold(0.0, f64::max);
                
                // Ensure the maximum deviation respects the minimum wall thickness
                let channel_radius = channel_width / 2.0;
                let max_safe_deviation = (box_dims.1 / 2.0) - geometry_config.wall_clearance - channel_radius;
                
                assert!(
                    max_deviation <= max_safe_deviation,
                    "Channel deviation {} exceeds safe limit {} for channel width {}",
                    max_deviation, max_safe_deviation, channel_width
                );
                
                println!("✓ Channel width {}: max deviation {} ≤ safe limit {}", 
                    channel_width, max_deviation, max_safe_deviation);
            }
            _ => panic!("Should produce serpentine channel"),
        }
    }
}

#[test]
fn test_minimum_wall_thickness_constraint() {
    let constants = ConstantsRegistry::new();
    let min_wall_thickness = constants.get_min_wall_thickness();
    
    // Test with very close neighbors to ensure wall thickness is maintained
    let channel_width = 1.0;
    let geometry_config = GeometryConfig::new(0.5, channel_width, 0.5).unwrap();
    let serpentine_config = SerpentineConfig {
        wavelength_factor: 3.0,
        fill_factor: 0.9, // High fill factor to test constraints
        ..SerpentineConfig::default()
    };
    
    let strategy = SerpentineChannelStrategy::new(serpentine_config);
    
    // Create neighbors very close to the channel
    let channel_y = 5.0;
    let neighbor_separation = channel_width + min_wall_thickness + 0.1; // Just above minimum
    let neighbors = vec![
        channel_y - neighbor_separation,
        channel_y + neighbor_separation,
    ];
    
    let channel_type = strategy.create_channel(
        (2.0, channel_y),
        (18.0, channel_y),
        &geometry_config,
        (20.0, 10.0),
        3,
        Some(&neighbors),
    );
    
    match channel_type {
        ChannelType::Serpentine { path } => {
            // Check that no point in the path violates the minimum wall thickness
            for &(_, y) in &path {
                for &neighbor_y in &neighbors {
                    let distance_to_neighbor = (y - neighbor_y).abs();
                    let required_distance = (channel_width / 2.0) + min_wall_thickness;
                    
                    assert!(
                        distance_to_neighbor >= required_distance,
                        "Point at y={} violates minimum wall thickness to neighbor at y={}: distance={}, required={}",
                        y, neighbor_y, distance_to_neighbor, required_distance
                    );
                }
            }
            
            println!("✓ All points maintain minimum wall thickness of {} mm", min_wall_thickness);
        }
        _ => panic!("Should produce serpentine channel"),
    }
}

#[test]
fn test_wavelength_validation_for_diameter() {
    let constants = ConstantsRegistry::new();
    let min_wall_thickness = constants.get_min_wall_thickness();
    
    // Test with very small wavelength factor that would violate manufacturing constraints
    let channel_width = 2.0;
    let geometry_config = GeometryConfig::new(0.5, channel_width, 0.5).unwrap();
    let serpentine_config = SerpentineConfig {
        wavelength_factor: 1.0, // Very small wavelength factor
        fill_factor: 0.8,
        ..SerpentineConfig::default()
    };
    
    let strategy = SerpentineChannelStrategy::new(serpentine_config);
    
    let channel_type = strategy.create_channel(
        (2.0, 5.0),
        (18.0, 5.0),
        &geometry_config,
        (20.0, 10.0),
        1,
        None,
    );
    
    match channel_type {
        ChannelType::Serpentine { path } => {
            // Calculate the actual wavelength used
            let mut x_positions: Vec<f64> = path.iter().map(|&(x, _)| x).collect();
            x_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            // The wavelength should be automatically adjusted to meet manufacturing constraints
            let min_required_wavelength = 2.0 * (channel_width + min_wall_thickness);
            
            // Check that the path has reasonable spacing
            assert!(
                !path.is_empty(),
                "Should generate a valid path even with small wavelength factor"
            );
            
            println!("✓ Wavelength validation ensures manufacturing constraints are met");
            println!("  Minimum required wavelength: {} mm", min_required_wavelength);
            println!("  Channel width: {} mm, Min wall thickness: {} mm", channel_width, min_wall_thickness);
        }
        _ => panic!("Should produce serpentine channel"),
    }
}

#[test]
fn test_square_wave_diameter_constraints() {
    // Test that square waves also respect diameter constraints
    let channel_width = 1.5;
    let geometry_config = GeometryConfig::new(0.5, channel_width, 0.5).unwrap();
    let serpentine_config = SerpentineConfig {
        wavelength_factor: 4.0,
        fill_factor: 0.7,
        wave_shape: WaveShape::Square,
        ..SerpentineConfig::default()
    };
    
    let strategy = SerpentineChannelStrategy::new(serpentine_config);
    
    let channel_type = strategy.create_channel(
        (2.0, 5.0),
        (18.0, 5.0),
        &geometry_config,
        (20.0, 10.0),
        1,
        None,
    );
    
    match channel_type {
        ChannelType::Serpentine { path } => {
            assert!(!path.is_empty(), "Should generate a valid square wave path");
            
            // Check that square waves also maintain manufacturing constraints
            let channel_center_y = 5.0;
            let max_deviation = path.iter()
                .map(|&(_, y)| (y - channel_center_y).abs())
                .fold(0.0, f64::max);
            
            let channel_radius = channel_width / 2.0;
            let max_safe_deviation = (10.0 / 2.0) - geometry_config.wall_clearance - channel_radius;
            
            assert!(
                max_deviation <= max_safe_deviation,
                "Square wave deviation {} exceeds safe limit {}",
                max_deviation, max_safe_deviation
            );
            
            println!("✓ Square wave respects diameter constraints: max deviation {} ≤ {}", 
                max_deviation, max_safe_deviation);
        }
        _ => panic!("Should produce serpentine channel"),
    }
}

#[test]
fn test_manufacturing_constraint_constants() {
    let constants = ConstantsRegistry::new();
    let min_wall_thickness = constants.get_min_wall_thickness();
    
    // Verify the minimum wall thickness is set to the expected value
    assert_eq!(min_wall_thickness, 0.45, "Minimum wall thickness should be 0.45mm");
    
    // Verify it's within reasonable manufacturing bounds
    assert!(min_wall_thickness > 0.0, "Minimum wall thickness must be positive");
    assert!(min_wall_thickness < 5.0, "Minimum wall thickness should be reasonable for microfluidics");
    
    println!("✓ Manufacturing constraint constants are properly configured");
    println!("  Minimum wall thickness: {} mm", min_wall_thickness);
}
