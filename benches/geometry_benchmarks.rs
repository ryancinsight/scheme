use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, ArcConfig},
};

/// Benchmark geometry generation for different pattern complexities
fn bench_geometry_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("geometry_generation");
    
    // Test different box sizes
    let box_sizes = vec![
        (100.0, 50.0),   // Small
        (200.0, 100.0),  // Medium  
        (400.0, 200.0),  // Large
        (800.0, 400.0),  // Extra Large
    ];
    
    // Test different pattern complexities
    let patterns = vec![
        ("single_bifurcation", vec![SplitType::Bifurcation]),
        ("double_bifurcation", vec![SplitType::Bifurcation, SplitType::Bifurcation]),
        ("triple_bifurcation", vec![SplitType::Bifurcation, SplitType::Bifurcation, SplitType::Bifurcation]),
        ("single_trifurcation", vec![SplitType::Trifurcation]),
        ("double_trifurcation", vec![SplitType::Trifurcation, SplitType::Trifurcation]),
        ("mixed_complex", vec![SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation]),
    ];
    
    let config = GeometryConfig::default();
    
    for box_dims in &box_sizes {
        for (pattern_name, pattern) in &patterns {
            let benchmark_name = format!("{}x{}-{}", box_dims.0 as i32, box_dims.1 as i32, pattern_name);
            let dims = *box_dims;

            group.bench_with_input(
                BenchmarkId::new("straight_channels", &benchmark_name),
                &(dims, pattern),
                |b, (dims, pat)| {
                    b.iter(|| {
                        create_geometry(
                            black_box(*dims),
                            black_box(pat),
                            black_box(&config),
                            black_box(&ChannelTypeConfig::AllStraight),
                        )
                    })
                },
            );

            group.bench_with_input(
                BenchmarkId::new("serpentine_channels", &benchmark_name),
                &(dims, pattern),
                |b, (dims, pat)| {
                    let serpentine_config = ChannelTypeConfig::AllSerpentine(SerpentineConfig::default());
                    b.iter(|| {
                        create_geometry(
                            black_box(*dims),
                            black_box(pat),
                            black_box(&config),
                            black_box(&serpentine_config),
                        )
                    })
                },
            );

            group.bench_with_input(
                BenchmarkId::new("arc_channels", &benchmark_name),
                &(dims, pattern),
                |b, (dims, pat)| {
                    let arc_config = ChannelTypeConfig::AllArcs(ArcConfig::default());
                    b.iter(|| {
                        create_geometry(
                            black_box(*dims),
                            black_box(pat),
                            black_box(&config),
                            black_box(&arc_config),
                        )
                    })
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark channel type strategy selection
fn bench_channel_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_strategies");
    
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let pattern = vec![SplitType::Bifurcation];
    
    // Benchmark different channel type configurations
    let configs = vec![
        ("all_straight", ChannelTypeConfig::AllStraight),
        ("all_serpentine", ChannelTypeConfig::AllSerpentine(SerpentineConfig::default())),
        ("all_arcs", ChannelTypeConfig::AllArcs(ArcConfig::default())),
        ("smart_mixed", ChannelTypeConfig::Smart {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
        }),
        ("mixed_by_position", ChannelTypeConfig::MixedByPosition {
            middle_zone_fraction: 0.4,
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
        }),
    ];
    
    for (config_name, channel_config) in configs {
        group.bench_function(config_name, |b| {
            b.iter(|| {
                create_geometry(
                    black_box(box_dims),
                    black_box(&pattern),
                    black_box(&config),
                    black_box(&channel_config),
                )
            })
        });
    }
    
    group.finish();
}

/// Benchmark memory usage patterns for different geometries
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    let config = GeometryConfig::default();
    
    // Test memory usage for increasingly complex patterns
    let complex_patterns = vec![
        ("simple", vec![SplitType::Bifurcation]),
        ("moderate", vec![SplitType::Bifurcation, SplitType::Bifurcation]),
        ("complex", vec![SplitType::Bifurcation, SplitType::Trifurcation, SplitType::Bifurcation]),
        ("very_complex", vec![
            SplitType::Bifurcation, 
            SplitType::Trifurcation, 
            SplitType::Bifurcation, 
            SplitType::Trifurcation
        ]),
    ];
    
    for (complexity_name, pattern) in complex_patterns {
        group.bench_function(complexity_name, |b| {
            b.iter(|| {
                let system = create_geometry(
                    black_box((400.0, 200.0)),
                    black_box(&pattern),
                    black_box(&config),
                    black_box(&ChannelTypeConfig::Smart {
                        serpentine_config: SerpentineConfig::default(),
                        arc_config: ArcConfig::default(),
                    }),
                );
                // Force evaluation to measure actual memory allocation
                black_box(system.nodes.len() + system.channels.len())
            })
        });
    }
    
    group.finish();
}

/// Benchmark configuration validation performance
fn bench_configuration_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_validation");
    
    group.bench_function("geometry_config_creation", |b| {
        b.iter(|| {
            GeometryConfig::new(
                black_box(0.5),
                black_box(1.0), 
                black_box(0.5)
            )
        })
    });
    
    group.bench_function("serpentine_config_creation", |b| {
        b.iter(|| {
            SerpentineConfig::new(
                black_box(0.8),
                black_box(4.0),
                black_box(6.0),
                black_box(1.5)
            )
        })
    });
    
    group.bench_function("arc_config_creation", |b| {
        b.iter(|| {
            ArcConfig::new(
                black_box(0.3),
                black_box(50)
            )
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_geometry_generation,
    bench_channel_strategies,
    bench_memory_usage,
    bench_configuration_validation
);
criterion_main!(benches);
