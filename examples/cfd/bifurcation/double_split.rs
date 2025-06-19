use pyvismil::{
    cfd::simulation::run_simulation,
    config::{CfdConfig, GeometryConfig},
    geometry::{create_geometry, SplitType},
    visualizations::cfd::plot_cfd_results,
};
use std::fs;

fn main() {
    println!("Generating geometry...");
    let box_dims = (127.0, 85.0);
    let splits = vec![SplitType::Bifurcation, SplitType::Bifurcation];
    let geo_config = GeometryConfig::default();
    let system = create_geometry(box_dims, &splits, &geo_config);

    println!("Running simulation...");
    let cfd_config = CfdConfig::default();
    let results = run_simulation(&system, &cfd_config).expect("Simulation failed");

    println!("Plotting CFD results...");
    let output_dir = "outputs/cfd/bifurcation/double_split";
    fs::create_dir_all(output_dir).unwrap();

    if let Err(e) = plot_cfd_results(&results, output_dir) {
        eprintln!("Error plotting CFD results: {}", e);
    }
} 