//! geometry/types.rs - Core 2D Geometry Types
//!
//! This module defines the fundamental data structures used throughout
//! the 2D microfluidic schematic design system. It provides types for
//! representing points, nodes, channels, and complete channel systems.

/// A 2D point represented as (x, y) coordinates
pub type Point2D = (f64, f64);

/// A node represents a connection point in the channel system
///
/// Nodes are used to define the endpoints of channels and serve as
/// junction points where multiple channels can connect.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node
    pub id: usize,
    /// 2D coordinates of the node
    pub point: Point2D,
}

/// Represents the different types of channels that can be generated
///
/// Each channel type has different characteristics:
/// - `Straight`: Direct line between two points
/// - `Serpentine`: Sinusoidal path with Gaussian envelope for smooth transitions
/// - `Arc`: Curved path using quadratic Bezier curves
#[derive(Debug, Clone)]
pub enum ChannelType {
    /// A straight line channel between two points
    Straight,
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
#[derive(Debug, Clone)]
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
}

/// Represents a complete microfluidic channel system
///
/// This is the main data structure that contains all the geometric information
/// needed to represent a 2D microfluidic schematic, including nodes, channels,
/// and the containing boundary box.
#[derive(Debug, Clone)]
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
    pub fn get_lines(&self) -> Vec<(Point2D, Point2D)> {
        let mut lines = self.box_outline.clone();
        for channel in &self.channels {
            match &channel.channel_type {
                ChannelType::Straight => {
                    let from = self.nodes[channel.from_node].point;
                    let to = self.nodes[channel.to_node].point;
                    lines.push((from, to));
                }
                ChannelType::Serpentine { path } | ChannelType::Arc { path } => {
                    for i in 0..path.len() - 1 {
                        lines.push((path[i], path[i + 1]));
                    }
                }
            }
        }
        lines
    }

    pub fn get_path_segments(&self) -> Vec<Vec<Point2D>> {
        self.channels
            .iter()
            .filter_map(|c| match &c.channel_type {
                ChannelType::Serpentine { path } | ChannelType::Arc { path } => Some(path.clone()),
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
#[derive(Clone, Copy, Debug)]
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