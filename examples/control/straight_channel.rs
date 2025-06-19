use pyvismil::{
    cfd::simulation::run_simulation,
    config::{CfdConfig, GeometryConfig},
    geometry::create_geometry,
    visualizations::{plot_cfd_results, plot_geometry},
};
use std::fs;

fn main() {
    println!("Generating control geometry (straight channel)...");
    let box_dims = (127.0, 85.0);
    let splits = vec![]; // No splits
    let geo_config = GeometryConfig::default();
    let system = create_geometry(box_dims, &splits, &geo_config);

    // --- Create Schematic Plot ---
    println!("Plotting schematic...");
    let output_dir = "outputs/control/straight_channel";
    fs::create_dir_all(output_dir).unwrap();
    let schematic_output_path = format!("{}/layout.png", output_dir);
    if let Err(e) = plot_geometry(&system, &schematic_output_path) {
        eprintln!("Error plotting geometry: {}", e);
    }

    // --- Run Simulation and Create CFD Plots ---
    println!("Running simulation...");
    let cfd_config = CfdConfig::default();
    let results = run_simulation(&system, &cfd_config).expect("Simulation failed");

    println!("Plotting CFD results...");
    if let Err(e) = plot_cfd_results(&results, output_dir) {
        eprintln!("Error plotting CFD results: {}", e);
    }
    
    println!("Control example finished.");
} 