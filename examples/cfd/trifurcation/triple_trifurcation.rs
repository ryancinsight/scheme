use pyvismil::{
    cfd::flow::run_simulation,
    geometry::{create_geometry, SplitType},
    visualizations::cfd::plot_cfd_results,
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
    let system = create_geometry(box_dims, &splits);
    println!("Running simulation...");
    let results = run_simulation(&system);

    println!("Plotting CFD results...");
    let output_dir = "outputs/cfd/trifurcation/triple_trifurcation";
    fs::create_dir_all(output_dir).unwrap();

    if let Err(e) = plot_cfd_results(&results, output_dir) {
        eprintln!("Error plotting CFD results: {}", e);
    }
} 