use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use scheme::{
    geometry::{
        generator::create_geometry,
        metadata::{MetadataContainer, FlowMetadata, ThermalMetadata, ManufacturingMetadata, OptimizationMetadata, PerformanceMetadata},
        builders::{ChannelBuilder, ChannelExt, NodeExt},
        types::{ChannelType, Channel},
        SplitType,
    },
    config::{GeometryConfig, ChannelTypeConfig},
};

/// Benchmark metadata container operations
fn bench_metadata_container_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_container");
    
    // Benchmark insertion performance
    group.bench_function("insert_single_metadata", |b| {
        b.iter(|| {
            let mut container = MetadataContainer::new();
            let flow_data = FlowMetadata {
                flow_rate: 10.0,
                pressure_drop: 1000.0,
                reynolds_number: 0.1,
                velocity: 0.001,
            };
            container.insert(black_box(flow_data));
            black_box(container);
        })
    });
    
    // Benchmark multiple insertions
    group.bench_function("insert_multiple_metadata", |b| {
        b.iter(|| {
            let mut container = MetadataContainer::new();
            
            container.insert(black_box(FlowMetadata {
                flow_rate: 10.0,
                pressure_drop: 1000.0,
                reynolds_number: 0.1,
                velocity: 0.001,
            }));
            
            container.insert(black_box(ThermalMetadata {
                temperature: 25.0,
                heat_transfer_coefficient: 100.0,
                thermal_conductivity: 0.6,
            }));
            
            container.insert(black_box(ManufacturingMetadata {
                width_tolerance: 0.5,
                height_tolerance: 0.3,
                surface_roughness: 0.1,
                manufacturing_method: "Soft Lithography".to_string(),
            }));
            
            container.insert(black_box(OptimizationMetadata {
                original_length: 50.0,
                optimized_length: 60.0,
                improvement_percentage: 20.0,
                iterations: 25,
                optimization_time_ms: 150,
                optimization_profile: "Balanced".to_string(),
            }));
            
            container.insert(black_box(PerformanceMetadata {
                generation_time_us: 1000,
                memory_usage_bytes: 256,
                path_points_count: 10,
            }));
            
            black_box(container);
        })
    });
    
    // Benchmark retrieval performance
    let mut container = MetadataContainer::new();
    container.insert(FlowMetadata {
        flow_rate: 10.0,
        pressure_drop: 1000.0,
        reynolds_number: 0.1,
        velocity: 0.001,
    });
    container.insert(ThermalMetadata {
        temperature: 25.0,
        heat_transfer_coefficient: 100.0,
        thermal_conductivity: 0.6,
    });
    
    group.bench_function("retrieve_metadata", |b| {
        b.iter(|| {
            let flow_data = container.get::<FlowMetadata>();
            let thermal_data = container.get::<ThermalMetadata>();
            black_box((flow_data, thermal_data));
        })
    });
    
    // Benchmark cloning performance
    group.bench_function("clone_container", |b| {
        b.iter(|| {
            let cloned = container.clone();
            black_box(cloned);
        })
    });
    
    group.finish();
}

/// Benchmark channel builder with metadata
fn bench_channel_builder_with_metadata(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_builder_metadata");
    
    // Benchmark building channel without metadata
    group.bench_function("build_channel_no_metadata", |b| {
        b.iter(|| {
            let channel = ChannelBuilder::new(0, 0, 1, 1.0, 0.5, ChannelType::Straight)
                .build();
            black_box(channel);
        })
    });
    
    // Benchmark building channel with single metadata
    group.bench_function("build_channel_single_metadata", |b| {
        b.iter(|| {
            let flow_data = FlowMetadata {
                flow_rate: 10.0,
                pressure_drop: 1000.0,
                reynolds_number: 0.1,
                velocity: 0.001,
            };
            
            let channel = ChannelBuilder::new(0, 0, 1, 1.0, 0.5, ChannelType::Straight)
                .with_metadata(black_box(flow_data))
                .build();
            black_box(channel);
        })
    });
    
    // Benchmark building channel with multiple metadata
    group.bench_function("build_channel_multiple_metadata", |b| {
        b.iter(|| {
            let flow_data = FlowMetadata {
                flow_rate: 10.0,
                pressure_drop: 1000.0,
                reynolds_number: 0.1,
                velocity: 0.001,
            };
            
            let thermal_data = ThermalMetadata {
                temperature: 25.0,
                heat_transfer_coefficient: 100.0,
                thermal_conductivity: 0.6,
            };
            
            let manufacturing_data = ManufacturingMetadata {
                width_tolerance: 0.5,
                height_tolerance: 0.3,
                surface_roughness: 0.1,
                manufacturing_method: "Soft Lithography".to_string(),
            };
            
            let channel = ChannelBuilder::new(0, 0, 1, 1.0, 0.5, ChannelType::Straight)
                .with_metadata(black_box(flow_data))
                .with_metadata(black_box(thermal_data))
                .with_metadata(black_box(manufacturing_data))
                .build();
            black_box(channel);
        })
    });
    
    group.finish();
}

