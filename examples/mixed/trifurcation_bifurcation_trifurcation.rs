use pyvismil::{
    drawing::plot_geometry,
    geometry::{create_geometry, SplitType},
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example_name = "trifurcation_bifurcation_trifurcation";
    let output_dir = format!("outputs/mixed/{}", example_name);
    fs::create_dir_all(&output_dir)?;

    let output_path = format!("{}/layout.png", output_dir);
    let box_dimensions = (127.0, 85.0);
    let splits = [
        SplitType::Trifurcation,
        SplitType::Bifurcation,
        SplitType::Trifurcation,
    ];

    println!("Generating mixed geometry (trifurcation -> bifurcation -> trifurcation)...");
    let channel_system = create_geometry(box_dimensions, &splits);

    println!("Plotting geometry to {}...", output_path);
    plot_geometry(&channel_system, &output_path)?;

    Ok(())
} 