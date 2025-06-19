use pyvismil::{
    cfd::flow::run_simulation,
    geometry::{create_geometry, SplitType},
    visualizations::cfd::plot_cfd_results,
};

fn main() {
    println!("Generating geometry...");
    let box_dims = (127.0, 85.0);
    let splits = vec![SplitType::Bifurcation, SplitType::Trifurcation];
    let system = create_geometry(box_dims, &splits);
    println!("Running simulation...");
    let results = run_simulation(&system);

    println!("Plotting CFD results...");
    if let Err(e) = plot_cfd_results(&results, "outputs/cfd/mixed/bifurcation_trifurcation") {
        eprintln!("Error plotting CFD results: {}", e);
    }
} 