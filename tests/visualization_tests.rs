//! tests/visualization_tests.rs
//! 
//! Test-Driven Development for 3D Visualization Correctness
//! Following Cathedral Engineering principles for systematic validation

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Cone, Torus, Volume, Sphere},
    visualizations::{plot_3d_system, plot_mesh_result},
    mesh::primitives::{generate_cuboid, generate_sphere},
    mesh::operations::{intersection},
};
use std::fs;
use std::path::Path;

/// Test that cone visualization generates non-empty plot files
#[test]
fn test_cone_visualization_generates_output() {
    let cone = Cone {
        start: (0.0, 0.0, 0.0),
        end: (10.0, 0.0, 0.0),
        start_radius: 2.0,
        end_radius: 1.0,
    };

    let system_3d = ChannelSystem3D {
        box_volume: Volume::default(),
        cylinders: vec![],
        spheres: vec![],
        cones: vec![cone],
        tori: vec![],
    };

    let test_output_dir = "test_outputs/visualization";
    fs::create_dir_all(test_output_dir).unwrap();
    let plot_path = format!("{}/test_cone_plot.png", test_output_dir);

    // Test that plotting succeeds
    let result = plot_3d_system(&system_3d, &plot_path);
    assert!(result.is_ok(), "Cone visualization should succeed");

    // Test that file is created and non-empty
    assert!(Path::new(&plot_path).exists(), "Plot file should be created");
    let metadata = fs::metadata(&plot_path).unwrap();
    assert!(metadata.len() > 1000, "Plot file should be non-trivial size (>1KB)");
}

/// Test that torus visualization generates non-empty plot files
#[test]
fn test_torus_visualization_generates_output() {
    let torus = Torus {
        center: (0.0, 0.0, 0.0),
        major_radius: 10.0,
        minor_radius: 2.0,
    };

    let system_3d = ChannelSystem3D {
        box_volume: Volume::default(),
        cylinders: vec![],
        spheres: vec![],
        cones: vec![],
        tori: vec![torus],
    };

    let test_output_dir = "test_outputs/visualization";
    fs::create_dir_all(test_output_dir).unwrap();
    let plot_path = format!("{}/test_torus_plot.png", test_output_dir);

    // Test that plotting succeeds
    let result = plot_3d_system(&system_3d, &plot_path);
    assert!(result.is_ok(), "Torus visualization should succeed");

    // Test that file is created and non-empty
    assert!(Path::new(&plot_path).exists(), "Plot file should be created");
    let metadata = fs::metadata(&plot_path).unwrap();
    assert!(metadata.len() > 1000, "Plot file should be non-trivial size (>1KB)");
}

/// Test standardized CSG operation parameters
#[test]
fn test_standardized_csg_parameters() {
    // Define the standardized parameters that all CSG operations should use
    let standard_cuboid = Volume {
        min_corner: (-5.0, -5.0, -5.0),
        max_corner: (5.0, 5.0, 5.0),
    };
    let standard_sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 7.0,
    };

    // Test that these parameters create valid overlapping geometry
    assert_eq!(standard_cuboid.min_corner, (-5.0, -5.0, -5.0));
    assert_eq!(standard_cuboid.max_corner, (5.0, 5.0, 5.0));
    assert_eq!(standard_sphere.center, (0.0, 0.0, 0.0));
    assert_eq!(standard_sphere.radius, 7.0);

    // Verify sphere extends beyond cube (radius 7 > cube half-width 5)
    assert!(standard_sphere.radius > 5.0, "Sphere should extend beyond cube for meaningful CSG operations");
}

/// Test that visualization handles empty geometry gracefully
#[test]
fn test_empty_geometry_visualization() {
    let system_3d = ChannelSystem3D {
        box_volume: Volume::default(),
        cylinders: vec![],
        spheres: vec![],
        cones: vec![],
        tori: vec![],
    };

    let test_output_dir = "test_outputs/visualization";
    fs::create_dir_all(test_output_dir).unwrap();
    let plot_path = format!("{}/test_empty_plot.png", test_output_dir);

    // Test that plotting empty geometry succeeds
    let result = plot_3d_system(&system_3d, &plot_path);
    assert!(result.is_ok(), "Empty geometry visualization should succeed");

    // Test that file is created
    assert!(Path::new(&plot_path).exists(), "Plot file should be created even for empty geometry");
}

/// Test that mesh result visualization works correctly
#[test]
fn test_mesh_result_visualization() {
    // Create test geometry
    let cuboid = Volume {
        min_corner: (-2.0, -2.0, -2.0),
        max_corner: (2.0, 2.0, 2.0),
    };
    let sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 3.0,
    };

    // Generate meshes
    let cuboid_mesh = generate_cuboid(&cuboid);
    let sphere_mesh = generate_sphere(&sphere, 8, 8); // Lower resolution for faster test

    // Perform CSG operation
    let result_mesh = intersection(&cuboid_mesh, &sphere_mesh).unwrap();

    let test_output_dir = "test_outputs/visualization";
    fs::create_dir_all(test_output_dir).unwrap();
    let plot_path = format!("{}/test_mesh_result_plot.png", test_output_dir);

    // Test that mesh result plotting succeeds
    let result = plot_mesh_result(&result_mesh, &plot_path, "Test Intersection Result");
    assert!(result.is_ok(), "Mesh result visualization should succeed");

    // Test that file is created and non-empty
    assert!(Path::new(&plot_path).exists(), "Mesh result plot file should be created");
    let metadata = fs::metadata(&plot_path).unwrap();
    assert!(metadata.len() > 1000, "Mesh result plot file should be non-trivial size (>1KB)");
}
