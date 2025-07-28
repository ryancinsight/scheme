use scheme::{
    config::{ChannelTypeConfig, GeometryConfig},
    geometry::{
        generator::create_geometry,
        metadata::{Metadata, MetadataContainer},
        builders::ChannelExt,
        SplitType,
    },
    impl_metadata,
};
use std::any::Any;
use std::fs;

// Custom metadata type for biological applications
#[derive(Debug, Clone, PartialEq)]
pub struct BiologicalMetadata {
    pub cell_type: String,
    pub cell_concentration: f64, // cells/mL
    pub viability: f64, // percentage
    pub ph_level: f64,
    pub osmolarity: f64, // mOsm/kg
}

// Use the convenience macro to implement Metadata trait
impl_metadata!(BiologicalMetadata, "BiologicalMetadata");

// Custom metadata type for chemical analysis
#[derive(Debug, Clone, PartialEq)]
pub struct ChemicalMetadata {
    pub reagent_name: String,
    pub concentration: f64, // mol/L
    pub molecular_weight: f64, // g/mol
    pub reaction_rate: f64, // 1/s
    pub diffusion_coefficient: f64, // m²/s
}

impl_metadata!(ChemicalMetadata, "ChemicalMetadata");

// Custom metadata type for quality control
#[derive(Debug, Clone, PartialEq)]
pub struct QualityControlMetadata {
    pub inspection_date: String,
    pub inspector_id: String,
    pub defect_count: usize,
    pub quality_score: f64, // 0.0 to 1.0
    pub certification_level: String,
}

impl_metadata!(QualityControlMetadata, "QualityControlMetadata");

// Custom metadata type for simulation results
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationMetadata {
    pub simulation_software: String,
    pub mesh_elements: usize,
    pub convergence_iterations: usize,
    pub simulation_time_hours: f64,
    pub accuracy_level: String,
}

impl_metadata!(SimulationMetadata, "SimulationMetadata");

