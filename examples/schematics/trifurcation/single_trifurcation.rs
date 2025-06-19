use pyvismil::{
    visualizations::plot_geometry,
    geometry::{create_geometry, SplitType},
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example_name = "single_trifurcation";
    let output_dir = format!("outputs/schematics/trifurcation/{}", example_name);
    fs::create_dir_all(&output_dir)?;

    let output_path = format!("{}/layout.png", output_dir);
    let box_dimensions = (127.0, 85.0);
    let splits = [SplitType::Trifurcation];

    println!(
        "Generating dynamic geometry with {} split(s)...",
        splits.len()
    );
    let channel_system = create_geometry(box_dimensions, &splits);

    println!("Plotting geometry to {}...", output_path);
    plot_geometry(&channel_system, &output_path)?;

    Ok(())
} 