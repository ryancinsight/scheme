use pyvismil::{
    config::GeometryConfig,
    geometry::{create_geometry, SplitType},
    visualizations::plot_geometry,
};
use std::fs;

fn main() {
    println!("Generating geometry...");
    let box_dims = (127.0, 85.0);
    let splits = vec![
        SplitType::Trifurcation,
        SplitType::Trifurcation,
        SplitType::Bifurcation,
    ];
    let geo_config = GeometryConfig::default();
    let system = create_geometry(box_dims, &splits, &geo_config);

    println!("Plotting geometry...");
    let output_dir = "outputs/schematics/mixed/trifurcation_trifurcation_bifurcation";
    fs::create_dir_all(output_dir).unwrap();
    let output_path = format!("{}/layout.png", output_dir);

    if let Err(e) = plot_geometry(&system, &output_path) {
        eprintln!("Error plotting geometry: {}", e);
    }
} 