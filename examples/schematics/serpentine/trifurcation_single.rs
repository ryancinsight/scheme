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

    let splits = vec![SplitType::Trifurcation];
    let system = create_geometry((20.0, 10.0), &splits, &config, &channel_config);

    let output_path = "outputs/serpentine/trifurcation_single.png";
    plot_geometry(&system, output_path).unwrap();
    println!("Generated serpentine (single trifurcation) schematic: {}", output_path);
}
