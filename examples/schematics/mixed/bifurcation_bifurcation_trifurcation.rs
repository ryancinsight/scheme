use pyvismil::{
    visualizations::plot_geometry,
    geometry::{create_geometry, SplitType},
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example_name = "bifurcation_bifurcation_trifurcation";
    let output_dir = format!("outputs/schematics/mixed/{}", example_name);
    fs::create_dir_all(&output_dir)?;

    let output_path = format!("{}/layout.png", output_dir);
    let box_dimensions = (200.0, 150.0);
    let splits = [
        SplitType::Bifurcation,
        SplitType::Bifurcation,
        SplitType::Trifurcation,
    ];

    println!(
        "Generating dynamic geometry with {} splits...",
        splits.len()
    );
    let channel_system = create_geometry(box_dimensions, &splits);

    println!("Plotting geometry to {}...", output_path);
    plot_geometry(&channel_system, &output_path)?;

    Ok(())
} 