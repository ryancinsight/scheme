use scheme::{
    config::{ChannelTypeConfig, GeometryConfig},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/serpentine").unwrap();

    let config = GeometryConfig::default();

    let channel_config = ChannelTypeConfig::default(); // MixedByPosition

    let splits = vec![SplitType::Bifurcation, SplitType::Bifurcation];
    let system = create_geometry((20.0, 10.0), &splits, &config, &channel_config);

    let output_path = "outputs/serpentine/bifurcation_double.png";
    plot_geometry(&system, output_path).unwrap();
    println!("Generated serpentine (double bifurcation) schematic: {}", output_path);
} 