fn main() {
    fs::create_dir_all("outputs/metadata").unwrap();

    println!("Custom Metadata Types Demo");
    println!("==========================");
    println!();

    // Create a basic system
    let config = GeometryConfig::default();
    let mut system = create_geometry(
        (300.0, 150.0),
        &[SplitType::Bifurcation, SplitType::Trifurcation],
        &config,
        &ChannelTypeConfig::AllStraight,
    );
    
    println!("Created system with {} channels", system.channels.len());
    println!();

    // 1. Add biological metadata to channels
    println!("1. Adding Biological Metadata");
    let cell_types = ["HeLa", "CHO", "E. coli", "Yeast", "T-cells", "Neurons"];
    
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let bio_data = BiologicalMetadata {
            cell_type: cell_types[i % cell_types.len()].to_string(),
            cell_concentration: 1e6 + i as f64 * 5e5, // cells/mL
            viability: 95.0 - i as f64 * 2.0, // percentage
            ph_level: 7.4 + i as f64 * 0.1,
            osmolarity: 300.0 + i as f64 * 10.0, // mOsm/kg
        };
        
        channel.add_metadata(bio_data);
        
        let bio = channel.get_metadata::<BiologicalMetadata>().unwrap();
        println!("   Channel {}: {} cells at {:.0} cells/mL, {:.1}% viability", 
            i, bio.cell_type, bio.cell_concentration, bio.viability);
    }
    println!();

    // 2. Add chemical metadata
    println!("2. Adding Chemical Metadata");
    let reagents = ["Glucose", "ATP", "NADH", "Tris-HCl", "EDTA", "BSA"];
    
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let chem_data = ChemicalMetadata {
            reagent_name: reagents[i % reagents.len()].to_string(),
            concentration: 0.1 + i as f64 * 0.05, // mol/L
            molecular_weight: 180.0 + i as f64 * 50.0, // g/mol
            reaction_rate: 0.01 + i as f64 * 0.005, // 1/s
            diffusion_coefficient: 1e-9 + i as f64 * 1e-10, // m²/s
        };
        
        channel.add_metadata(chem_data);
        
        let chem = channel.get_metadata::<ChemicalMetadata>().unwrap();
        println!("   Channel {}: {} at {:.2} mol/L, MW = {:.0} g/mol", 
            i, chem.reagent_name, chem.concentration, chem.molecular_weight);
    }
    println!();

    // 3. Add quality control metadata
    println!("3. Adding Quality Control Metadata");
    let inspectors = ["QC001", "QC002", "QC003"];
    let cert_levels = ["ISO9001", "FDA", "CE", "GMP"];
    
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let qc_data = QualityControlMetadata {
            inspection_date: format!("2024-01-{:02}", 15 + i),
            inspector_id: inspectors[i % inspectors.len()].to_string(),
            defect_count: i % 3, // 0-2 defects
            quality_score: 0.95 - i as f64 * 0.02,
            certification_level: cert_levels[i % cert_levels.len()].to_string(),
        };
        
        channel.add_metadata(qc_data);

        match channel.get_metadata::<QualityControlMetadata>() {
            Some(qc) => {
                println!("   Channel {}: Inspected by {} on {}, Score: {:.2}, Cert: {}",
                    i, qc.inspector_id, qc.inspection_date, qc.quality_score, qc.certification_level);
            }
            None => {
                println!("   Channel {}: Warning - QC metadata could not be retrieved", i);
            }
        }
    }
    println!();

    // 4. Add simulation metadata
    println!("4. Adding Simulation Metadata");
    let software_tools = ["COMSOL", "ANSYS Fluent", "OpenFOAM", "STAR-CCM+"];
    
    for (i, channel) in system.channels.iter_mut().enumerate() {
        let sim_data = SimulationMetadata {
            simulation_software: software_tools[i % software_tools.len()].to_string(),
            mesh_elements: 10000 + i * 5000,
            convergence_iterations: 100 + i * 20,
            simulation_time_hours: 2.0 + i as f64 * 0.5,
            accuracy_level: if i % 2 == 0 { "High".to_string() } else { "Medium".to_string() },
        };
        
        channel.add_metadata(sim_data);
        
        let sim = channel.get_metadata::<SimulationMetadata>().unwrap();
        println!("   Channel {}: {} simulation, {} elements, {:.1}h runtime", 
            i, sim.simulation_software, sim.mesh_elements, sim.simulation_time_hours);
    }
    println!();

    // 5. Complex metadata queries and analysis
    println!("5. Complex Metadata Analysis");
    
    // Find channels with high cell viability
    let high_viability_channels: Vec<_> = system.channels.iter()
        .enumerate()
        .filter(|(_, channel)| {
            channel.get_metadata::<BiologicalMetadata>()
                .map_or(false, |bio| bio.viability > 90.0)
        })
        .collect();
    
    println!("   Channels with >90% cell viability: {} out of {}", 
        high_viability_channels.len(), system.channels.len());
    
    // Find channels with quality issues
    let quality_issues: Vec<_> = system.channels.iter()
        .enumerate()
        .filter(|(_, channel)| {
            channel.get_metadata::<QualityControlMetadata>()
                .map_or(false, |qc| qc.defect_count > 0 || qc.quality_score < 0.9)
        })
        .collect();
    
    println!("   Channels with quality issues: {} out of {}", 
        quality_issues.len(), system.channels.len());
    
    // Calculate average simulation time
    let avg_sim_time: f64 = system.channels.iter()
        .filter_map(|channel| channel.get_metadata::<SimulationMetadata>())
        .map(|sim| sim.simulation_time_hours)
        .sum::<f64>() / system.channels.len() as f64;
    
    println!("   Average simulation time: {:.1} hours", avg_sim_time);
    println!();

    // 6. Demonstrate metadata type checking
    println!("6. Metadata Type Information");
    for (i, channel) in system.channels.iter().enumerate() {
        let metadata_types = channel.metadata_types();
        println!("   Channel {}: {} metadata types: {:?}", 
            i, metadata_types.len(), metadata_types);
        
        // Check for specific metadata types
        println!("     Has BiologicalMetadata: {}", channel.has_metadata::<BiologicalMetadata>());
        println!("     Has ChemicalMetadata: {}", channel.has_metadata::<ChemicalMetadata>());
        
        if i == 0 { // Only show details for first channel to avoid clutter
            if let Some(bio) = channel.get_metadata::<BiologicalMetadata>() {
                println!("     Biological details: {:?}", bio);
            }
        }
        println!();
    }

    // 7. Demonstrate metadata container operations
    println!("7. Advanced Metadata Container Operations");
    if let Some(first_channel) = system.channels.get_mut(0) {
        println!("   Original metadata count: {}", first_channel.metadata_types().len());
        
        // Create a new metadata container with selected metadata
        let mut new_container = MetadataContainer::new();
        
        // Copy only biological and chemical metadata
        if let Some(bio) = first_channel.get_metadata::<BiologicalMetadata>() {
            new_container.insert(bio.clone());
        }
        if let Some(chem) = first_channel.get_metadata::<ChemicalMetadata>() {
            new_container.insert(chem.clone());
        }
        
        println!("   New container metadata count: {}", new_container.len());
        println!("   New container types: {:?}", new_container.metadata_types());
        
        // Remove quality control metadata
        let removed = first_channel.remove_metadata::<QualityControlMetadata>();
        println!("   Removed QualityControlMetadata: {}", removed);
        println!("   Remaining metadata count: {}", first_channel.metadata_types().len());
    }
    println!();

    println!("Custom Metadata Demo Complete!");
    println!("==============================");
    println!("This demo showed how to:");
    println!("• Create custom metadata types for specific domains");
    println!("• Use the impl_metadata! macro for easy implementation");
    println!("• Add multiple metadata types to channels");
    println!("• Perform complex queries and analysis");
    println!("• Manage metadata containers efficiently");
    println!("• Maintain type safety throughout the process");
    println!();
    println!("The extensible metadata system supports any custom data type");
    println!("that implements the Metadata trait, making it easy to extend");
    println!("the system for new use cases and domains.");
}
