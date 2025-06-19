use pyvismil::{
    drawing::plot_geometry,
    geometry::{create_geometry, SplitType},
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example_name = "single_trifurcation";
    let output_dir = format!("outputs/trifurcation/{}", example_name);
    fs::create_dir_all(&output_dir)?;

    let output_path = format!("{}/layout.png", output_dir);
    let box_dimensions = (127.0, 85.0);
    const NUM_SPLITS: u32 = 1;

    println!(
        "Generating trifurcation geometry with {} split(s)...",
        NUM_SPLITS
    );
    let channel_system = create_geometry(box_dimensions, NUM_SPLITS, SplitType::Trifurcation);

    println!("Plotting geometry to {}...", output_path);
    plot_geometry(&channel_system, &output_path)?;

    Ok(())
} 