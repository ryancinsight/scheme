//! examples/3d/operations/union_test.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Sphere, Volume},
    mesh::{operations::union, primitives::{generate_cuboid, generate_sphere}, write_stl},
    visualizations::plot_3d_system,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/operations";
    std::fs::create_dir_all(output_dir)?;

    // 1. Define two overlapping shapes: a cuboid and a sphere.
    // NOTE: Using simplified geometry to prevent stack overflow with the current
    // recursive CSG implementation.
    let cuboid_volume = Volume {
        min_corner: (-2.0, -2.0, -2.0),
        max_corner: (2.0, 2.0, 2.0),
    };
    let sphere = Sphere {
        center: (0.0, 0.0, 0.0),
        radius: 3.0,
    };

    // 2. Generate meshes for both shapes.
    println!("Generating component meshes...");
    let cuboid_mesh = generate_cuboid(&cuboid_volume);
    let sphere_mesh = generate_sphere(&sphere, 12, 12);

    // 3. Perform the Union operation.
    println!("Performing Union operation...");
    let result_mesh = union(&cuboid_mesh, &sphere_mesh)?;

    // 4. Save the resulting mesh to an STL file.
    let output_path = format!("{}/union.stl", output_dir);
    write_stl(&output_path, &result_mesh)?;

    // 5. Plot the original shapes for visualization.
    println!("Plotting the original shapes...");
    let system_for_plotting = ChannelSystem3D {
        box_volume: cuboid_volume,
        cylinders: vec![],
        spheres: vec![sphere],
    };
    let plot_path = format!("{}/union_test.png", output_dir);
    plot_3d_system(&system_for_plotting, &plot_path)?;

    println!(
        "Union test finished. View the result in '{}' and '{}'",
        output_path, plot_path
    );

    Ok(())
} 