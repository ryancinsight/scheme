use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig},
    visualizations::schematic::plot_geometry,
};
use std::fs;
use std::path::Path;

/// Setup function to create test geometries
fn create_test_geometries() -> Vec<(String, scheme::geometry::ChannelSystem)> {
    let config = GeometryConfig::default();
    let mut geometries = Vec::new();
    
    // Simple geometry
    let simple = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    geometries.push(("simple_straight".to_string(), simple));
    
    // Complex geometry with serpentine channels
    let complex_serpentine = create_geometry(
        (400.0, 200.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(SerpentineConfig::default()),
    );
    geometries.push(("complex_serpentine".to_string(), complex_serpentine));
    
    // Complex geometry with arc channels
    let complex_arc = create_geometry(
        (400.0, 200.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllArcs(ArcConfig::default()),
    );
    geometries.push(("complex_arc".to_string(), complex_arc));
    
    // Very complex mixed geometry
    let very_complex = create_geometry(
        (800.0, 400.0),
        &[
            SplitType::Bifurcation, 
            SplitType::Trifurcation, 
            SplitType::Bifurcation, 
            SplitType::Trifurcation
        ],
        &config,
        &ChannelTypeConfig::Smart {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
        },
    );
    geometries.push(("very_complex_smart".to_string(), very_complex));
    
    geometries
}

/// Benchmark PNG visualization rendering performance
fn bench_png_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("png_rendering");
    let geometries = create_test_geometries();
    
    // Ensure output directory exists
    fs::create_dir_all("target/bench_outputs").ok();
    
    for (name, geometry) in &geometries {
        group.bench_with_input(
            BenchmarkId::new("png_render", name),
            geometry,
            |b, geom| {
                let output_path = format!("target/bench_outputs/bench_{}.png", name);
                b.iter(|| {
                    plot_geometry(black_box(geom), black_box(&output_path))
                        .expect("PNG rendering should succeed")
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark rendering performance for different image sizes
fn bench_rendering_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering_sizes");
    
    let config = GeometryConfig::default();
    let pattern = vec![SplitType::Bifurcation, SplitType::Trifurcation];
    
    // Test different output image dimensions by varying geometry box size
    let sizes = vec![
        ("small", (200.0, 100.0)),
        ("medium", (400.0, 200.0)),
        ("large", (800.0, 400.0)),
        ("xlarge", (1600.0, 800.0)),
    ];
    
    fs::create_dir_all("target/bench_outputs").ok();
    
    for (size_name, box_dims) in &sizes {
        let geometry = create_geometry(
            *box_dims,
            &pattern,
            &config,
            &ChannelTypeConfig::AllStraight,
        );
        
        group.bench_with_input(
            BenchmarkId::new("size_scaling", size_name),
            &geometry,
            |b, geom| {
                let output_path = format!("target/bench_outputs/bench_size_{}.png", size_name);
                b.iter(|| {
                    plot_geometry(black_box(geom), black_box(&output_path))
                        .expect("PNG rendering should succeed")
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark channel count impact on rendering performance
fn bench_channel_count_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_count_impact");
    
    let config = GeometryConfig::default();
    let box_dims = (400.0, 200.0);
    
    // Create patterns with increasing channel counts
    let patterns = vec![
        ("1_split", vec![SplitType::Bifurcation]),
        ("2_splits", vec![SplitType::Bifurcation, SplitType::Bifurcation]),
        ("3_splits", vec![SplitType::Bifurcation, SplitType::Bifurcation, SplitType::Bifurcation]),
        ("mixed_2", vec![SplitType::Bifurcation, SplitType::Trifurcation]),
        ("mixed_3", vec![SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation]),
        ("trifurcation_heavy", vec![SplitType::Trifurcation, SplitType::Trifurcation]),
    ];
    
    fs::create_dir_all("target/bench_outputs").ok();
    
    for (pattern_name, pattern) in &patterns {
        let geometry = create_geometry(
            box_dims,
            pattern,
            &config,
            &ChannelTypeConfig::AllStraight,
        );
        
        let channel_count = geometry.channels.len();
        let benchmark_name = format!("{}_({}_channels)", pattern_name, channel_count);
        
        group.bench_with_input(
            BenchmarkId::new("channel_count", &benchmark_name),
            &geometry,
            |b, geom| {
                let output_path = format!("target/bench_outputs/bench_channels_{}.png", pattern_name);
                b.iter(|| {
                    plot_geometry(black_box(geom), black_box(&output_path))
                        .expect("PNG rendering should succeed")
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark serpentine path complexity impact on rendering
fn bench_serpentine_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("serpentine_complexity");
    
    let config = GeometryConfig::default();
    let box_dims = (400.0, 200.0);
    let pattern = vec![SplitType::Bifurcation, SplitType::Bifurcation];
    
    // Test different serpentine configurations
    let serpentine_configs = vec![
        ("low_density", SerpentineConfig {
            fill_factor: 0.5,
            wavelength_factor: 6.0,
            gaussian_width_factor: 8.0,
            wave_density_factor: 1.0,
            wave_phase_direction: 0.0,
            optimization_enabled: false,
            optimization_profile: scheme::config::OptimizationProfile::Balanced,
            target_fill_ratio: 0.9,
        }),
        ("medium_density", SerpentineConfig {
            fill_factor: 0.7,
            wavelength_factor: 4.0,
            gaussian_width_factor: 6.0,
            wave_density_factor: 1.5,
            wave_phase_direction: 0.0,
            optimization_enabled: false,
            optimization_profile: scheme::config::OptimizationProfile::Balanced,
            target_fill_ratio: 0.9,
        }),
        ("high_density", SerpentineConfig {
            fill_factor: 0.9,
            wavelength_factor: 2.0,
            gaussian_width_factor: 4.0,
            wave_density_factor: 2.5,
            wave_phase_direction: 0.0,
            optimization_enabled: false,
            optimization_profile: scheme::config::OptimizationProfile::Balanced,
            target_fill_ratio: 0.9,
        }),
    ];
    
    fs::create_dir_all("target/bench_outputs").ok();
    
    for (config_name, serpentine_config) in &serpentine_configs {
        let geometry = create_geometry(
            box_dims,
            &pattern,
            &config,
            &ChannelTypeConfig::AllSerpentine(*serpentine_config),
        );
        
        group.bench_with_input(
            BenchmarkId::new("serpentine_complexity", config_name),
            &geometry,
            |b, geom| {
                let output_path = format!("target/bench_outputs/bench_serpentine_{}.png", config_name);
                b.iter(|| {
                    plot_geometry(black_box(geom), black_box(&output_path))
                        .expect("PNG rendering should succeed")
                })
            },
        );
    }
    
    group.finish();
}

/// Cleanup function to remove benchmark output files
#[allow(dead_code)]
fn cleanup_bench_outputs() {
    if Path::new("target/bench_outputs").exists() {
        fs::remove_dir_all("target/bench_outputs").ok();
    }
}

criterion_group!(
    benches,
    bench_png_rendering,
    bench_rendering_sizes,
    bench_channel_count_impact,
    bench_serpentine_complexity
);
criterion_main!(benches);
