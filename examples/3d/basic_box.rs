use pyvismil::{
    geometry::mod_3d::{ChannelSystem3D, Volume},
    mesh::{generate_mesh_from_system, write_stl},
    visualizations::plot_3d_system,
};
use std::fs;

fn main() {
    println!("Generating 3D box...");

    let system_3d = ChannelSystem3D {
        box_volume: Volume {
            min_corner: (0.0, 0.0, 0.0),
            max_corner: (127.0, 85.0, 44.0),
        },
        cylinders: vec![],
        spheres: vec![],
    };

    let output_dir = "outputs/3d";
    fs::create_dir_all(output_dir).unwrap();
    
    // --- Generate and save 3D plot ---
    println!("Plotting 3D box...");
    let plot_output_path = format!("{}/box_plot.png", output_dir);
    if let Err(e) = plot_3d_system(&system_3d, &plot_output_path) {
        eprintln!("Error plotting 3D box: {}", e);
    }

    // --- Generate and save STL mesh ---
    println!("Generating mesh...");
    let triangles = generate_mesh_from_system(&system_3d);
    let stl_output_path = format!("{}/box.stl", output_dir);
    if let Err(e) = write_stl(&stl_output_path, &triangles) {
        eprintln!("Error writing STL file: {}", e);
    }
} 