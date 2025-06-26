//! geometry/mod_2d.rs

// Arc import removed - no longer needed for 2D-only functionality

pub type Point2D = (f64, f64);

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub point: Point2D,
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
    pub nodes: Vec<Node>,
    pub channels: Vec<Channel>,
    pub box_outline: Vec<(Point2D, Point2D)>,
}

impl ChannelSystem {
    pub fn get_lines(&self) -> Vec<(Point2D, Point2D)> {
        self.channels
            .iter()
            .map(|c| (self.nodes[c.from_node].point, self.nodes[c.to_node].point))
            .collect()
    }
}

#[derive(Clone, Copy)]
pub enum SplitType {
    Bifurcation,
    Trifurcation,
}

impl SplitType {
    pub fn branch_count(&self) -> usize {
        match self {
            SplitType::Bifurcation => 2,
            SplitType::Trifurcation => 3,
        }
    }
}

// CFD functionality removed - Scheme focuses exclusively on 2D schematic design