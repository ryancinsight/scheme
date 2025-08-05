//! geometry/types.rs - Core 2D Geometry Types
//!
//! This module defines the fundamental data structures used throughout
//! the 2D microfluidic schematic design system. It provides types for
//! representing points, nodes, channels, and complete channel systems.
//!
//! The system supports extensible metadata through the metadata module,
//! allowing for easy addition of new tracking variables without breaking
//! existing functionality.

use crate::geometry::metadata::MetadataContainer;
use serde::{Deserialize, Serialize};

/// A 2D point represented as (x, y) coordinates
pub type Point2D = (f64, f64);

/// A node represents a connection point in the channel system
///
/// Nodes are used to define the endpoints of channels and serve as
/// junction points where multiple channels can connect.
///
/// The node supports extensible metadata for tracking additional properties
/// like pressure, temperature, or manufacturing tolerances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for this node
    pub id: usize,
    /// 2D coordinates of the node
    pub point: Point2D,
    /// Optional metadata container for extensible properties
    #[serde(skip)]
    pub metadata: Option<MetadataContainer>,
}

/// Categories of channel types for visualization and analysis
///
/// This enum groups channel types into categories for consistent coloring
/// and styling in visualizations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelTypeCategory {
    /// Straight line channels (Straight, SmoothStraight)
    Straight,
    /// Curved channels (Serpentine, Arc)
    Curved,
    /// Tapered channels (Frustum)
    Tapered,
}

impl From<&ChannelType> for ChannelTypeCategory {
    fn from(channel_type: &ChannelType) -> Self {
        match channel_type {
            ChannelType::Straight | ChannelType::SmoothStraight { .. } => ChannelTypeCategory::Straight,
            ChannelType::Serpentine { .. } | ChannelType::Arc { .. } => ChannelTypeCategory::Curved,
            ChannelType::Frustum { .. } => ChannelTypeCategory::Tapered,
        }
    }
}

/// Represents the different types of channels that can be generated
///
/// Each channel type has different characteristics:
/// - `Straight`: Direct line between two points
/// - `SmoothStraight`: Straight line with optional smooth transition zones at endpoints
/// - `Serpentine`: Sinusoidal path with Gaussian envelope for smooth transitions
/// - `Arc`: Curved path using quadratic Bezier curves
/// - `Frustum`: Tapered channel with variable width for venturi throat functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    /// A straight line channel between two points
    Straight,
    /// A straight line channel with smooth transition zones at endpoints
    SmoothStraight {
        /// The sequence of points defining the smooth straight path with transitions
        path: Vec<Point2D>
    },
    /// A serpentine (S-shaped) channel with a predefined path
    Serpentine {
        /// The sequence of points defining the serpentine path
        path: Vec<Point2D>
    },
    /// An arc channel with a curved path
    Arc {
        /// The sequence of points defining the arc path
        path: Vec<Point2D>
    },
    /// A frustum (tapered) channel with variable width for venturi throat functionality
    Frustum {
        /// The sequence of points defining the centerline path
        path: Vec<Point2D>,
        /// Width values corresponding to each point in the path
        widths: Vec<f64>,
        /// Inlet width (starting width)
        inlet_width: f64,
        /// Throat width (minimum width at center)
        throat_width: f64,
        /// Outlet width (ending width)
        outlet_width: f64,
    },
}

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::Straight
    }
}

/// Represents a single channel in the microfluidic system
///
/// A channel connects two nodes and has physical properties like width and height.
/// The channel type determines how the path between the nodes is generated.
///
/// The channel supports extensible metadata for tracking additional properties
/// like flow rates, pressure drops, optimization history, or manufacturing data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Unique identifier for this channel
    pub id: usize,
    /// ID of the starting node
    pub from_node: usize,
    /// ID of the ending node
    pub to_node: usize,
    /// Physical width of the channel
    pub width: f64,
    /// Physical height of the channel
    pub height: f64,
    /// The type and path of this channel
    pub channel_type: ChannelType,
    /// Optional metadata container for extensible properties
    #[serde(skip)]
    pub metadata: Option<MetadataContainer>,
}

