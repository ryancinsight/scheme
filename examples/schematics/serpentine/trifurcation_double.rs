use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/serpentine").unwrap();

    let config = GeometryConfig::default();
    let channel_config = ChannelTypeConfig::AllSerpentine(SerpentineConfig::default());

    let splits = vec![SplitType::Trifurcation, SplitType::Trifurcation];
    let system = create_geometry((30.0, 15.0), &splits, &config, &channel_config);

    let output_path = "outputs/serpentine/trifurcation_double.png";
    plot_geometry(&system, output_path).unwrap();
    println!("Generated serpentine (double trifurcation) schematic: {}", output_path);
} 