/// Benchmark extension trait operations
fn bench_extension_trait_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("extension_traits");
    
    // Create a channel with metadata for testing
    let mut channel = Channel {
        id: 0,
        from_node: 0,
        to_node: 1,
        width: 1.0,
        height: 0.5,
        channel_type: ChannelType::Straight,
        metadata: None,
    };
    
    // Add initial metadata
    channel.add_metadata(FlowMetadata {
        flow_rate: 10.0,
        pressure_drop: 1000.0,
        reynolds_number: 0.1,
        velocity: 0.001,
    });
    
    channel.add_metadata(ThermalMetadata {
        temperature: 25.0,
        heat_transfer_coefficient: 100.0,
        thermal_conductivity: 0.6,
    });
    
    // Benchmark metadata access
    group.bench_function("get_metadata", |b| {
        b.iter(|| {
            let flow_data = channel.get_metadata::<FlowMetadata>();
            let thermal_data = channel.get_metadata::<ThermalMetadata>();
            black_box((flow_data, thermal_data));
        })
    });
    
    // Benchmark metadata existence check
    group.bench_function("has_metadata", |b| {
        b.iter(|| {
            let has_flow = channel.has_metadata::<FlowMetadata>();
            let has_thermal = channel.has_metadata::<ThermalMetadata>();
            let has_manufacturing = channel.has_metadata::<ManufacturingMetadata>();
            black_box((has_flow, has_thermal, has_manufacturing));
        })
    });
    
    // Benchmark adding metadata to existing channel
    group.bench_function("add_metadata_to_existing", |b| {
        b.iter(|| {
            let mut test_channel = channel.clone();
            let perf_data = PerformanceMetadata {
                generation_time_us: 1000,
                memory_usage_bytes: 256,
                path_points_count: 10,
            };
            test_channel.add_metadata(black_box(perf_data));
            black_box(test_channel);
        })
    });
    
    group.finish();
}

/// Benchmark metadata system overhead in geometry generation
fn bench_metadata_system_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_system_overhead");
    
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Benchmark standard geometry generation (baseline)
    group.bench_function("standard_generation", |b| {
        b.iter(|| {
            let system = create_geometry(
                black_box(box_dims),
                black_box(&splits),
                black_box(&config),
                black_box(&ChannelTypeConfig::AllStraight),
            );
            black_box(system);
        })
    });
    
    // Benchmark geometry generation with metadata addition
    group.bench_function("generation_with_metadata_addition", |b| {
        b.iter(|| {
            let mut system = create_geometry(
                black_box(box_dims),
                black_box(&splits),
                black_box(&config),
                black_box(&ChannelTypeConfig::AllStraight),
            );
            
            // Add metadata to all channels
            for channel in &mut system.channels {
                channel.add_metadata(FlowMetadata {
                    flow_rate: 10.0,
                    pressure_drop: 1000.0,
                    reynolds_number: 0.1,
                    velocity: 0.001,
                });
                
                channel.add_metadata(ThermalMetadata {
                    temperature: 25.0,
                    heat_transfer_coefficient: 100.0,
                    thermal_conductivity: 0.6,
                });
            }
            
            // Add metadata to all nodes
            for node in &mut system.nodes {
                node.add_metadata(PerformanceMetadata {
                    generation_time_us: 1000,
                    memory_usage_bytes: 64,
                    path_points_count: 1,
                });
            }
            
            black_box(system);
        })
    });
    
    group.finish();
}

/// Benchmark memory usage with different metadata loads
fn bench_metadata_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_memory_usage");
    
    // Test different numbers of metadata entries
    let metadata_counts = vec![1, 5, 10, 20];
    
    for &count in &metadata_counts {
        group.bench_with_input(
            BenchmarkId::new("metadata_entries", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let mut container = MetadataContainer::new();
                    
                    for i in 0..count {
                        // Add different types of metadata to test memory usage
                        match i % 5 {
                            0 => container.insert(FlowMetadata {
                                flow_rate: i as f64,
                                pressure_drop: (i * 100) as f64,
                                reynolds_number: i as f64 * 0.01,
                                velocity: i as f64 * 0.001,
                            }),
                            1 => container.insert(ThermalMetadata {
                                temperature: 25.0 + i as f64,
                                heat_transfer_coefficient: 100.0 + i as f64,
                                thermal_conductivity: 0.6,
                            }),
                            2 => container.insert(ManufacturingMetadata {
                                width_tolerance: 0.5,
                                height_tolerance: 0.3,
                                surface_roughness: 0.1,
                                manufacturing_method: format!("Method_{}", i),
                            }),
                            3 => container.insert(OptimizationMetadata {
                                original_length: 50.0 + i as f64,
                                optimized_length: 60.0 + i as f64,
                                improvement_percentage: i as f64,
                                iterations: 25 + i,
                                optimization_time_ms: 150 + i as u64,
                                optimization_profile: format!("Profile_{}", i),
                            }),
                            4 => container.insert(PerformanceMetadata {
                                generation_time_us: 1000 + i as u64,
                                memory_usage_bytes: 256 + i,
                                path_points_count: 10 + i,
                            }),
                            _ => unreachable!(),
                        }
                    }
                    
                    black_box(container);
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    metadata_benches,
    bench_metadata_container_operations,
    bench_channel_builder_with_metadata,
    bench_extension_trait_operations,
    bench_metadata_system_overhead,
    bench_metadata_memory_usage
);

criterion_main!(metadata_benches);
