use pyvismil::{
    visualizations::plot_cfd_result,
    geometry::{create_geometry, SplitType},
    cfd::flow::trace_flow,
};
use std::fs;

fn main() {
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Trifurcation];
    let system = create_geometry(box_dims, &splits);

    let initial_flow_rate = 1.0;
    let flow_results = trace_flow(&system, initial_flow_rate);

    let output_dir = "outputs/cfd/trifurcation/single_trifurcation";
    fs::create_dir_all(output_dir).unwrap();
    let output_path = format!("{}/layout.png", output_dir);

    plot_cfd_result(&system, &output_path, &flow_results).unwrap();
} 