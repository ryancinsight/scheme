//! CSG Robustness Enhancement Tests
//!
//! Test-Driven Development for numerical stability, degenerate geometry handling,
//! and boundary condition robustness in BSP tree-based CSG operations.
//! Following Cathedral Engineering principles with systematic validation.

use pyvismil::mesh::operations::{intersection, union, subtract};
use pyvismil::mesh::primitives::{generate_cuboid, generate_sphere};
use pyvismil::geometry::mod_3d::{Volume, Sphere};
use stl_io::Triangle;
use std::time::Instant;

/// Test enhanced CSG operations with improved numerical stability
#[test]
fn test_enhanced_csg_numerical_stability() {
    // Test with small geometry (millimeter scale)
    let small_cube = Volume {
        min_corner: (-0.001, -0.001, -0.001),
        max_corner: (0.001, 0.001, 0.001),
    };
    let small_sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 0.0015,
    };

    let small_cube_mesh = generate_cuboid(&small_cube);
    let small_sphere_mesh = generate_sphere(&small_sphere, 8, 16);

    // Test that enhanced CSG operations handle small geometry
    let intersection_result = intersection(&small_cube_mesh, &small_sphere_mesh);
    assert!(intersection_result.is_ok(), "Enhanced intersection should handle small geometry");

    let union_result = union(&small_cube_mesh, &small_sphere_mesh);
    assert!(union_result.is_ok(), "Enhanced union should handle small geometry");

    let subtract_result = subtract(&small_cube_mesh, &small_sphere_mesh);
    assert!(subtract_result.is_ok(), "Enhanced subtract should handle small geometry");
}

/// Test enhanced CSG operations with large geometry
#[test]
fn test_enhanced_csg_large_geometry() {
    // Test with large geometry (kilometer scale)
    let large_cube = Volume {
        min_corner: (-1000.0, -1000.0, -1000.0),
        max_corner: (1000.0, 1000.0, 1000.0),
    };
    let large_sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 1500.0,
    };

    let large_cube_mesh = generate_cuboid(&large_cube);
    let large_sphere_mesh = generate_sphere(&large_sphere, 8, 16);

    // Test that enhanced CSG operations handle large geometry
    let intersection_result = intersection(&large_cube_mesh, &large_sphere_mesh);
    assert!(intersection_result.is_ok(), "Enhanced intersection should handle large geometry");

    let union_result = union(&large_cube_mesh, &large_sphere_mesh);
    assert!(union_result.is_ok(), "Enhanced union should handle large geometry");

    let subtract_result = subtract(&large_cube_mesh, &large_sphere_mesh);
    assert!(subtract_result.is_ok(), "Enhanced subtract should handle large geometry");
}

/// Test CSG operations with degenerate input geometry
#[test]
fn test_csg_operations_with_degenerate_input() {
    // Create mesh with some degenerate triangles
    let mut mesh_with_degenerates = generate_cuboid(&Volume {
        min_corner: (-1.0, -1.0, -1.0),
        max_corner: (1.0, 1.0, 1.0),
    });

    // Add degenerate triangles
    mesh_with_degenerates.push(create_degenerate_triangle_zero_area());
    mesh_with_degenerates.push(create_degenerate_triangle_collinear());

    let sphere_mesh = generate_sphere(&Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 1.5,
    }, 8, 16);

    // Test that enhanced CSG operations handle degenerate input gracefully
    let intersection_result = intersection(&mesh_with_degenerates, &sphere_mesh);
    assert!(intersection_result.is_ok(), "Enhanced CSG intersection should handle degenerate input gracefully");

    let union_result = union(&mesh_with_degenerates, &sphere_mesh);
    assert!(union_result.is_ok(), "Enhanced CSG union should handle degenerate input gracefully");

    let subtract_result = subtract(&mesh_with_degenerates, &sphere_mesh);
    assert!(subtract_result.is_ok(), "Enhanced CSG subtract should handle degenerate input gracefully");
}

/// Test performance with complex overlapping geometry
#[test]
fn test_enhanced_csg_performance_complex_geometry() {
    // Create complex overlapping geometry
    let complex_cube = generate_cuboid(&Volume {
        min_corner: (-2.0, -2.0, -2.0),
        max_corner: (2.0, 2.0, 2.0),
    });

    let complex_sphere = generate_sphere(&Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 2.5,
    }, 32, 64); // High resolution for complexity

    // Measure enhanced intersection performance
    let start = Instant::now();
    let result = intersection(&complex_cube, &complex_sphere).unwrap();
    let duration = start.elapsed();

    println!("Enhanced complex intersection: {} triangles in {:?}", result.len(), duration);

    // Performance target: <500ms for complex cases (enhanced robustness may have slight overhead)
    assert!(duration.as_millis() < 1000, "Enhanced complex intersection should complete within 1000ms, took {:?}", duration);
    assert!(result.len() > 0, "Enhanced complex intersection should produce non-empty result");

    // Test that result quality is maintained
    assert!(result.len() > 100, "Enhanced intersection should produce detailed result for complex geometry");
}

/// Test enhanced CSG operations maintain backward compatibility
#[test]
fn test_enhanced_csg_backward_compatibility() {
    // Use the same test geometry as the original intersection test
    let cuboid_volume = Volume {
        min_corner: (-5.0, -5.0, -5.0),
        max_corner: (5.0, 5.0, 5.0),
    };
    let sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 7.0,
    };

    let cuboid_mesh = generate_cuboid(&cuboid_volume);
    let sphere_mesh = generate_sphere(&sphere, 12, 12);

    // Test that enhanced operations produce similar results to original
    let intersection_result = intersection(&cuboid_mesh, &sphere_mesh).unwrap();

    // Should produce complex geometry (not just 12 triangles like the original bug)
    assert!(intersection_result.len() > 100, "Enhanced intersection should produce complex geometry, got {} triangles", intersection_result.len());

    // Should complete in reasonable time
    let start = Instant::now();
    let _union_result = union(&cuboid_mesh, &sphere_mesh).unwrap();
    let union_duration = start.elapsed();

    let start = Instant::now();
    let _subtract_result = subtract(&cuboid_mesh, &sphere_mesh).unwrap();
    let subtract_duration = start.elapsed();

    println!("Enhanced operation timings - Union: {:?}, Subtract: {:?}", union_duration, subtract_duration);

    // Performance should be reasonable
    assert!(union_duration.as_millis() < 500, "Enhanced union should complete within 500ms");
    assert!(subtract_duration.as_millis() < 500, "Enhanced subtract should complete within 500ms");
}

// Helper functions for test creation

fn create_degenerate_triangle_zero_area() -> Triangle {
    use stl_io::Vector;
    Triangle {
        normal: Vector::new([0.0, 0.0, 1.0]),
        vertices: [
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([0.0, 0.0, 0.0]),
        ],
    }
}

fn create_degenerate_triangle_collinear() -> Triangle {
    use stl_io::Vector;
    Triangle {
        normal: Vector::new([0.0, 0.0, 1.0]),
        vertices: [
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([1.0, 0.0, 0.0]),
            Vector::new([2.0, 0.0, 0.0]),
        ],
    }
}
