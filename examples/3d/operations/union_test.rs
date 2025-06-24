//! examples/3d/operations/union_test.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Sphere, Volume},
    mesh::{operations::union, primitives::{generate_cuboid, generate_sphere}, write_stl},
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

    // 3. Perform the Union operation.
    println!("Performing Union operation...");
    let result_mesh = union(&cuboid_mesh, &sphere_mesh)?;

    // 4. Save the resulting mesh to an STL file.
    let output_path = format!("{}/union.stl", output_dir);
    write_stl(&output_path, &result_mesh)?;

    // 5. Plot the input shapes for context.
    println!("Plotting the input shapes...");
    let system_for_plotting = ChannelSystem3D {
        box_volume: cuboid_volume,
        cylinders: vec![],
        spheres: vec![sphere],
        cones: vec![],
        tori: vec![],
    };
    let input_plot_path = format!("{}/union_input.png", output_dir);
    plot_3d_system(&system_for_plotting, &input_plot_path)?;

    // 6. Plot the computed union result.
    let result_plot_path = format!("{}/union_result.png", output_dir);
    println!("Plotting the union result...");
    plot_mesh_result(&result_mesh, &result_plot_path, "Union Result")?;

    println!(
        "Union test finished. View the STL result in '{}', input shapes in '{}', and computed result in '{}'",
        output_path, input_plot_path, result_plot_path
    );

    Ok(())
} 