use pyvismil::{
    visualizations::plot_geometry,
    geometry::{create_geometry, SplitType},
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example_name = "triple_split";
    let output_dir = format!("outputs/schematics/bifurcation/{}", example_name);
    fs::create_dir_all(&output_dir)?;

    let output_path = format!("{}/layout.png", output_dir);
    let box_dimensions = (127.0, 85.0);
    let splits = [SplitType::Bifurcation; 3];

    println!(
        "Generating dynamic geometry with {} splits...",
        splits.len()
    );
    let channel_system = create_geometry(box_dimensions, &splits);

    println!("Plotting geometry to {}...", output_path);
    plot_geometry(&channel_system, &output_path)?;

    Ok(())
} 