/// Represents a complete microfluidic channel system
///
/// This is the main data structure that contains all the geometric information
/// needed to represent a 2D microfluidic schematic, including nodes, channels,
/// and the containing boundary box.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSystem {
    /// Dimensions of the containing box (width, height)
    pub box_dims: (f64, f64),
    /// All nodes in the system
    pub nodes: Vec<Node>,
    /// All channels in the system
    pub channels: Vec<Channel>,
    /// Line segments defining the boundary box outline
    pub box_outline: Vec<(Point2D, Point2D)>,
}

impl ChannelSystem {
    /// Export the channel system to JSON format
    ///
    /// This method serializes the entire channel system to a JSON string,
    /// making it easy to save, load, or transfer channel system data.
    ///
    /// # Returns
    ///
    /// A JSON string representation of the channel system, or an error if
    /// serialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::generator::create_geometry;
    /// use scheme::geometry::SplitType;
    /// use scheme::config::{GeometryConfig, ChannelTypeConfig};
    ///
    /// let system = create_geometry(
    ///     (200.0, 100.0),
    ///     &[SplitType::Bifurcation],
    ///     &GeometryConfig::default(),
    ///     &ChannelTypeConfig::AllStraight,
    /// );
    ///
    /// let json = system.to_json().expect("Failed to serialize");
    /// println!("Exported system: {}", json);
    /// ```
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import a channel system from JSON format
    ///
    /// This method deserializes a channel system from a JSON string.
    ///
    /// # Arguments
    ///
    /// * `json` - A JSON string representation of a channel system
    ///
    /// # Returns
    ///
    /// A channel system instance, or an error if deserialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::ChannelSystem;
    ///
    /// let json = r#"{"box_dims": [200.0, 100.0], "nodes": [], "channels": [], "box_outline": []}"#;
    /// let system = ChannelSystem::from_json(json).expect("Failed to deserialize");
    /// ```
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get all line segments that make up this channel system
    ///
    /// This method extracts all the individual line segments from all channels
    /// in the system, which is useful for rendering and analysis.
    ///
    /// # Returns
    ///
    /// A vector of line segments, where each segment is represented as a tuple
    /// of two points (start, end).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::generator::create_geometry;
    /// use scheme::geometry::SplitType;
    /// use scheme::config::{GeometryConfig, ChannelTypeConfig};
    ///
    /// let system = create_geometry(
    ///     (200.0, 100.0),
    ///     &[SplitType::Bifurcation],
    ///     &GeometryConfig::default(),
    ///     &ChannelTypeConfig::AllStraight,
    /// );
    ///
    /// let lines = system.get_lines();
    /// println!("System has {} line segments", lines.len());
    /// ```
    pub fn get_lines(&self) -> Vec<(Point2D, Point2D)> {
        let mut lines = self.box_outline.clone();
        for channel in &self.channels {
            match &channel.channel_type {
                ChannelType::Straight => {
                    let from = self.nodes[channel.from_node].point;
                    let to = self.nodes[channel.to_node].point;
                    lines.push((from, to));
                }
                ChannelType::SmoothStraight { path } | ChannelType::Serpentine { path } | ChannelType::Arc { path } => {
                    for i in 0..path.len() - 1 {
                        lines.push((path[i], path[i + 1]));
                    }
                }
                ChannelType::Frustum { path, .. } => {
                    for i in 0..path.len() - 1 {
                        lines.push((path[i], path[i + 1]));
                    }
                }
            }
        }
        lines
    }

