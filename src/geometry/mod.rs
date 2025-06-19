//! geometry/mod.rs

pub mod generator;
pub use generator::create_geometry;

pub type Point = (f64, f64);

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub point: Point,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: usize,
    pub from_node: usize,
    pub to_node: usize,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct ChannelSystem {
    pub box_dims: (f64, f64),
    pub lines: Vec<(Point, Point)>,
    pub nodes: Vec<Node>,
    pub channels: Vec<Channel>,
}

#[derive(Clone, Copy)]
pub enum SplitType {
    Bifurcation,
    Trifurcation,
}

/// Holds the results of a CFD simulation, linking them to the geometry.
pub struct CfdResults {
    pub system: ChannelSystem,
    pub node_pressures: std::collections::HashMap<usize, f64>,
    pub channel_flow_rates: std::collections::HashMap<usize, f64>,
    pub channel_resistances: std::collections::HashMap<usize, f64>,
} 