use scheme::{
    config::{ChannelTypeConfig, GeometryConfig},
    geometry::{
        generator::create_geometry,
        metadata::{
            MetadataContainer, FlowMetadata, ThermalMetadata, ManufacturingMetadata,
            OptimizationMetadata, PerformanceMetadata, Metadata,
        },
        builders::{ChannelBuilder, NodeBuilder, ChannelExt, NodeExt},
        types::{ChannelType, Node, Channel},
        SplitType,
    },
    impl_metadata,
};
use std::any::Any;

// Custom test metadata type
#[derive(Debug, Clone, PartialEq)]
struct TestMetadata {
    value: i32,
    name: String,
}

impl_metadata!(TestMetadata, "TestMetadata");

/// Test basic metadata container operations
#[test]
fn test_metadata_container_operations() {
    let mut container = MetadataContainer::new();
    
    // Test empty container
    assert!(container.is_empty());
    assert_eq!(container.len(), 0);
    
    // Test insertion
    let flow_data = FlowMetadata {
        flow_rate: 10.0,
        pressure_drop: 1000.0,
        reynolds_number: 0.1,
        velocity: 0.001,
    };
    
    container.insert(flow_data.clone());
    assert!(!container.is_empty());
    assert_eq!(container.len(), 1);
    assert!(container.contains::<FlowMetadata>());
    
    // Test retrieval
    let retrieved = container.get::<FlowMetadata>().unwrap();
    assert_eq!(retrieved, &flow_data);
    
    // Test multiple types
    let thermal_data = ThermalMetadata {
        temperature: 25.0,
        heat_transfer_coefficient: 100.0,
        thermal_conductivity: 0.6,
    };
    
    container.insert(thermal_data.clone());
    assert_eq!(container.len(), 2);
    
    // Test metadata types
    let types = container.metadata_types();
    assert_eq!(types.len(), 2);
    assert!(types.contains(&"FlowMetadata"));
    assert!(types.contains(&"ThermalMetadata"));
    
    // Test removal
    let removed = container.remove::<FlowMetadata>();
    assert!(removed.is_some());
    assert_eq!(container.len(), 1);
    assert!(!container.contains::<FlowMetadata>());
    assert!(container.contains::<ThermalMetadata>());
}

/// Test node builder with metadata
#[test]
fn test_node_builder_with_metadata() {
    let flow_data = FlowMetadata {
        flow_rate: 5.0,
        pressure_drop: 500.0,
        reynolds_number: 0.05,
        velocity: 0.0005,
    };
    
    let thermal_data = ThermalMetadata {
        temperature: 30.0,
        heat_transfer_coefficient: 120.0,
        thermal_conductivity: 0.7,
    };
    
    let node = NodeBuilder::new(0, (10.0, 20.0))
        .with_metadata(flow_data.clone())
        .with_metadata(thermal_data.clone())
        .build();
    
    assert_eq!(node.id, 0);
    assert_eq!(node.point, (10.0, 20.0));
    assert!(node.has_metadata::<FlowMetadata>());
    assert!(node.has_metadata::<ThermalMetadata>());
    
    let retrieved_flow = node.get_metadata::<FlowMetadata>().unwrap();
    let retrieved_thermal = node.get_metadata::<ThermalMetadata>().unwrap();
    
    assert_eq!(retrieved_flow, &flow_data);
    assert_eq!(retrieved_thermal, &thermal_data);
}

/// Test channel builder with metadata
#[test]
fn test_channel_builder_with_metadata() {
    let manufacturing_data = ManufacturingMetadata {
        width_tolerance: 0.5,
        height_tolerance: 0.3,
        surface_roughness: 0.1,
        manufacturing_method: "Soft Lithography".to_string(),
    };
    
    let opt_data = OptimizationMetadata {
        original_length: 50.0,
        optimized_length: 60.0,
        improvement_percentage: 20.0,
        iterations: 25,
        optimization_time_ms: 150,
        optimization_profile: "Balanced".to_string(),
    };
    
    let channel = ChannelBuilder::new(0, 0, 1, 1.0, 0.5, ChannelType::Straight)
        .with_metadata(manufacturing_data.clone())
        .with_metadata(opt_data.clone())
        .build();
    
    assert_eq!(channel.id, 0);
    assert_eq!(channel.from_node, 0);
    assert_eq!(channel.to_node, 1);
    assert!(channel.has_metadata::<ManufacturingMetadata>());
    assert!(channel.has_metadata::<OptimizationMetadata>());
    
    let retrieved_manufacturing = channel.get_metadata::<ManufacturingMetadata>().unwrap();
    let retrieved_opt = channel.get_metadata::<OptimizationMetadata>().unwrap();
    
    assert_eq!(retrieved_manufacturing, &manufacturing_data);
    assert_eq!(retrieved_opt, &opt_data);
}

