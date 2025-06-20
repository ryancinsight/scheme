//! examples/3d/operations/difference_sphere_minus_cube.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Sphere, Volume},
    mesh::{operations::subtract, primitives::{generate_cuboid, generate_sphere}, write_stl},
    visualizations::plot_3d_system,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/operations";
    std::fs::create_dir_all(output_dir)?;

    // 1. Define two overlapping shapes: a cuboid and a sphere.
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

    // 3. Perform the Difference operation (Sphere - Cube).
    // This should create a sphere with a cubic hole in it
    println!("Performing Difference operation (Sphere - Cube)...");
    let result_mesh = subtract(&sphere_mesh, &cuboid_mesh)?;

    // 4. Plot the original shapes for context.
    println!("Plotting the original shapes...");
    let original_system = ChannelSystem3D {
        box_volume: cuboid_volume.clone(),
        cylinders: vec![],
        spheres: vec![sphere.clone()],
    };
    let plot_path = format!("{}/difference_sphere_minus_cube.png", output_dir);
    plot_3d_system(&original_system, &plot_path)?;

    // 5. Save the result to an STL file.
    let stl_path = format!("{}/difference_sphere_minus_cube.stl", output_dir);
    write_stl(&stl_path, &result_mesh)?;

    println!("Difference (Sphere - Cube) test finished.");
    println!("Expected result: A sphere with a cubic hole carved out of it");
    println!("View the result in '{}' and the context plot in '{}'", stl_path, plot_path);
    Ok(())
} 