use scheme::{
    config::{ChannelTypeConfig, GeometryConfig},
    geometry::{
        generator::create_geometry,
        metadata::{FlowMetadata, ThermalMetadata, ManufacturingMetadata, OptimizationMetadata},
        builders::{ChannelExt, NodeExt},
        SplitType,
    },
    visualizations::schematic::plot_geometry,
};
use std::fs;

fn main() {
    fs::create_dir_all("outputs/metadata").unwrap();

    println!("Extensible Metadata System Demo");
    println!("===============================");
    println!();

    // 1. Create a basic system without metadata
    println!("1. Creating Basic System");
    let config = GeometryConfig::default();
    let mut system = create_geometry(
        (200.0, 100.0),
        &[SplitType::Bifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    println!("   Created system with {} channels and {} nodes", 
        system.channels.len(), system.nodes.len());
    println!();

    // 2. Add flow metadata to channels
    println!("2. Adding Flow Metadata to Channels");
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let flow_data = FlowMetadata {
            flow_rate: 10.0 + i as f64 * 5.0, // Different flow rates
            pressure_drop: 1000.0 + i as f64 * 200.0,
            reynolds_number: 0.1 + i as f64 * 0.05,
            velocity: 0.001 + i as f64 * 0.0002,
        };
        
        channel.add_metadata(flow_data);
        println!("   Channel {}: Flow rate = {:.1} μL/min, Pressure drop = {:.0} Pa", 
            i, channel.get_metadata::<FlowMetadata>().unwrap().flow_rate,
            channel.get_metadata::<FlowMetadata>().unwrap().pressure_drop);
    }
    println!();

    // 3. Add thermal metadata to channels
    println!("3. Adding Thermal Metadata to Channels");
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let thermal_data = ThermalMetadata {
            temperature: 25.0 + i as f64 * 2.0,
            heat_transfer_coefficient: 100.0 + i as f64 * 10.0,
            thermal_conductivity: 0.6,
        };
        
        channel.add_metadata(thermal_data);
        println!("   Channel {}: Temperature = {:.1}°C, HTC = {:.0} W/(m²·K)", 
            i, channel.get_metadata::<ThermalMetadata>().unwrap().temperature,
            channel.get_metadata::<ThermalMetadata>().unwrap().heat_transfer_coefficient);
    }
    println!();

    // 4. Add manufacturing metadata
    println!("4. Adding Manufacturing Metadata");
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let manufacturing_data = ManufacturingMetadata {
            width_tolerance: 0.5,
            height_tolerance: 0.3,
            surface_roughness: 0.1,
            manufacturing_method: if i % 2 == 0 { 
                "Soft Lithography".to_string() 
            } else { 
                "3D Printing".to_string() 
            },
        };
        
        channel.add_metadata(manufacturing_data);
        println!("   Channel {}: Method = {}, Width tolerance = ±{:.1} μm", 
            i, channel.get_metadata::<ManufacturingMetadata>().unwrap().manufacturing_method,
            channel.get_metadata::<ManufacturingMetadata>().unwrap().width_tolerance);
    }
    println!();

    // 5. Add optimization metadata (simulated)
    println!("5. Adding Optimization Metadata");
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let opt_data = OptimizationMetadata {
            original_length: 50.0 + i as f64 * 10.0,
            optimized_length: 55.0 + i as f64 * 12.0,
            improvement_percentage: 10.0 + i as f64 * 2.0,
            iterations: 25 + i * 5,
            optimization_time_ms: 100 + i as u64 * 20,
            optimization_profile: "Balanced".to_string(),
        };
        
        channel.add_metadata(opt_data);
        println!("   Channel {}: Length improvement = {:.1}%, Iterations = {}", 
            i, channel.get_metadata::<OptimizationMetadata>().unwrap().improvement_percentage,
            channel.get_metadata::<OptimizationMetadata>().unwrap().iterations);
    }
    println!();

    // 6. Query and analyze metadata
    println!("6. Metadata Analysis");
    let mut total_flow_rate = 0.0;
    let mut avg_temperature = 0.0;
    let mut total_improvement = 0.0;
    
    for channel in &system.channels {
        if let Some(flow_data) = channel.get_metadata::<FlowMetadata>() {
            total_flow_rate += flow_data.flow_rate;
        }
        
        if let Some(thermal_data) = channel.get_metadata::<ThermalMetadata>() {
            avg_temperature += thermal_data.temperature;
        }
        
        if let Some(opt_data) = channel.get_metadata::<OptimizationMetadata>() {
            total_improvement += opt_data.improvement_percentage;
        }
    }
    
    let channel_count = system.channels.len() as f64;
    avg_temperature /= channel_count;
    let avg_improvement = total_improvement / channel_count;
    
    println!("   Total system flow rate: {:.1} μL/min", total_flow_rate);
    println!("   Average temperature: {:.1}°C", avg_temperature);
    println!("   Average length improvement: {:.1}%", avg_improvement);
    println!();

    // 7. Show metadata types for each channel
    println!("7. Metadata Types Summary");
    for (i, channel) in system.channels.iter().enumerate() {
        let metadata_types = channel.metadata_types();
        println!("   Channel {}: {} metadata types: {:?}", 
            i, metadata_types.len(), metadata_types);
    }
    println!();

    // 8. Demonstrate metadata removal
    println!("8. Metadata Removal Demo");
    if let Some(first_channel) = system.channels.get_mut(0) {
        println!("   Before removal: {} metadata types", first_channel.metadata_types().len());
        
        let removed = first_channel.remove_metadata::<FlowMetadata>();
        println!("   Removed FlowMetadata: {}", removed);
        
        println!("   After removal: {} metadata types: {:?}", 
            first_channel.metadata_types().len(), first_channel.metadata_types());
    }
    println!();

    // 9. Generate visualization
    plot_geometry(&system, "outputs/metadata/system_with_metadata.png").unwrap();
    println!("9. Generated visualization: outputs/metadata/system_with_metadata.png");
    println!();

    // 10. Performance comparison
    println!("10. Performance Impact Analysis");
    
    // Create system without metadata
    let start_time = std::time::Instant::now();
    let _system_no_metadata = create_geometry(
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
        channel.add_metadata(ThermalMetadata {
            temperature: 25.0,
            heat_transfer_coefficient: 100.0,
            thermal_conductivity: 0.6,
        });
    }
    let time_with_metadata = start_time.elapsed();
    
    println!("   Generation without metadata: {:?}", time_no_metadata);
    println!("   Generation with metadata: {:?}", time_with_metadata);
    println!("   Metadata overhead: {:.1}x", 
        time_with_metadata.as_secs_f64() / time_no_metadata.as_secs_f64());
    println!();

    println!("Demo complete! The extensible metadata system allows you to:");
    println!("• Add any type of tracking data to channels and nodes");
    println!("• Query metadata in a type-safe manner");
    println!("• Remove metadata when no longer needed");
    println!("• Maintain backward compatibility with existing code");
    println!("• Extend the system with custom metadata types");
}
