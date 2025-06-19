use super::{Channel, ChannelSystem, Node, Point, SplitType};
use crate::config::GeometryConfig;
use std::collections::HashMap;

struct GeometryGenerator {
    box_dims: (f64, f64),
    nodes: Vec<Node>,
    channels: Vec<Channel>,
    node_counter: usize,
    channel_counter: usize,
    point_to_node_id: HashMap<(i64, i64), usize>,
    config: GeometryConfig,
}

impl GeometryGenerator {
    fn new(box_dims: (f64, f64), config: GeometryConfig) -> Self {
        Self {
            box_dims,
            nodes: Vec::new(),
            channels: Vec::new(),
            node_counter: 0,
            channel_counter: 0,
            point_to_node_id: HashMap::new(),
            config,
        }
    }

    fn point_to_key(p: Point) -> (i64, i64) {
        ((p.0 * 1e9) as i64, (p.1 * 1e9) as i64)
    }

    fn get_or_create_node(&mut self, p: Point) -> usize {
        let key = Self::point_to_key(p);
        if let Some(id) = self.point_to_node_id.get(&key) {
            return *id;
        }
        let id = self.node_counter;
        self.nodes.push(Node { id, point: p });
        self.point_to_node_id.insert(key, id);
        self.node_counter += 1;
        id
    }

    fn add_channel(&mut self, p1: Point, p2: Point) {
        let from_id = self.get_or_create_node(p1);
        let to_id = self.get_or_create_node(p2);
        let id = self.channel_counter;
        self.channels.push(Channel {
            id,
            from_node: from_id,
            to_node: to_id,
            width: self.config.channel_width,
            height: self.config.channel_height,
        });
        self.channel_counter += 1;
    }

    fn generate(mut self, splits: &[SplitType]) -> ChannelSystem {
        let (length, width) = self.box_dims;

        if splits.is_empty() {
            let p1 = (0.0, width / 2.0);
            let p2 = (length, width / 2.0);
            self.add_channel(p1, p2);
            return self.finalize();
        }
        
        let first_half_lines = self.generate_first_half(splits);

        for (p1, p2) in &first_half_lines {
            self.add_channel(*p1, *p2);
        }

        self.mirror_geometry(&first_half_lines);
        
        self.finalize()
    }

    fn generate_first_half(&self, splits: &[SplitType]) -> Vec<(Point, Point)> {
        let (length, width) = self.box_dims;
        let effective_width = width - (2.0 * self.config.wall_clearance);
        let half_l = length / 2.0;
        let num_splits = splits.len() as u32;
        let num_segments_per_half = num_splits as f64 * 2.0 + 1.0;
        let dx = half_l / num_segments_per_half;

        let mut y_coords: Vec<f64> = vec![width / 2.0];
        let mut y_ranges: Vec<f64> = vec![effective_width];
        let mut current_x = 0.0;
        let mut lines = Vec::new();

        for split_type in splits {
            for y in &y_coords {
                lines.push(((current_x, *y), (current_x + dx, *y)));
            }
            current_x += dx;

            let (next_y_coords, next_y_ranges, new_lines) = 
                self.apply_split(split_type, &y_coords, &y_ranges, current_x, dx);
            
            y_coords = next_y_coords;
            y_ranges = next_y_ranges;
            lines.extend(new_lines);

            current_x += dx;
        }
        
        for y in &y_coords {
            lines.push(((current_x, *y), (half_l, *y)));
        }

        lines
    }

    fn apply_split(
        &self,
        split_type: &SplitType,
        y_coords: &[f64],
        y_ranges: &[f64],
        current_x: f64,
        dx: f64,
    ) -> (Vec<f64>, Vec<f64>, Vec<(Point, Point)>) {
        let mut next_y_coords = Vec::new();
        let mut next_y_ranges = Vec::new();
        let mut new_lines = Vec::new();

        for (j, y_center) in y_coords.iter().enumerate() {
            let y_range = y_ranges[j];
            let n_branches = split_type.branch_count();
            let y_separation = y_range / n_branches as f64;

            for i in 0..n_branches {
                let offset = (i as f64 - (n_branches - 1) as f64 / 2.0) * y_separation;
                let y_new = y_center + offset;
                
                new_lines.push(((current_x, *y_center), (current_x + dx, y_new)));
                next_y_coords.push(y_new);
                next_y_ranges.push(y_separation);
            }
        }
        (next_y_coords, next_y_ranges, new_lines)
    }

    fn mirror_geometry(&mut self, first_half_lines: &[(Point, Point)]) {
        let (length, _) = self.box_dims;
        for (p1, p2) in first_half_lines {
            let mirrored_p1 = (length - p2.0, p2.1);
            let mirrored_p2 = (length - p1.0, p1.1);
            self.add_channel(mirrored_p1, mirrored_p2);
        }
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

pub fn create_geometry(
    box_dims: (f64, f64),
    splits: &[SplitType],
    config: &GeometryConfig,
) -> ChannelSystem {
    GeometryGenerator::new(box_dims, *config).generate(splits)
} 