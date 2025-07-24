use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig, presets},
    geometry::{generator::create_geometry, SplitType},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/serpentine").unwrap();

    let config = GeometryConfig::default();

    // Create three different serpentine configurations to demonstrate wave phase direction control
    
    // 1. Auto-symmetric (default) - perfect mirror symmetry
    let auto_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0, // Auto-determine for perfect symmetry
        optimization_enabled: false,
        target_fill_ratio: 0.9,
    };
    
    // 2. Force all waves to start with inward phase
    let inward_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: -1.0, // Force inward phase
        optimization_enabled: false,
        target_fill_ratio: 0.9,
    };
    
    // 3. Force all waves to start with outward phase
    let outward_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 1.0, // Force outward phase
        optimization_enabled: false,
        target_fill_ratio: 0.9,
    };

    // Create systems with bifurcations to show symmetry effects
    let splits = vec![SplitType::Bifurcation, SplitType::Bifurcation];
    
    // Generate auto-symmetric serpentines
    let auto_system = create_geometry((30.0, 15.0), &splits, &config, &ChannelTypeConfig::AllSerpentine(auto_config));
    let auto_output = "outputs/serpentine/wave_phase_auto.png";
    plot_geometry(&auto_system, auto_output).unwrap();
    println!("Generated auto-symmetric serpentines: {}", auto_output);
    
    // Generate inward-phase serpentines
    let inward_system = create_geometry((30.0, 15.0), &splits, &config, &ChannelTypeConfig::AllSerpentine(inward_config));
    let inward_output = "outputs/serpentine/wave_phase_inward.png";
    plot_geometry(&inward_system, inward_output).unwrap();
    println!("Generated inward-phase serpentines: {}", inward_output);
    
    // Generate outward-phase serpentines
    let outward_system = create_geometry((30.0, 15.0), &splits, &config, &ChannelTypeConfig::AllSerpentine(outward_config));
    let outward_output = "outputs/serpentine/wave_phase_outward.png";
    plot_geometry(&outward_system, outward_output).unwrap();
    println!("Generated outward-phase serpentines: {}", outward_output);

    // Also demonstrate preset configurations
    let preset_inward_system = create_geometry((30.0, 15.0), &splits, &config, &ChannelTypeConfig::AllSerpentine(presets::inward_serpentines()));
    let preset_inward_output = "outputs/serpentine/preset_inward_serpentines.png";
    plot_geometry(&preset_inward_system, preset_inward_output).unwrap();
    println!("Generated preset inward serpentines: {}", preset_inward_output);
    
    let preset_outward_system = create_geometry((30.0, 15.0), &splits, &config, &ChannelTypeConfig::AllSerpentine(presets::outward_serpentines()));
    let preset_outward_output = "outputs/serpentine/preset_outward_serpentines.png";
    plot_geometry(&preset_outward_system, preset_outward_output).unwrap();
    println!("Generated preset outward serpentines: {}", preset_outward_output);

    // Count serpentine channels in each system
    let count_serpentines = |system: &scheme::geometry::ChannelSystem| {
        system.channels.iter().filter(|ch| matches!(ch.channel_type, scheme::geometry::ChannelType::Serpentine { .. })).count()
    };

    println!("\nSerpentine channel counts:");
    println!("  Auto-symmetric: {}", count_serpentines(&auto_system));
    println!("  Inward-phase: {}", count_serpentines(&inward_system));
    println!("  Outward-phase: {}", count_serpentines(&outward_system));
    println!("  Preset inward: {}", count_serpentines(&preset_inward_system));
    println!("  Preset outward: {}", count_serpentines(&preset_outward_system));
}
