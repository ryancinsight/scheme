//! examples/3d/operations/subtract_test.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Sphere, Volume},
    mesh::{operations::subtract, primitives::{generate_cuboid, generate_sphere}, write_stl},
    visualizations::plot_3d_system,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/operations";
    std::fs::create_dir_all(output_dir)?;

    // 1. Define two overlapping shapes: a cuboid and a sphere.
    // NOTE: Using simplified geometry to prevent stack overflow with the current
    // recursive CSG implementation.
    let cuboid_volume = Volume {
        min_corner: (-5.0, -5.0, -5.0),
        max_corner: (5.0, 5.0, 5.0),
    };
    let sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 4.0,
    };

    // 2. Generate meshes for both shapes.
    println!("Generating component meshes...");
    let cuboid_mesh = generate_cuboid(&cuboid_volume);
    let sphere_mesh = generate_sphere(&sphere, 16, 16);

    // 3. Perform the Subtract operation (Cuboid - Sphere).
    println!("Performing Subtract operation...");
    let result_mesh = subtract(&cuboid_mesh, &sphere_mesh)?;

    // 4. Plot the original shapes for context.
    println!("Plotting the original shapes...");
    let original_system = ChannelSystem3D {
        box_volume: cuboid_volume.clone(),
        cylinders: vec![],
        spheres: vec![sphere.clone()],
    };
    let plot_path = format!("{}/subtract_test.png", output_dir);
    plot_3d_system(&original_system, &plot_path)?;

    // 5. Save the result to an STL file.
    let stl_path = format!("{}/subtract.stl", output_dir);
    write_stl(&stl_path, &result_mesh)?;

    println!("Subtract test finished. View the result in '{}' and the context plot in '{}'", stl_path, plot_path);
    Ok(())
} 