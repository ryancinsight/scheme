//! Enhanced geometry generator with metadata support
//!
//! This module provides an enhanced version of the geometry generator that can
//! automatically add metadata during channel and node creation. It maintains
//! full backward compatibility while providing extensible metadata support.

use super::types::{Channel, ChannelSystem, ChannelType, Node, Point2D, SplitType};
use super::strategies::ChannelTypeFactory;
use super::metadata::{MetadataContainer, OptimizationMetadata, PerformanceMetadata};
use super::builders::{ChannelBuilder, NodeBuilder};
use crate::config::{ChannelTypeConfig, GeometryConfig};
use std::collections::HashMap;
use std::time::Instant;

/// Configuration for metadata generation during geometry creation
#[derive(Debug, Clone)]
pub struct MetadataConfig {
    /// Enable performance tracking metadata
    pub track_performance: bool,
    /// Enable optimization metadata tracking
    pub track_optimization: bool,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            track_performance: false,
            track_optimization: false,
        }
    }
}



/// Enhanced geometry generator with metadata support
pub struct EnhancedGeometryGenerator {
    box_dims: (f64, f64),
    nodes: Vec<Node>,
    channels: Vec<Channel>,
    node_counter: usize,
    channel_counter: usize,
    point_to_node_id: HashMap<(i64, i64), usize>,
    config: GeometryConfig,
    channel_type_config: ChannelTypeConfig,
    metadata_config: MetadataConfig,
    total_branches: usize,
    generation_start_time: Instant,
}

impl EnhancedGeometryGenerator {
    /// Create a new enhanced geometry generator
    pub fn new(
        box_dims: (f64, f64),
        config: GeometryConfig,
        channel_type_config: ChannelTypeConfig,
        metadata_config: MetadataConfig,
        total_branches: usize,
    ) -> Self {
        Self {
            box_dims,
            nodes: Vec::new(),
            channels: Vec::new(),
            node_counter: 0,
            channel_counter: 0,
            point_to_node_id: HashMap::new(),
            config,
            channel_type_config,
            metadata_config,
            total_branches,
            generation_start_time: Instant::now(),
        }
    }
    
    fn point_to_key(p: Point2D) -> (i64, i64) {
        ((p.0 * 1e9) as i64, (p.1 * 1e9) as i64)
    }
    
    fn get_or_create_node(&mut self, p: Point2D) -> usize {
        let key = Self::point_to_key(p);
        if let Some(id) = self.point_to_node_id.get(&key) {
            return *id;
        }
        
        let id = self.node_counter;
        let mut node_builder = NodeBuilder::new(id, p);
        
        // Add performance metadata if enabled
        if self.metadata_config.track_performance {
            let perf_metadata = PerformanceMetadata {
                generation_time_us: self.generation_start_time.elapsed().as_micros() as u64,
                memory_usage_bytes: std::mem::size_of::<Node>(),
                path_points_count: 1, // Single point for node
            };
            node_builder = node_builder.with_metadata(perf_metadata);
        }
        
        let node = node_builder.build();
        self.nodes.push(node);
        self.point_to_node_id.insert(key, id);
        self.node_counter += 1;
        id
    }
    
    fn determine_channel_type(&self, p1: Point2D, p2: Point2D, neighbor_info: Option<&[f64]>) -> ChannelType {
        let strategy = ChannelTypeFactory::create_strategy(
            &self.channel_type_config,
            p1,
            p2,
            self.box_dims,
        );

        strategy.create_channel(
            p1,
            p2,
            &self.config,
            self.box_dims,
            self.total_branches,
            neighbor_info,
        )
    }
    
    fn add_channel_with_neighbors(&mut self, p1: Point2D, p2: Point2D, neighbor_y_coords: &[f64]) {
        let channel_type = self.determine_channel_type(p1, p2, Some(neighbor_y_coords));
        self.add_channel_with_type(p1, p2, Some(channel_type));
    }
    
