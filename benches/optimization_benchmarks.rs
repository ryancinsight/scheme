use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, OptimizationProfile},
};

/// Benchmark serpentine optimization performance across different profiles
fn bench_optimization_profiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_profiles");
    group.sample_size(20); // Reduce sample size for slower optimization benchmarks
    
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Test different optimization profiles
    let profiles = vec![
        ("fast", OptimizationProfile::Fast),
        ("balanced", OptimizationProfile::Balanced),
        ("thorough", OptimizationProfile::Thorough),
    ];
    
    for (profile_name, profile) in profiles {
        let serpentine_config = SerpentineConfig {
            optimization_enabled: true,
            optimization_profile: profile,
            target_fill_ratio: 0.9,
            ..SerpentineConfig::default()
        };
        
        group.bench_with_input(
            BenchmarkId::new("serpentine_optimization", profile_name),
            &serpentine_config,
            |b, serp_config| {
                b.iter(|| {
                    create_geometry(
                        black_box(box_dims),
                        black_box(&splits),
                        black_box(&config),
                        black_box(&ChannelTypeConfig::AllSerpentine(*serp_config)),
                    )
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark optimization performance vs complexity
fn bench_optimization_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_complexity");
    group.sample_size(20);
    
    let config = GeometryConfig::default();
    let box_dims = (300.0, 150.0);
    
    // Test different pattern complexities
    let patterns = vec![
        ("single_bifurcation", vec![SplitType::Bifurcation]),
        ("double_bifurcation", vec![SplitType::Bifurcation, SplitType::Bifurcation]),
        ("mixed_complex", vec![SplitType::Bifurcation, SplitType::Trifurcation]),
    ];
    
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        optimization_profile: OptimizationProfile::Balanced,
        target_fill_ratio: 0.9,
        ..SerpentineConfig::default()
    };
    
    for (pattern_name, pattern) in patterns {
        group.bench_with_input(
            BenchmarkId::new("optimization_vs_complexity", pattern_name),
            &pattern,
            |b, pat| {
                b.iter(|| {
                    create_geometry(
                        black_box(box_dims),
                        black_box(pat),
                        black_box(&config),
                        black_box(&ChannelTypeConfig::AllSerpentine(serpentine_config)),
                    )
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark optimization parameter sensitivity
fn bench_optimization_parameters(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_parameters");
    group.sample_size(20);
    
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Test different target fill ratios
    let fill_ratios = vec![0.8, 0.85, 0.9, 0.95];
    
    for &fill_ratio in &fill_ratios {
        let serpentine_config = SerpentineConfig {
            optimization_enabled: true,
            optimization_profile: OptimizationProfile::Balanced,
            target_fill_ratio: fill_ratio,
            ..SerpentineConfig::default()
        };
        
        group.bench_with_input(
            BenchmarkId::new("target_fill_ratio", format!("{:.0}%", fill_ratio * 100.0)),
            &serpentine_config,
            |b, serp_config| {
                b.iter(|| {
                    create_geometry(
                        black_box(box_dims),
                        black_box(&splits),
                        black_box(&config),
                        black_box(&ChannelTypeConfig::AllSerpentine(*serp_config)),
                    )
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark optimization vs non-optimization performance
fn bench_optimization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_overhead");
    
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Non-optimized serpentine
    let standard_config = SerpentineConfig {
        optimization_enabled: false,
        ..SerpentineConfig::default()
    };
    
    group.bench_function("standard_serpentine", |b| {
        b.iter(|| {
            create_geometry(
                black_box(box_dims),
                black_box(&splits),
                black_box(&config),
                black_box(&ChannelTypeConfig::AllSerpentine(standard_config)),
            )
        })
    });
    
    // Optimized serpentine (Fast profile)
    let optimized_config = SerpentineConfig {
        optimization_enabled: true,
        optimization_profile: OptimizationProfile::Fast,
        target_fill_ratio: 0.9,
        ..SerpentineConfig::default()
    };
    
    group.bench_function("optimized_serpentine_fast", |b| {
        b.iter(|| {
            create_geometry(
                black_box(box_dims),
                black_box(&splits),
                black_box(&config),
                black_box(&ChannelTypeConfig::AllSerpentine(optimized_config)),
            )
        })
    });
    
    group.finish();
}

/// Benchmark memory usage during optimization
fn bench_optimization_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_memory");
    group.sample_size(20);
    
    let config = GeometryConfig::default();
    let box_dims = (400.0, 200.0);
    let splits = vec![SplitType::Bifurcation, SplitType::Trifurcation];
    
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        optimization_profile: OptimizationProfile::Thorough,
        target_fill_ratio: 0.95,
        ..SerpentineConfig::default()
    };
    
    group.bench_function("memory_intensive_optimization", |b| {
        b.iter(|| {
            let system = create_geometry(
                black_box(box_dims),
                black_box(&splits),
                black_box(&config),
                black_box(&ChannelTypeConfig::AllSerpentine(serpentine_config)),
            );
            
            // Force evaluation of the system to ensure memory allocation
            black_box(system.channels.len());
            black_box(system.nodes.len());
        })
    });
    
    group.finish();
}

/// Benchmark optimization convergence characteristics
fn bench_optimization_convergence(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_convergence");
    group.sample_size(15); // Small sample size for thorough optimization
    
    let config = GeometryConfig::default();
    let box_dims = (300.0, 150.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Test convergence with different initial parameters
    let initial_configs = vec![
        ("default_start", SerpentineConfig::default()),
        ("aggressive_start", SerpentineConfig {
            wavelength_factor: 4.0,
            wave_density_factor: 3.0,
            fill_factor: 0.9,
            ..SerpentineConfig::default()
        }),
        ("conservative_start", SerpentineConfig {
            wavelength_factor: 2.0,
            wave_density_factor: 1.5,
            fill_factor: 0.7,
            ..SerpentineConfig::default()
        }),
    ];
    
    for (config_name, base_config) in initial_configs {
        let optimized_config = SerpentineConfig {
            optimization_enabled: true,
            optimization_profile: OptimizationProfile::Thorough,
            target_fill_ratio: 0.9,
            ..base_config
        };
        
        group.bench_with_input(
            BenchmarkId::new("convergence_from", config_name),
            &optimized_config,
            |b, serp_config| {
                b.iter(|| {
                    create_geometry(
                        black_box(box_dims),
                        black_box(&splits),
                        black_box(&config),
                        black_box(&ChannelTypeConfig::AllSerpentine(*serp_config)),
                    )
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    optimization_benches,
    bench_optimization_profiles,
    bench_optimization_complexity,
    bench_optimization_parameters,
    bench_optimization_overhead,
    bench_optimization_memory,
    bench_optimization_convergence
);

criterion_main!(optimization_benches);