    /// Get all line segments with their associated channel types for colored rendering
    ///
    /// This method extracts line segments along with channel type information,
    /// enabling different colors for different channel types in visualization.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Boundary lines (for the box outline)
    /// - Channel lines grouped by channel type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::generator::create_geometry;
    /// use scheme::geometry::SplitType;
    /// use scheme::config::{GeometryConfig, ChannelTypeConfig};
    ///
    /// let system = create_geometry(
    ///     (200.0, 100.0),
    ///     &[SplitType::Bifurcation],
    ///     &GeometryConfig::default(),
    ///     &ChannelTypeConfig::AllStraight,
    /// );
    ///
    /// let (boundary_lines, channel_lines) = system.get_lines_by_type();
    /// println!("System has {} boundary lines", boundary_lines.len());
    /// for (channel_type, lines) in channel_lines {
    ///     println!("Channel type {:?} has {} line segments", channel_type, lines.len());
    /// }
    /// ```
    pub fn get_lines_by_type(&self) -> (Vec<(Point2D, Point2D)>, std::collections::HashMap<ChannelTypeCategory, Vec<(Point2D, Point2D)>>) {
        use std::collections::HashMap;

        let boundary_lines = self.box_outline.clone();
        let mut channel_lines: HashMap<ChannelTypeCategory, Vec<(Point2D, Point2D)>> = HashMap::new();

        for channel in &self.channels {
            let category = ChannelTypeCategory::from(&channel.channel_type);
            let lines = channel_lines.entry(category).or_insert_with(Vec::new);

            match &channel.channel_type {
                ChannelType::Straight => {
                    let from = self.nodes[channel.from_node].point;
                    let to = self.nodes[channel.to_node].point;
                    lines.push((from, to));
                }
                ChannelType::SmoothStraight { path } | ChannelType::Serpentine { path } | ChannelType::Arc { path } => {
                    for i in 0..path.len() - 1 {
                        lines.push((path[i], path[i + 1]));
                    }
                }
                ChannelType::Frustum { path, .. } => {
                    for i in 0..path.len() - 1 {
                        lines.push((path[i], path[i + 1]));
                    }
                }
            }
        }

        (boundary_lines, channel_lines)
    }

    /// Get the path segments for all channels in the system
    ///
    /// This method returns the complete path information for each channel,
    /// which is particularly useful for serpentine and arc channels that
    /// have complex paths with multiple points.
    ///
    /// # Returns
    ///
    /// A vector where each element is a vector of points representing
    /// the complete path of one channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::generator::create_geometry;
    /// use scheme::geometry::SplitType;
    /// use scheme::config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig};
    ///
    /// let system = create_geometry(
    ///     (200.0, 100.0),
    ///     &[SplitType::Bifurcation],
    ///     &GeometryConfig::default(),
    ///     &ChannelTypeConfig::AllSerpentine(SerpentineConfig::default()),
    /// );
    ///
    /// let paths = system.get_path_segments();
    /// for (i, path) in paths.iter().enumerate() {
    ///     println!("Channel {} has {} points in its path", i, path.len());
    /// }
    /// ```
    pub fn get_path_segments(&self) -> Vec<Vec<Point2D>> {
        self.channels
            .iter()
            .filter_map(|c| match &c.channel_type {
                ChannelType::SmoothStraight { path } | ChannelType::Serpentine { path } | ChannelType::Arc { path } => Some(path.clone()),
                ChannelType::Frustum { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect()
    }
}

/// Defines the type of channel splitting pattern
///
/// Split types determine how many branches are created at each junction:
/// - `Bifurcation`: Splits into 2 branches
/// - `Trifurcation`: Splits into 3 branches
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SplitType {
    /// Split into two branches
    Bifurcation,
    /// Split into three branches
    Trifurcation,
}

impl SplitType {
    /// Returns the number of branches created by this split type
    pub fn branch_count(&self) -> usize {
        match self {
            SplitType::Bifurcation => 2,
            SplitType::Trifurcation => 3,
        }
    }
}

// CFD functionality removed - Scheme focuses exclusively on 2D schematic design