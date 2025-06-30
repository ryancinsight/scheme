use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::generator::create_geometry,
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/serpentine").unwrap();

    let config = GeometryConfig::default();

    let serpentine_config = SerpentineConfig {
        fill_factor: 1.0,
        wavelength_factor: 1.5,
        gaussian_width_factor: 10.0,
        wave_density_factor: 3.0,
    };
    let channel_config = ChannelTypeConfig::AllSerpentine(serpentine_config);

    // No splits results in a single channel
    let splits = vec![];
    let system = create_geometry((20.0, 10.0), &splits, &config, &channel_config);

    let output_path = "outputs/serpentine/single_serpentine_channel.png";
    plot_geometry(&system, output_path).unwrap();
    println!("Generated single serpentine channel schematic: {}", output_path);
} 