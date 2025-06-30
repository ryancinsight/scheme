use scheme::{
    config::{GeometryConfig, ChannelTypeConfig},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    println!("Generating geometry...");
    let box_dims = (127.0, 85.0);
    let splits = vec![
        SplitType::Bifurcation,
        SplitType::Bifurcation,
        SplitType::Bifurcation,
        SplitType::Bifurcation,
        SplitType::Bifurcation,
    ];
    let geo_config = GeometryConfig::default();
    let channel_config = ChannelTypeConfig::AllStraight;
    let system = create_geometry(box_dims, &splits, &geo_config, &channel_config);

    println!("Plotting geometry...");
    let output_dir = "outputs/schematics/bifurcation";
    fs::create_dir_all(output_dir).unwrap();
    let output_path = format!("{}/five_split.png", output_dir);

    if let Err(e) = plot_geometry(&system, &output_path) {
        eprintln!("Error plotting geometry: {}", e);
    }
} 
