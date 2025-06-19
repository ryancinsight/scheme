use pyvismil::geometry::{create_geometry, SplitType};
use pyvismil::visualizations::plot_geometry;
use pyvismil::config::GeometryConfig;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let splits = vec![
        SplitType::Bifurcation,
        SplitType::Trifurcation,
        SplitType::Bifurcation,
    ];
    println!("Generating dynamic geometry with {} splits...", splits.len());
    let geo_config = GeometryConfig::default();
    let system = create_geometry((200.0, 200.0), &splits, &geo_config);

    let output_dir = "outputs/schematics/mixed/bifurcation_trifurcation_bifurcation";
    fs::create_dir_all(output_dir)?;
    let output_path = format!("{}/layout.png", output_dir);

    println!("Plotting geometry to {}...", output_path);
    plot_geometry(&system, &output_path)?;
    Ok(())
} 