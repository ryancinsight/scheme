use scheme::{
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
        SplitType::Trifurcation,
    ];
    let geo_config = GeometryConfig::default();
    let system = create_geometry(box_dims, &splits, &geo_config);

    println!("Plotting geometry...");
    let output_dir = "outputs/schematics/trifurcation";
    fs::create_dir_all(output_dir).unwrap();
    let output_path = format!("{}/triple_trifurcation.png", output_dir);

    if let Err(e) = plot_geometry(&system, &output_path) {
        eprintln!("Error plotting geometry: {}", e);
    }
} 
