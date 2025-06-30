//! geometry/mod_2d.rs

// Arc import removed - no longer needed for 2D-only functionality

pub type Point2D = (f64, f64);

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub point: Point2D,
}

#[derive(Debug, Clone)]
pub enum ChannelType {
    Straight,
    Serpentine { path: Vec<Point2D> },
    Arc { path: Vec<Point2D> },
}

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::Straight
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: usize,
    pub from_node: usize,
    pub to_node: usize,
    pub width: f64,
    pub height: f64,
    pub channel_type: ChannelType,
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