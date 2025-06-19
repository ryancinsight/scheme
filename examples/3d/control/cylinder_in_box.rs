//! examples/3d/control/cylinder_in_box.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Cylinder, Volume},
    mesh::{generate_mesh_from_system, write_stl},
    visualizations::plot_3d_system,
};
use std::fs;

fn main() {
    println!("Generating 3D box with a central cylinder...");

    let box_dims = (127.0, 85.0, 44.0);
    let cylinder_radius = 5.0;

    let system_3d = ChannelSystem3D {
        box_volume: Volume {
            min_corner: (0.0, 0.0, 0.0),
            max_corner: box_dims,
        },
        cylinders: vec![Cylinder {
            start: (0.0, box_dims.1 / 2.0, box_dims.2 / 2.0),
            end: (box_dims.0, box_dims.1 / 2.0, box_dims.2 / 2.0),
            radius: cylinder_radius,
        }],
    };

    let output_dir = "outputs/3d/control";
    fs::create_dir_all(output_dir).unwrap();

    // --- Generate and save 3D plot ---
    println!("Plotting 3D system...");
    let plot_output_path = format!("{}/cylinder_in_box_plot.png", output_dir);
    if let Err(e) = plot_3d_system(&system_3d, &plot_output_path) {
        eprintln!("Error plotting 3D system: {}", e);
    }

    // --- Generate and save STL mesh ---
    println!("Generating mesh...");
    let triangles = generate_mesh_from_system(&system_3d);
    let stl_output_path = format!("{}/cylinder_in_box.stl", output_dir);
    if let Err(e) = write_stl(&stl_output_path, &triangles) {
        eprintln!("Error writing STL file: {}", e);
    }
} 