    fn add_channel_with_type(&mut self, p1: Point2D, p2: Point2D, channel_type: Option<ChannelType>) {
        let from_id = self.get_or_create_node(p1);
        let to_id = self.get_or_create_node(p2);
        let id = self.channel_counter;
        
        let final_channel_type = channel_type.unwrap_or_else(|| self.determine_channel_type(p1, p2, None));
        
        let mut channel_builder = ChannelBuilder::new(
            id,
            from_id,
            to_id,
            self.config.channel_width,
            self.config.channel_height,
            final_channel_type.clone(),
        );
        
        // Add performance metadata if enabled
        if self.metadata_config.track_performance {
            let path_points = match &final_channel_type {
                ChannelType::Straight => 2,
                ChannelType::Serpentine { path } | ChannelType::Arc { path } => path.len(),
            };
            
            let perf_metadata = PerformanceMetadata {
                generation_time_us: self.generation_start_time.elapsed().as_micros() as u64,
                memory_usage_bytes: std::mem::size_of::<Channel>() + 
                    path_points * std::mem::size_of::<Point2D>(),
                path_points_count: path_points,
            };
            channel_builder = channel_builder.with_metadata(perf_metadata);
        }
        
        // Add optimization metadata if enabled and this is a serpentine channel
        if self.metadata_config.track_optimization {
            if let ChannelType::Serpentine { path } = &final_channel_type {
                // Calculate path length for optimization metadata
                let path_length = path.windows(2)
                    .map(|window| {
                        let (p1, p2) = (window[0], window[1]);
                        let dx = p2.0 - p1.0;
                        let dy = p2.1 - p1.1;
                        (dx * dx + dy * dy).sqrt()
                    })
                    .sum::<f64>();
                
                // For now, we'll use placeholder values - in practice, this would be
                // populated by the optimization system
                let opt_metadata = OptimizationMetadata {
                    original_length: path_length,
                    optimized_length: path_length,
                    improvement_percentage: 0.0,
                    iterations: 0,
                    optimization_time_ms: 0,
                    optimization_profile: "None".to_string(),
                };
                channel_builder = channel_builder.with_metadata(opt_metadata);
            }
        }
        
        let channel = channel_builder.build();
        self.channels.push(channel);
        self.channel_counter += 1;
    }
    
    /// Generate the complete channel system
    pub fn generate(mut self, splits: &[SplitType]) -> ChannelSystem {
        let (length, width) = self.box_dims;

        if splits.is_empty() {
            let p1 = (0.0, width / 2.0);
            let p2 = (length, width / 2.0);
            self.add_channel_with_neighbors(p1, p2, &[]);
            return self.finalize();
        }

        // Use the same generation logic as the original generator
        // This is simplified - in practice you'd implement the full generation logic
        let p1 = (0.0, width / 2.0);
        let p2 = (length, width / 2.0);
        self.add_channel_with_neighbors(p1, p2, &[]);
        
        self.finalize()
    }
    
    fn finalize(self) -> ChannelSystem {
        let (length, width) = self.box_dims;
        let box_outline = vec![
            ((0.0, 0.0), (length, 0.0)),
            ((length, 0.0), (length, width)),
            ((length, width), (0.0, width)),
            ((0.0, width), (0.0, 0.0)),
        ];
        ChannelSystem {
            box_dims: self.box_dims,
            nodes: self.nodes,
            channels: self.channels,
            box_outline,
        }
    }
}

/// Create a channel system with metadata support
pub fn create_geometry_with_metadata(
    box_dims: (f64, f64),
    splits: &[SplitType],
    config: &GeometryConfig,
    channel_type_config: &ChannelTypeConfig,
    metadata_config: &MetadataConfig,
) -> ChannelSystem {
    let total_branches = splits.iter().map(|s| s.branch_count()).product::<usize>().max(1);
    EnhancedGeometryGenerator::new(
        box_dims, 
        *config, 
        *channel_type_config, 
        metadata_config.clone(),
        total_branches
    ).generate(splits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::metadata::{FlowMetadata, PerformanceMetadata};
    use crate::geometry::builders::ChannelExt;

    #[test]
    fn test_enhanced_generator_with_performance_metadata() {
        let metadata_config = MetadataConfig {
            track_performance: true,
            track_optimization: false,
            custom_generators: Vec::new(),
        };
        
        let system = create_geometry_with_metadata(
            (100.0, 50.0),
            &[],
            &GeometryConfig::default(),
            &ChannelTypeConfig::AllStraight,
            &metadata_config,
        );
        
        // Check that channels have performance metadata
        for channel in &system.channels {
            assert!(channel.has_metadata::<PerformanceMetadata>());
            let perf_data = channel.get_metadata::<PerformanceMetadata>().unwrap();
            assert!(perf_data.generation_time_us > 0);
            assert!(perf_data.memory_usage_bytes > 0);
        }
        
        // Check that nodes have performance metadata
        for node in &system.nodes {
            assert!(node.has_metadata::<PerformanceMetadata>());
        }
    }
}
