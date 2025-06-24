//! examples/3d/operations/subtract_test.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Sphere, Volume},
    mesh::{operations::subtract, primitives::{generate_cuboid, generate_sphere}, write_stl},
    visualizations::{plot_3d_system, plot_mesh_result},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/operations";
    std::fs::create_dir_all(output_dir)?;

    // 1. Define two overlapping shapes: a cuboid and a sphere.
    // Standardized CSG operation parameters for consistent testing
    // Cube: 10x10x10 centered at origin
    // Sphere: radius 7.0 centered at origin (extends beyond cube for meaningful operations)
    let cuboid_volume = Volume {
        min_corner: (-5.0, -5.0, -5.0),
        max_corner: (5.0, 5.0, 5.0),
    };
    let sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 7.0,
    };

    // 2. Generate meshes for both shapes.
    println!("Generating component meshes...");
    let cuboid_mesh = generate_cuboid(&cuboid_volume);
    let sphere_mesh = generate_sphere(&sphere, 12, 12);

    // 3. Perform the Subtract operation (Cuboid - Sphere).
    println!("Performing Subtract operation...");
    let result_mesh = subtract(&cuboid_mesh, &sphere_mesh)?;

    // 4. Plot the input shapes for context.
    println!("Plotting the input shapes...");
    let original_system = ChannelSystem3D {
        box_volume: cuboid_volume.clone(),
        cylinders: vec![],
        spheres: vec![sphere.clone()],
        cones: vec![],
        tori: vec![],
    };
    let input_plot_path = format!("{}/subtract_input.png", output_dir);
    plot_3d_system(&original_system, &input_plot_path)?;

    // 5. Save the result to an STL file.
    let stl_path = format!("{}/subtract.stl", output_dir);
    write_stl(&stl_path, &result_mesh)?;

    // 6. Plot the computed subtract result.
    let result_plot_path = format!("{}/subtract_result.png", output_dir);
    println!("Plotting the subtract result...");
    plot_mesh_result(&result_mesh, &result_plot_path, "Subtract Result (Cube - Sphere)")?;

    println!(
        "Subtract test finished. View the STL result in '{}', input shapes in '{}', and computed result in '{}'",
        stl_path, input_plot_path, result_plot_path
    );
    Ok(())
} 