/// Test extension traits for nodes
#[test]
fn test_node_extension_traits() {
    let mut node = Node {
        id: 0,
        point: (0.0, 0.0),
        metadata: None,
    };
    
    // Test adding metadata
    let test_data = TestMetadata {
        value: 42,
        name: "test".to_string(),
    };
    
    node.add_metadata(test_data.clone());
    assert!(node.has_metadata::<TestMetadata>());
    
    // Test getting metadata
    let retrieved = node.get_metadata::<TestMetadata>().unwrap();
    assert_eq!(retrieved, &test_data);
    
    // Test mutable access
    {
        let mutable_data = node.get_metadata_mut::<TestMetadata>().unwrap();
        mutable_data.value = 100;
        mutable_data.name = "modified".to_string();
    }
    
    let modified = node.get_metadata::<TestMetadata>().unwrap();
    assert_eq!(modified.value, 100);
    assert_eq!(modified.name, "modified");
    
    // Test metadata types
    let types = node.metadata_types();
    assert_eq!(types, vec!["TestMetadata"]);
    
    // Test removal
    assert!(node.remove_metadata::<TestMetadata>());
    assert!(!node.has_metadata::<TestMetadata>());
    assert!(!node.remove_metadata::<TestMetadata>()); // Should return false for non-existent
}

/// Test extension traits for channels
#[test]
fn test_channel_extension_traits() {
    let mut channel = Channel {
        id: 0,
        from_node: 0,
        to_node: 1,
        width: 1.0,
        height: 0.5,
        channel_type: ChannelType::Straight,
        metadata: None,
    };
    
    // Test adding multiple metadata types
    let flow_data = FlowMetadata {
        flow_rate: 15.0,
        pressure_drop: 1500.0,
        reynolds_number: 0.15,
        velocity: 0.0015,
    };
    
    let perf_data = PerformanceMetadata {
        generation_time_us: 1000,
        memory_usage_bytes: 256,
        path_points_count: 10,
    };
    
    channel.add_metadata(flow_data.clone());
    channel.add_metadata(perf_data.clone());
    
    assert!(channel.has_metadata::<FlowMetadata>());
    assert!(channel.has_metadata::<PerformanceMetadata>());
    
    // Test metadata types
    let types = channel.metadata_types();
    assert_eq!(types.len(), 2);
    assert!(types.contains(&"FlowMetadata"));
    assert!(types.contains(&"PerformanceMetadata"));
    
    // Test getting specific metadata
    let retrieved_flow = channel.get_metadata::<FlowMetadata>().unwrap();
    let retrieved_perf = channel.get_metadata::<PerformanceMetadata>().unwrap();
    
    assert_eq!(retrieved_flow, &flow_data);
    assert_eq!(retrieved_perf, &perf_data);
}

