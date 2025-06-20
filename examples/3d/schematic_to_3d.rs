//! examples/3d/schematic_to_3d.rs

use pyvismil::{
    config::{ConversionConfig, GeometryConfig},
    geometry::{convert_2d_to_3d, create_geometry, SplitType},
    mesh::{difference, write_stl},
    visualizations,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "outputs/3d/schematic_to_3d";
    fs::create_dir_all(output_dir)?;

    // 1. Define 2D geometry
    let geo_config = GeometryConfig::default();
    let schematic = create_geometry((200.0, 100.0), &[SplitType::Bifurcation], &geo_config);

    // 2. Convert to 3D
    println!("Converting 2D schematic to 3D system...");
    let conv_config = ConversionConfig::default();
    let system_3d = convert_2d_to_3d(&schematic, &conv_config);

    // 3. Plot the 3D representation
    println!("Plotting 3D system...");
    let plot_path = format!("{}/schematic_to_3d_plot.png", output_dir);
    visualizations::plot_3d_system(&system_3d, &plot_path)?;

    // 4. Generate mesh with boolean difference
    println!("Generating mesh with boolean difference...");
    let final_mesh = difference(&system_3d)?;

    // 5. Write the final mesh to an STL file
    let stl_path = format!("{}/schematic_to_3d.stl", output_dir);
    write_stl(&stl_path, &final_mesh)?;

    println!("Successfully converted schematic to 3D and saved files.");

    Ok(())
} 