use pyvismil::cfd::run_simulation;
use pyvismil::config::{CfdConfig, GeometryConfig};
use pyvismil::geometry::{create_geometry, SplitType};
use pyvismil::visualizations::plot_cfd_results;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating geometry...");
    let splits = vec![
        SplitType::Bifurcation,
        SplitType::Trifurcation,
        SplitType::Bifurcation,
    ];
    let geo_config = GeometryConfig::default();
    let system = create_geometry((200.0, 200.0), &splits, &geo_config);

    println!("Running simulation...");
    let cfd_config = CfdConfig::default();
    let results = run_simulation(&system, &cfd_config)?;

    println!("Plotting CFD results...");
    let output_dir = "outputs/cfd/mixed/bifurcation_trifurcation_bifurcation";
    std::fs::create_dir_all(output_dir)?;
    plot_cfd_results(&results, output_dir)?;

    Ok(())
} 