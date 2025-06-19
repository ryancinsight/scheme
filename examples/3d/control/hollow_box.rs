//! examples/3d/control/hollow_box.rs

use pyvismil::geometry::mod_3d::{ChannelSystem3D, Cylinder, Volume};
use pyvismil::mesh::{hollow_out_system, write_stl};
use pyvismil::visualizations::plot_3d_system;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/control";
    fs::create_dir_all(output_dir)?;

    // Define a bounding box volume
    let box_volume = Volume {
        min_corner: (0.0, 0.0, 0.0),
        max_corner: (127.0, 85.0, 44.0),
    };

    // Define a cylinder to subtract
    let cylinder = Cylinder {
        start: (0.0, 42.5, 22.0),
        end: (127.0, 42.5, 22.0),
        radius: 15.0,
    };

    // Create a 3D system for visualization purposes
    let system_3d = ChannelSystem3D {
        box_volume: box_volume.clone(),
        cylinders: vec![cylinder.clone()],
    };

    // Generate the hollow mesh
    let triangles = hollow_out_system(&system_3d)?;

    // Save the final mesh to an STL file
    let stl_path = format!("{}/hollow_box.stl", output_dir);
    write_stl(&stl_path, &triangles)?;

    // Also save a plot for visualization of the components
    let plot_path = format!("{}/hollow_box_plot.png", output_dir);
    plot_3d_system(&system_3d, &plot_path)?;

    Ok(())
} 