//! examples/3d/operations/intersection_test.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Sphere, Volume},
    mesh::{operations::intersection, primitives::{generate_cuboid, generate_sphere}, write_stl},
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

    // 2. Create a ChannelSystem3D for visualization purposes.
    let system_3d = ChannelSystem3D {
        box_volume: cuboid_volume.clone(),
        cylinders: vec![],
        spheres: vec![sphere.clone()],
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

    // 6. Plot the original shapes for visualization.
    let plot_path = format!("{}/intersection_test.png", output_dir);
    println!("Plotting the original shapes...");
    plot_3d_system(&system_3d, &plot_path)?;

    println!(
        "Intersection test finished. View the result in '{}' and '{}'",
        mesh_path, plot_path
    );

    Ok(())
} 