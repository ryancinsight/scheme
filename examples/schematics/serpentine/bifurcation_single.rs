use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/serpentine").unwrap();

    let config = GeometryConfig::default();

    let serpentine_config = SerpentineConfig {
        fill_factor: 0.9,
        wavelength_factor: 2.0,
        gaussian_width_factor: 8.0,
        wave_density_factor: 3.5,
    };
    let channel_config = ChannelTypeConfig::AllSerpentine(serpentine_config);

    let splits = vec![SplitType::Bifurcation];
    let system = create_geometry((20.0, 10.0), &splits, &config, &channel_config);

    let output_path = "outputs/serpentine/bifurcation_single.png";
    plot_geometry(&system, output_path).unwrap();
    println!("Generated serpentine (single bifurcation) schematic: {}", output_path);
} 