use pyvismil::{drawing::plot_geometry, geometry::create_dynamic_split_geometry};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example_name = "triple_split";
    let output_dir = format!("outputs/{}", example_name);
    fs::create_dir_all(&output_dir)?;

    let output_path = format!("{}/layout.png", output_dir);
    let box_dimensions = (127.0, 85.0);
    const NUM_SPLITS: u32 = 3;

    println!(
        "Generating dynamic geometry with {} splits...",
        NUM_SPLITS
    );
    let channel_system = create_dynamic_split_geometry(box_dimensions, NUM_SPLITS);

    println!("Plotting geometry to {}...", output_path);
    plot_geometry(&channel_system, &output_path)?;

    Ok(())
} 