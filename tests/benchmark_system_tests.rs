use scheme::{
    geometry::{
        generator::create_geometry,
        metadata::{MetadataContainer, FlowMetadata, ThermalMetadata},
        builders::{ChannelBuilder, ChannelExt},
        types::ChannelType,
        SplitType,
    },
    config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig, OptimizationProfile},
};
use std::time::Instant;

/// Test that optimization benchmarks can run without errors
#[test]
fn test_optimization_benchmark_functionality() {
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Test Fast optimization profile
    let fast_config = SerpentineConfig {
        optimization_enabled: true,
        optimization_profile: OptimizationProfile::Fast,
        target_fill_ratio: 0.9,
        ..SerpentineConfig::default()
    };
    
    let start_time = Instant::now();
    let system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllSerpentine(fast_config),
    );
    let duration = start_time.elapsed();
    
    // Verify system was created successfully
    assert!(system.channels.len() > 0);
    assert!(system.nodes.len() > 0);
    
    // Fast optimization should complete reasonably quickly
    assert!(duration.as_secs() < 30, "Fast optimization took too long: {:?}", duration);
}

/// Test that metadata benchmarks can run without errors
#[test]
fn test_metadata_benchmark_functionality() {
    // Test metadata container operations
    let mut container = MetadataContainer::new();
    
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
    
    // Test insertion performance
    let start_time = Instant::now();
    container.insert(flow_data.clone());
    container.insert(thermal_data.clone());
    let insertion_time = start_time.elapsed();
    
    // Insertion should be fast
    assert!(insertion_time.as_millis() < 10, "Metadata insertion too slow: {:?}", insertion_time);
    
    // Test retrieval performance
    let start_time = Instant::now();
    let retrieved_flow = container.get::<FlowMetadata>().unwrap();
    let retrieved_thermal = container.get::<ThermalMetadata>().unwrap();
    let retrieval_time = start_time.elapsed();
    
    // Retrieval should be fast
    assert!(retrieval_time.as_millis() < 5, "Metadata retrieval too slow: {:?}", retrieval_time);
    
    // Verify data integrity
    assert_eq!(retrieved_flow, &flow_data);
    assert_eq!(retrieved_thermal, &thermal_data);
}

/// Test channel builder with metadata performance
#[test]
fn test_channel_builder_metadata_performance() {
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
    
    // Test building channel with multiple metadata
    let start_time = Instant::now();
    let channel = ChannelBuilder::new(0, 0, 1, 1.0, 0.5, ChannelType::Straight)
        .with_metadata(flow_data.clone())
        .with_metadata(thermal_data.clone())
        .build();
    let build_time = start_time.elapsed();
    
    // Building should be fast
    assert!(build_time.as_millis() < 10, "Channel building too slow: {:?}", build_time);
    
    // Verify metadata was added correctly
    assert!(channel.has_metadata::<FlowMetadata>());
    assert!(channel.has_metadata::<ThermalMetadata>());
    
    let retrieved_flow = channel.get_metadata::<FlowMetadata>().unwrap();
    let retrieved_thermal = channel.get_metadata::<ThermalMetadata>().unwrap();
    
    assert_eq!(retrieved_flow, &flow_data);
    assert_eq!(retrieved_thermal, &thermal_data);
}

/// Test metadata system overhead in geometry generation
#[test]
fn test_metadata_system_overhead() {
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Benchmark standard generation
    let start_time = Instant::now();
    let system_standard = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    let standard_time = start_time.elapsed();
    
    // Benchmark generation with metadata addition
    let start_time = Instant::now();
    let mut system_with_metadata = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    // Add metadata to all channels
    for channel in &mut system_with_metadata.channels {
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
    let metadata_time = start_time.elapsed();
    
    // Both systems should have the same structure
    assert_eq!(system_standard.channels.len(), system_with_metadata.channels.len());
    assert_eq!(system_standard.nodes.len(), system_with_metadata.nodes.len());
    
    // Metadata overhead should be reasonable (less than 10x slower)
    let overhead_ratio = metadata_time.as_secs_f64() / standard_time.as_secs_f64();
    assert!(overhead_ratio < 10.0, "Metadata overhead too high: {:.2}x", overhead_ratio);
    
    println!("Standard generation: {:?}", standard_time);
    println!("With metadata: {:?}", metadata_time);
    println!("Overhead ratio: {:.2}x", overhead_ratio);
}

/// Test optimization performance scaling
#[test]
fn test_optimization_performance_scaling() {
    let config = GeometryConfig::default();
    
    // Test different complexities
    let test_cases = vec![
        ("simple", (200.0, 100.0), vec![SplitType::Bifurcation]),
        ("medium", (300.0, 150.0), vec![SplitType::Bifurcation, SplitType::Bifurcation]),
        ("complex", (400.0, 200.0), vec![SplitType::Bifurcation, SplitType::Trifurcation]),
    ];
    
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        optimization_profile: OptimizationProfile::Fast, // Use Fast for testing
        target_fill_ratio: 0.9,
        ..SerpentineConfig::default()
    };
    
    let mut previous_time = None;
    
    for (name, box_dims, splits) in test_cases {
        let start_time = Instant::now();
        let system = create_geometry(
            box_dims,
            &splits,
            &config,
            &ChannelTypeConfig::AllSerpentine(serpentine_config),
        );
        let duration = start_time.elapsed();
        
        // Verify system was created
        assert!(system.channels.len() > 0, "No channels created for {}", name);
        assert!(system.nodes.len() > 0, "No nodes created for {}", name);
        
        // Check that optimization time scales reasonably
        if let Some(prev_time) = previous_time {
            let scaling_factor = duration.as_secs_f64() / prev_time;
            assert!(scaling_factor < 10.0, "Optimization scaling too poor for {}: {:.2}x", name, scaling_factor);
        }
        
        previous_time = Some(duration.as_secs_f64());
        
        println!("{} optimization: {:?} ({} channels)", name, duration, system.channels.len());
    }
}