/// Test backward compatibility - existing code should work unchanged
#[test]
fn test_backward_compatibility() {
    let config = GeometryConfig::default();
    let system = create_geometry(
        (100.0, 50.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    // System should be created successfully
    assert!(system.channels.len() > 0);
    assert!(system.nodes.len() > 0);
    
    // Channels and nodes should have no metadata by default
    for channel in &system.channels {
        assert!(channel.metadata.is_none() || channel.metadata.as_ref().unwrap().is_empty());
    }
    
    for node in &system.nodes {
        assert!(node.metadata.is_none() || node.metadata.as_ref().unwrap().is_empty());
    }
    
    // All existing operations should work
    let lines = system.get_lines();
    assert!(lines.len() > 0);
    
    let _path_segments = system.get_path_segments();
    // Path segments might be empty for straight channels, which is fine
}

/// Test metadata with complex channel systems
#[test]
fn test_metadata_with_complex_systems() {
    let config = GeometryConfig::default();
    let mut system = create_geometry(
        (300.0, 150.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    // Add different metadata to each channel
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let flow_data = FlowMetadata {
            flow_rate: 10.0 + i as f64,
            pressure_drop: 1000.0 + i as f64 * 100.0,
            reynolds_number: 0.1 + i as f64 * 0.01,
            velocity: 0.001 + i as f64 * 0.0001,
        };
        
        channel.add_metadata(flow_data);
        
        // Add thermal data to every other channel
        if i % 2 == 0 {
            let thermal_data = ThermalMetadata {
                temperature: 25.0 + i as f64,
                heat_transfer_coefficient: 100.0,
                thermal_conductivity: 0.6,
            };
            channel.add_metadata(thermal_data);
        }
    }
    
    // Verify metadata was added correctly
    for (i, channel) in system.channels.iter().enumerate() {
        assert!(channel.has_metadata::<FlowMetadata>());
        
        let flow_data = channel.get_metadata::<FlowMetadata>().unwrap();
        assert_eq!(flow_data.flow_rate, 10.0 + i as f64);
        
        if i % 2 == 0 {
            assert!(channel.has_metadata::<ThermalMetadata>());
        } else {
            assert!(!channel.has_metadata::<ThermalMetadata>());
        }
    }
}

/// Test custom metadata types
#[test]
fn test_custom_metadata_types() {
    #[derive(Debug, Clone, PartialEq)]
    struct CustomData {
        id: usize,
        description: String,
        values: Vec<f64>,
    }
    
    impl_metadata!(CustomData, "CustomData");
    
    let mut container = MetadataContainer::new();
    
    let custom_data = CustomData {
        id: 123,
        description: "Test custom metadata".to_string(),
        values: vec![1.0, 2.0, 3.0],
    };
    
    container.insert(custom_data.clone());
    
    assert!(container.contains::<CustomData>());
    let retrieved = container.get::<CustomData>().unwrap();
    assert_eq!(retrieved, &custom_data);
    
    // Test with channel
    let mut channel = Channel {
        id: 0,
        from_node: 0,
        to_node: 1,
        width: 1.0,
        height: 0.5,
        channel_type: ChannelType::Straight,
        metadata: None,
    };
    
    channel.add_metadata(custom_data.clone());
    assert!(channel.has_metadata::<CustomData>());
    
    let channel_custom = channel.get_metadata::<CustomData>().unwrap();
    assert_eq!(channel_custom.values, vec![1.0, 2.0, 3.0]);
}

/// Test metadata performance impact
#[test]
fn test_metadata_performance_impact() {
    let config = GeometryConfig::default();
    
    // Create system without metadata
    let start_time = std::time::Instant::now();
    let system_no_metadata = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    let time_no_metadata = start_time.elapsed();
    
    // Create system and add metadata
    let start_time = std::time::Instant::now();
    let mut system_with_metadata = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    // Add metadata to all channels
    for channel in system_with_metadata.channels.iter_mut() {
        channel.add_metadata(FlowMetadata {
            flow_rate: 10.0,
            pressure_drop: 1000.0,
            reynolds_number: 0.1,
            velocity: 0.001,
        });
    }
    let time_with_metadata = start_time.elapsed();
    
    // Both systems should have the same structure
    assert_eq!(system_no_metadata.channels.len(), system_with_metadata.channels.len());
    assert_eq!(system_no_metadata.nodes.len(), system_with_metadata.nodes.len());
    
    // Performance impact should be minimal for basic operations
    // (This is more of a documentation test than a strict requirement)
    println!("Generation without metadata: {:?}", time_no_metadata);
    println!("Generation with metadata: {:?}", time_with_metadata);
    
    // The metadata system should not significantly impact basic geometry generation
    assert!(time_with_metadata.as_millis() < time_no_metadata.as_millis() + 100);
}
