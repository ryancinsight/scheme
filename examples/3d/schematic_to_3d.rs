//! examples/3d/schematic_to_3d.rs

use pyvismil::{
    config::{ConversionConfig, GeometryConfig},
    geometry::{convert_2d_to_3d, create_geometry, SplitType},
    mesh::{hollow_out_system, write_stl},
    visualizations::plot_3d_system,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/schematic_to_3d";
    fs::create_dir_all(output_dir)?;

    // 1. Define the 2D geometry
    let splits = [SplitType::Bifurcation, SplitType::Trifurcation];
    let geo_config = GeometryConfig {
        channel_height: 6.0, // This corresponds to a 6mm diameter
        ..Default::default()
    };
    let system_2d = create_geometry((127.0, 85.0), &splits, &geo_config);

    // 2. Convert the 2D geometry to 3D
    println!("Converting 2D schematic to 3D system...");
    let conversion_config = ConversionConfig::default();
    let system_3d = convert_2d_to_3d(&system_2d, &conversion_config);

    // 3. Generate and save the 3D plot
    println!("Plotting 3D system...");
    let plot_path = format!("{}/schematic_to_3d_plot.png", output_dir);
    plot_3d_system(&system_3d, &plot_path)?;

    // 4. Generate and save the STL mesh with boolean subtraction
    println!("Generating mesh with boolean subtraction...");
    let triangles = hollow_out_system(&system_3d)?;
    let stl_path = format!("{}/schematic_to_3d.stl", output_dir);
    write_stl(&stl_path, &triangles)?;

    println!("Successfully converted schematic to 3D and saved files.");
    Ok(())
} 