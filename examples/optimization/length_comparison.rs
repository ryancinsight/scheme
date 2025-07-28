use scheme::{
    config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig, presets},
    geometry::{generator::create_geometry, SplitType, optimization::calculate_path_length},
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/optimization").unwrap();

    let config = GeometryConfig::default();

    // Create standard serpentine configuration (no optimization)
    let standard_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        ..SerpentineConfig::default()
    };

    // Create optimized serpentine configuration
    let optimized_config = SerpentineConfig {
        fill_factor: 0.8,
        wavelength_factor: 3.0,
        gaussian_width_factor: 6.0,
        wave_density_factor: 2.0,
        wave_phase_direction: 0.0,
        optimization_enabled: true,
        target_fill_ratio: 0.95, // Aggressive optimization
        optimization_profile: scheme::config::OptimizationProfile::Balanced,
        ..SerpentineConfig::default()
    };

    // Test different scenarios
    let scenarios = vec![
        ("single_channel", vec![], (200.0, 100.0)),
        ("bifurcation", vec![SplitType::Bifurcation], (200.0, 100.0)),
        ("double_bifurcation", vec![SplitType::Bifurcation, SplitType::Bifurcation], (300.0, 150.0)),
        ("trifurcation", vec![SplitType::Trifurcation], (250.0, 120.0)),
        ("mixed_splits", vec![SplitType::Bifurcation, SplitType::Trifurcation], (350.0, 180.0)),
    ];

    println!("Serpentine Channel Length Optimization Comparison");
    println!("=================================================");
    println!();

    for (scenario_name, splits, box_dims) in scenarios {
        println!("Scenario: {}", scenario_name);
        println!("Box dimensions: {:.1} x {:.1}", box_dims.0, box_dims.1);
        println!("Splits: {:?}", splits);

        // Generate standard system
        let standard_system = create_geometry(
            box_dims,
            &splits,
            &config,
            &ChannelTypeConfig::AllSerpentine(standard_config),
        );

        // Generate optimized system
        let optimized_system = create_geometry(
            box_dims,
            &splits,
            &config,
            &ChannelTypeConfig::AllSerpentine(optimized_config),
        );

        // Calculate total path lengths
        let standard_total_length = calculate_total_serpentine_length(&standard_system);
        let optimized_total_length = calculate_total_serpentine_length(&optimized_system);

        // Calculate improvement
        let improvement = if standard_total_length > 0.0 {
            ((optimized_total_length - standard_total_length) / standard_total_length) * 100.0
        } else {
            0.0
        };

        println!("  Standard total length:  {:.2} mm", standard_total_length);
        println!("  Optimized total length: {:.2} mm", optimized_total_length);
        println!("  Length improvement:     {:.1}%", improvement);

        // Generate visualizations
        let standard_output = format!("outputs/optimization/{}_standard.png", scenario_name);
        let optimized_output = format!("outputs/optimization/{}_optimized.png", scenario_name);

        plot_geometry(&standard_system, &standard_output).unwrap();
        plot_geometry(&optimized_system, &optimized_output).unwrap();

        println!("  Generated: {} and {}", standard_output, optimized_output);
        println!();
    }

    // Demonstrate preset optimization
    println!("Preset Optimization Comparison");
    println!("==============================");
    
    let preset_scenarios = vec![
        ("smooth_serpentine", presets::smooth_serpentine()),
        ("high_density_serpentine", presets::high_density_serpentine()),
        ("optimized_serpentine", presets::optimized_serpentine()),
    ];

    let test_splits = vec![SplitType::Bifurcation, SplitType::Bifurcation];
    let test_box_dims = (300.0, 150.0);

    for (preset_name, preset_config) in preset_scenarios {
        let system = create_geometry(
            test_box_dims,
            &test_splits,
            &config,
            &ChannelTypeConfig::AllSerpentine(preset_config),
        );

        let total_length = calculate_total_serpentine_length(&system);
        let optimization_status = if preset_config.optimization_enabled {
            format!("ENABLED (target: {:.1}%)", preset_config.target_fill_ratio * 100.0)
        } else {
            "DISABLED".to_string()
        };

        println!("Preset: {}", preset_name);
        println!("  Optimization: {}", optimization_status);
        println!("  Total length: {:.2} mm", total_length);

        let output_path = format!("outputs/optimization/preset_{}.png", preset_name);
        plot_geometry(&system, &output_path).unwrap();
        println!("  Generated: {}", output_path);
        println!();
    }

    // Performance comparison
    println!("Performance Analysis");
    println!("===================");
    
    let large_system_splits = vec![
        SplitType::Bifurcation,
        SplitType::Trifurcation,
        SplitType::Bifurcation,
    ];
    let large_box_dims = (500.0, 250.0);

    // Time standard generation
    let start_time = std::time::Instant::now();
    let standard_large = create_geometry(
        large_box_dims,
        &large_system_splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(standard_config),
    );
    let standard_duration = start_time.elapsed();

    // Time optimized generation
    let start_time = std::time::Instant::now();
    let optimized_large = create_geometry(
        large_box_dims,
        &large_system_splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(optimized_config),
    );
    let optimized_duration = start_time.elapsed();

    let standard_large_length = calculate_total_serpentine_length(&standard_large);
    let optimized_large_length = calculate_total_serpentine_length(&optimized_large);
    let large_improvement = ((optimized_large_length - standard_large_length) / standard_large_length) * 100.0;

    println!("Large system (3 splits, {} channels):", standard_large.channels.len());
    println!("  Standard generation time:  {:?}", standard_duration);
    println!("  Optimized generation time: {:?}", optimized_duration);
    println!("  Standard total length:     {:.2} mm", standard_large_length);
    println!("  Optimized total length:    {:.2} mm", optimized_large_length);
    println!("  Length improvement:        {:.1}%", large_improvement);

    plot_geometry(&standard_large, "outputs/optimization/large_system_standard.png").unwrap();
    plot_geometry(&optimized_large, "outputs/optimization/large_system_optimized.png").unwrap();

    println!();
    println!("All optimization comparison files generated in outputs/optimization/");
}

/// Calculate total length of all serpentine channels in a system
fn calculate_total_serpentine_length(system: &scheme::geometry::ChannelSystem) -> f64 {
    system.channels.iter()
        .map(|channel| {
            match &channel.channel_type {
                scheme::geometry::ChannelType::Serpentine { path } => {
                    calculate_path_length(path)
                },
                _ => 0.0, // Non-serpentine channels don't contribute
            }
        })
        .sum()
}