/// Test memory usage characteristics
#[test]
fn test_memory_usage_characteristics() {
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Create system and measure approximate memory usage
    let system = create_geometry(
        box_dims,
        &splits,
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    // Calculate approximate memory usage
    let channel_memory = system.channels.len() * std::mem::size_of::<scheme::geometry::types::Channel>();
    let node_memory = system.nodes.len() * std::mem::size_of::<scheme::geometry::types::Node>();
    let total_memory = channel_memory + node_memory;
    
    // Memory usage should be reasonable (less than 1MB for simple system)
    assert!(total_memory < 1_000_000, "Memory usage too high: {} bytes", total_memory);
    
    println!("Channels: {} ({} bytes)", system.channels.len(), channel_memory);
    println!("Nodes: {} ({} bytes)", system.nodes.len(), node_memory);
    println!("Total approximate memory: {} bytes", total_memory);
}

/// Test benchmark system reliability
#[test]
fn test_benchmark_system_reliability() {
    let config = GeometryConfig::default();
    let box_dims = (200.0, 100.0);
    let splits = vec![SplitType::Bifurcation];
    
    // Run the same operation multiple times to test consistency
    let mut times = Vec::new();
    
    for _ in 0..5 {
        let start_time = Instant::now();
        let system = create_geometry(
            box_dims,
            &splits,
            &config,
            &ChannelTypeConfig::AllStraight,
        );
        let duration = start_time.elapsed();
        
        // Verify consistent results
        assert!(system.channels.len() > 0);
        assert!(system.nodes.len() > 0);
        
        times.push(duration.as_micros());
    }
    
    // Calculate coefficient of variation
    let mean = times.iter().sum::<u128>() as f64 / times.len() as f64;
    let variance = times.iter()
        .map(|&x| (x as f64 - mean).powi(2))
        .sum::<f64>() / times.len() as f64;
    let std_dev = variance.sqrt();
    let cv = std_dev / mean;
    
    // Coefficient of variation should be reasonable (less than 200% for test environments)
    assert!(cv < 2.0, "Benchmark results too variable: CV = {:.2}", cv);
    
    println!("Benchmark times: {:?} μs", times);
    println!("Mean: {:.0} μs, Std Dev: {:.0} μs, CV: {:.2}", mean, std_dev, cv);
}

/// Test that all benchmark functions can be called without panicking
#[test]
fn test_benchmark_functions_callable() {
    // This test ensures that all benchmark functions can be called
    // without panicking, which validates the benchmark setup
    
    // Test metadata container operations
    let mut container = MetadataContainer::new();
    container.insert(FlowMetadata {
        flow_rate: 10.0,
        pressure_drop: 1000.0,
        reynolds_number: 0.1,
        velocity: 0.001,
    });
    
    let _retrieved = container.get::<FlowMetadata>();
    let _cloned = container.clone();
    
    // Test channel builder
    let _channel = ChannelBuilder::new(0, 0, 1, 1.0, 0.5, ChannelType::Straight)
        .with_metadata(FlowMetadata {
            flow_rate: 10.0,
            pressure_drop: 1000.0,
            reynolds_number: 0.1,
            velocity: 0.001,
        })
        .build();
    
    // Test optimization
    let config = GeometryConfig::default();
    let serpentine_config = SerpentineConfig {
        optimization_enabled: true,
        optimization_profile: OptimizationProfile::Fast,
        target_fill_ratio: 0.9,
        ..SerpentineConfig::default()
    };
    
    let _system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllSerpentine(serpentine_config),
    );
    
    // If we reach here, all benchmark functions are callable
    assert!(true);
}
