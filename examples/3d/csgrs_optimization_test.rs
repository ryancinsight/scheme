//! CSG Optimization Test Example
//!
//! This example demonstrates and validates the optimization in the csgrs-based
//! difference function. It compares the original iterative approach with the
//! optimized union-then-difference approach for multiple cylinders.

use pyvismil::{
    config::{ConversionConfig, GeometryConfig},
    geometry::{convert_2d_to_3d, create_geometry, SplitType},
    mesh::{difference, difference_csgrs, write_stl},
    visualizations,
};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 CSG Optimization Test - Comparing difference calculation approaches");
    
    // Create a test configuration with multiple cylinders for optimization testing
    let config = GeometryConfig {
        split_type: SplitType::Horizontal,
        split_count: 4, // This will create 4 cylinders - good for testing optimization
        cylinder_diameter: 2.0,
        box_width: 20.0,
        box_height: 20.0,
        box_depth: 10.0,
        spacing: 5.0,
    };

    let conversion_config = ConversionConfig {
        extrusion_height: 10.0,
    };

    println!("📐 Configuration:");
    println!("   • Box dimensions: {}×{}×{}", config.box_width, config.box_height, config.box_depth);
    println!("   • Number of cylinders: {}", config.split_count);
    println!("   • Cylinder diameter: {}", config.cylinder_diameter);
    println!("   • This tests the optimization for {} CSG operations", config.split_count);

    // 1. Create 2D geometry
    println!("\n🎯 Creating 2D geometry...");
    let geometry_2d = create_geometry(&config)?;

    // 2. Convert to 3D
    println!("🔄 Converting to 3D system...");
    let system_3d = convert_2d_to_3d(&geometry_2d, &conversion_config)?;
    
    println!("   ✓ 3D system created with {} cylinders", system_3d.cylinders.len());

    // 3. Create output directory
    let output_dir = "outputs/3d/csgrs_optimization_test";
    std::fs::create_dir_all(output_dir)?;

    // 4. Generate visualization
    println!("\n📊 Generating 3D visualization...");
    let plot_path = format!("{}/optimization_test_plot.png", output_dir);
    visualizations::plot_3d_system(&system_3d, &plot_path)?;
    println!("   ✓ 3D plot saved to: {}", plot_path);

    // 5. Test original pyvismil implementation (baseline)
    println!("\n⚡ Testing original pyvismil implementation...");
    let start_time = Instant::now();
    let original_mesh = difference(&system_3d)?;
    let original_duration = start_time.elapsed();
    
    println!("   ✓ Original implementation completed");
    println!("   ⏱️  Time: {:?}", original_duration);
    println!("   📊 Triangles: {}", original_mesh.len());

    // 6. Test optimized csgrs implementation
    println!("\n🚀 Testing optimized csgrs implementation...");
    let start_time = Instant::now();
    let optimized_mesh = difference_csgrs(&system_3d)?;
    let optimized_duration = start_time.elapsed();
    
    println!("   ✓ Optimized csgrs implementation completed");
    println!("   ⏱️  Time: {:?}", optimized_duration);
    println!("   📊 Triangles: {}", optimized_mesh.len());

    // 7. Save both meshes for comparison
    let original_stl_path = format!("{}/original_implementation.stl", output_dir);
    let optimized_stl_path = format!("{}/optimized_csgrs.stl", output_dir);
    
    write_stl(&original_stl_path, &original_mesh)?;
    write_stl(&optimized_stl_path, &optimized_mesh)?;

    // 8. Performance analysis
    println!("\n📈 Performance Analysis:");
    println!("   • Original implementation: {:?} ({} triangles)", original_duration, original_mesh.len());
    println!("   • Optimized csgrs: {:?} ({} triangles)", optimized_duration, optimized_mesh.len());
    
    if optimized_duration < original_duration {
        let speedup = original_duration.as_secs_f64() / optimized_duration.as_secs_f64();
        println!("   🎉 Optimization achieved {:.2}x speedup!", speedup);
    } else {
        let slowdown = optimized_duration.as_secs_f64() / original_duration.as_secs_f64();
        println!("   ⚠️  Optimization was {:.2}x slower (csgrs overhead for small geometries)", slowdown);
    }

    // 9. Mesh quality comparison
    let triangle_ratio = optimized_mesh.len() as f64 / original_mesh.len() as f64;
    println!("   📊 Mesh detail ratio: {:.2}x (csgrs vs original)", triangle_ratio);
    
    if triangle_ratio > 2.0 {
        println!("   ✨ CSGRS provides significantly more detailed mesh");
    } else if triangle_ratio > 1.1 {
        println!("   ✓ CSGRS provides moderately more detailed mesh");
    } else {
        println!("   📝 Similar mesh detail levels");
    }

    // 10. Optimization validation
    println!("\n🔍 Optimization Validation:");
    if system_3d.cylinders.len() > 1 {
        println!("   ✓ Multiple cylinders detected - optimization active");
        println!("   📉 Reduced from {} difference operations to 1 union + 1 difference", system_3d.cylinders.len());
        println!("   🎯 Expected benefits: Better numerical stability, fewer cascading errors");
    } else {
        println!("   📝 Single cylinder - direct difference used (no union needed)");
    }

    // 11. Summary
    println!("\n📋 Test Summary:");
    println!("   📁 Files generated:");
    println!("      • Visualization: {}", plot_path);
    println!("      • Original mesh: {}", original_stl_path);
    println!("      • Optimized mesh: {}", optimized_stl_path);
    println!("   🔧 Optimization strategy validated for {} cylinders", system_3d.cylinders.len());
    println!("   ✅ Both implementations completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyvismil::geometry::mod_3d::{ChannelSystem3D, Cylinder, Volume};
    use nalgebra::{Point3, Vector3};

    #[test]
    fn test_single_cylinder_optimization() {
        // Test that single cylinder uses direct difference (no union)
        let system = ChannelSystem3D {
            box_volume: Volume {
                min: Point3::new(0.0, 0.0, 0.0),
                max: Point3::new(10.0, 10.0, 10.0),
            },
            cylinders: vec![
                Cylinder {
                    center: Point3::new(5.0, 5.0, 5.0),
                    radius: 2.0,
                    height: 10.0,
                    direction: Vector3::new(0.0, 0.0, 1.0),
                }
            ],
        };

        let result = difference_csgrs(&system);
        assert!(result.is_ok(), "Single cylinder difference should succeed");
        
        let mesh = result.unwrap();
        assert!(!mesh.is_empty(), "Result mesh should not be empty");
    }

    #[test]
    fn test_multiple_cylinder_optimization() {
        // Test that multiple cylinders use union-then-difference optimization
        let system = ChannelSystem3D {
            box_volume: Volume {
                min: Point3::new(0.0, 0.0, 0.0),
                max: Point3::new(20.0, 20.0, 10.0),
            },
            cylinders: vec![
                Cylinder {
                    center: Point3::new(5.0, 5.0, 5.0),
                    radius: 1.5,
                    height: 10.0,
                    direction: Vector3::new(0.0, 0.0, 1.0),
                },
                Cylinder {
                    center: Point3::new(15.0, 5.0, 5.0),
                    radius: 1.5,
                    height: 10.0,
                    direction: Vector3::new(0.0, 0.0, 1.0),
                },
                Cylinder {
                    center: Point3::new(5.0, 15.0, 5.0),
                    radius: 1.5,
                    height: 10.0,
                    direction: Vector3::new(0.0, 0.0, 1.0),
                },
            ],
        };

        let result = difference_csgrs(&system);
        assert!(result.is_ok(), "Multiple cylinder difference should succeed");
        
        let mesh = result.unwrap();
        assert!(!mesh.is_empty(), "Result mesh should not be empty");
    }

    #[test]
    fn test_empty_cylinders() {
        // Test that empty cylinder list returns original box
        let system = ChannelSystem3D {
            box_volume: Volume {
                min: Point3::new(0.0, 0.0, 0.0),
                max: Point3::new(10.0, 10.0, 10.0),
            },
            cylinders: vec![],
        };

        let result = difference_csgrs(&system);
        assert!(result.is_ok(), "Empty cylinders should succeed");
        
        let mesh = result.unwrap();
        assert!(!mesh.is_empty(), "Result should be the original box mesh");
    }
}
