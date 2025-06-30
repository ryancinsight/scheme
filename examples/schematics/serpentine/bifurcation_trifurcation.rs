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

    let splits = vec![SplitType::Bifurcation, SplitType::Trifurcation];
    let system = create_geometry((30.0, 15.0), &splits, &config, &channel_config);

    let output_path = "outputs/serpentine/bifurcation_trifurcation.png";
    plot_geometry(&system, output_path).unwrap();
    println!(
        "Generated serpentine (bifurcation-trifurcation) schematic: {}",
        output_path
    );
} 