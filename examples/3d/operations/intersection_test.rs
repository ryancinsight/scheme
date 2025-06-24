//! examples/3d/operations/intersection_test.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Sphere, Volume},
    mesh::{operations::intersection, primitives::{generate_cuboid, generate_sphere}, write_stl},
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

    // 2. Create a ChannelSystem3D for visualization purposes.
    let system_3d = ChannelSystem3D {
        box_volume: cuboid_volume.clone(),
        cylinders: vec![],
        spheres: vec![sphere.clone()],
        cones: vec![],
        tori: vec![],
    };

    // 3. Generate the meshes for each shape.
    println!("Generating component meshes...");
    let cuboid_mesh = generate_cuboid(&cuboid_volume);
    let sphere_mesh = generate_sphere(&sphere, 12, 12);

    // 4. Perform the intersection operation.
    println!("Performing intersection...");
    let intersection_mesh = intersection(&cuboid_mesh, &sphere_mesh)?;

    // 5. Save the resulting mesh to an STL file.
    let mesh_path = format!("{}/intersection.stl", output_dir);
    write_stl(&mesh_path, &intersection_mesh)?;

    // 6. Plot the input shapes for context.
    let input_plot_path = format!("{}/intersection_input.png", output_dir);
    println!("Plotting the input shapes...");
    plot_3d_system(&system_3d, &input_plot_path)?;

    // 7. Plot the computed intersection result.
    let result_plot_path = format!("{}/intersection_result.png", output_dir);
    println!("Plotting the intersection result...");
    plot_mesh_result(&intersection_mesh, &result_plot_path, "Intersection Result")?;

    println!(
        "Intersection test finished. View the STL result in '{}', input shapes in '{}', and computed result in '{}'",
        mesh_path, input_plot_path, result_plot_path
    );

    Ok(())
} 