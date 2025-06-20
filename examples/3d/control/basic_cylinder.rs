//! examples/3d/basic_cylinder.rs

use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Cylinder, Volume},
    mesh::{generate_mesh_from_system, write_stl},
    visualizations::plot_3d_system,
};
use std::fs;

fn main() {
    println!("Generating a single 3D cylinder...");

    let system_3d = ChannelSystem3D {
        box_volume: Volume::default(), // An empty/zero-volume box
        cylinders: vec![Cylinder {
            start: (0.0, 0.0, 0.0),
            end: (127.0, 0.0, 0.0),
            radius: 5.0,
        }],
        spheres: vec![],
    };

    let output_dir = "outputs/3d";
    fs::create_dir_all(output_dir).unwrap();

    // --- Generate and save 3D plot ---
    println!("Plotting 3D cylinder...");
    let plot_output_path = format!("{}/basic_cylinder_plot.png", output_dir);
    if let Err(e) = plot_3d_system(&system_3d, &plot_output_path) {
        eprintln!("Error plotting 3D system: {}", e);
    }

    // --- Generate and save STL mesh ---
    println!("Generating mesh...");
    let triangles = generate_mesh_from_system(&system_3d);
    let stl_output_path = format!("{}/basic_cylinder.stl", output_dir);
    if let Err(e) = write_stl(&stl_output_path, &triangles) {
        eprintln!("Error writing STL file: {}", e);
    }
} 