use super::mod_2d::{Channel, ChannelSystem, ChannelType, Node, Point2D, SplitType};
use crate::config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig};
use std::collections::HashMap;

struct GeometryGenerator {
    box_dims: (f64, f64),
    nodes: Vec<Node>,
    channels: Vec<Channel>,
    node_counter: usize,
    channel_counter: usize,
    point_to_node_id: HashMap<(i64, i64), usize>,
    config: GeometryConfig,
    channel_type_config: ChannelTypeConfig,
    total_branches: usize,
}

impl GeometryGenerator {
    fn new(
        box_dims: (f64, f64),
        config: GeometryConfig,
        channel_type_config: ChannelTypeConfig,
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
            total_branches,
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
        self.nodes.push(Node { id, point: p });
        self.point_to_node_id.insert(key, id);
        self.node_counter += 1;
        id
    }

    fn determine_channel_type(&self, p1: Point2D, p2: Point2D) -> ChannelType {
        match self.channel_type_config {
            ChannelTypeConfig::AllStraight => ChannelType::Straight,
            ChannelTypeConfig::AllSerpentine(config) => self.create_serpentine_type(p1, p2, config, None),
            ChannelTypeConfig::MixedByPosition { middle_zone_fraction, serpentine_config } => {
                let (length, _) = self.box_dims;
                let mid_x = length / 2.0;
                let channel_mid_x = (p1.0 + p2.0) / 2.0;
                let tolerance = length * middle_zone_fraction / 2.0;
                
                if (channel_mid_x - mid_x).abs() < tolerance {
                    self.create_serpentine_type(p1, p2, serpentine_config, None)
                } else {
                    ChannelType::Straight
                }
            }
            ChannelTypeConfig::Custom(func) => func(p1, p2, self.box_dims),
        }
    }

    fn create_serpentine_type(&self, p1: Point2D, p2: Point2D, config: SerpentineConfig, neighbor_info: Option<&[f64]>) -> ChannelType {
        let n_points = 100;
        let mut path = Vec::with_capacity(n_points);

        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let channel_length = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);

        let branch_factor = (self.total_branches as f64).powf(0.75).max(1.0);

        // Calculate number of periods based on channel length for better scaling
        // Use a base wavelength but allow fractional periods for proportional scaling
        let base_wavelength = config.wavelength_factor * self.config.channel_width;
        
        // Scale the number of periods with channel length - longer channels get more waves
        // Use a minimum period count that scales with length, plus ensure reasonable density
        let min_periods_for_length = (channel_length / (base_wavelength * 2.0)).max(1.0);
        let dynamic_periods = min_periods_for_length * config.wave_density_factor;
        
        let dynamic_gaussian_width_factor = config.gaussian_width_factor * branch_factor;
        
        let wavelength = channel_length / dynamic_periods;
        
        // Calculate dynamic amplitude based on actual available space
        let y_center = (p1.1 + p2.1) / 2.0;
        let mut amplitude = self.calculate_dynamic_amplitude(y_center, config.fill_factor, neighbor_info);

        // Mirror vertically for channels in the bottom half
        if y_center < self.box_dims.1 / 2.0 {
            amplitude *= -1.0;
        }

        // Mirror horizontally for channels in the right half
        if (p1.0 + p2.0) / 2.0 > self.box_dims.0 / 2.0 {
            amplitude *= -1.0;
        }
        
        let sigma = channel_length / dynamic_gaussian_width_factor;

        for i in 0..n_points {
            let frac = i as f64 / (n_points - 1) as f64;
            let x_local = frac * channel_length;

            let sine_wave = amplitude * (2.0 * std::f64::consts::PI * x_local / wavelength).sin();
            
            let dist_from_center = x_local - channel_length / 2.0;
            let gaussian = (-dist_from_center.powi(2) / (2.0 * sigma.powi(2))).exp();
            
            let y_local = sine_wave * gaussian;

            let x_rotated = x_local * angle.cos() - y_local * angle.sin();
            let y_rotated = x_local * angle.sin() + y_local * angle.cos();

            // Ensure exact endpoint alignment for first and last points
            if i == 0 {
                path.push(p1);
            } else if i == n_points - 1 {
                path.push(p2);
            } else {
                path.push((p1.0 + x_rotated, p1.1 + y_rotated));
            }
        }
        
        ChannelType::Serpentine { path }
    }

    /// Calculate dynamic amplitude based on available space around the channel
    fn calculate_dynamic_amplitude(&self, y_center: f64, fill_factor: f64, neighbor_info: Option<&[f64]>) -> f64 {
        let safety_margin = self.config.channel_width * 1.5; // Safety margin to prevent overlap
        
        // Calculate distance to box walls
        let distance_to_top = self.box_dims.1 - y_center;
        let distance_to_bottom = y_center;
        let distance_to_walls = distance_to_top.min(distance_to_bottom);
        
        let available_space = if let Some(neighbors) = neighbor_info {
            // Calculate distance to nearest neighboring channels
            let mut min_neighbor_distance = distance_to_walls;
            
            for &neighbor_y in neighbors {
                if (neighbor_y - y_center).abs() > 0.01 { // Avoid self
                    let distance_to_neighbor = (neighbor_y - y_center).abs() / 2.0;
                    min_neighbor_distance = min_neighbor_distance.min(distance_to_neighbor);
                }
            }
            
            min_neighbor_distance
        } else {
            // Fallback to simple calculation if no neighbor info available
            let y_range_per_channel = self.box_dims.1 / self.total_branches as f64;
            y_range_per_channel / 2.0
        };
        
        // Calculate amplitude with safety margin and enhanced fill factor
        let max_safe_amplitude = (available_space - safety_margin).max(0.0);
        let enhanced_fill_factor = (fill_factor * 1.5).min(0.95); // Boost amplitude but cap at 95%
        
        (max_safe_amplitude * enhanced_fill_factor).max(self.config.channel_width * 0.5)
    }



    fn add_channel_with_neighbors(&mut self, p1: Point2D, p2: Point2D, neighbor_y_coords: &[f64]) {
        let channel_type = match self.channel_type_config {
            ChannelTypeConfig::AllStraight => ChannelType::Straight,
            ChannelTypeConfig::AllSerpentine(config) => {
                self.create_serpentine_type(p1, p2, config, Some(neighbor_y_coords))
            },
            ChannelTypeConfig::MixedByPosition { middle_zone_fraction, serpentine_config } => {
                let (length, _) = self.box_dims;
                let mid_x = length / 2.0;
                let channel_mid_x = (p1.0 + p2.0) / 2.0;
                let tolerance = length * middle_zone_fraction / 2.0;
                
                if (channel_mid_x - mid_x).abs() < tolerance {
                    self.create_serpentine_type(p1, p2, serpentine_config, Some(neighbor_y_coords))
                } else {
                    ChannelType::Straight
                }
            }
            ChannelTypeConfig::Custom(func) => func(p1, p2, self.box_dims),
        };
        
        self.add_channel_with_type(p1, p2, Some(channel_type));
    }

    fn add_channel_with_type(&mut self, p1: Point2D, p2: Point2D, channel_type: Option<ChannelType>) {
        let from_id = self.get_or_create_node(p1);
        let to_id = self.get_or_create_node(p2);
        let id = self.channel_counter;
        
        let final_channel_type = channel_type.unwrap_or_else(|| self.determine_channel_type(p1, p2));
        
        let channel = Channel {
            id,
            from_node: from_id,
            to_node: to_id,
            width: self.config.channel_width,
            height: self.config.channel_height,
            channel_type: final_channel_type,
        };
        
        self.channels.push(channel);
        self.channel_counter += 1;
    }

    fn generate(mut self, splits: &[SplitType]) -> ChannelSystem {
        let (length, width) = self.box_dims;

        if splits.is_empty() {
            let p1 = (0.0, width / 2.0);
            let p2 = (length, width / 2.0);
            // For single channel, pass empty neighbor list so it uses box boundaries
            self.add_channel_with_neighbors(p1, p2, &[]);
            return self.finalize();
        }
        
        let first_half_lines = self.generate_first_half(splits);

        // Collect y-coordinates for dynamic amplitude calculation
        let mut y_coords_for_amplitude: Vec<f64> = Vec::new();
        for (p1, p2) in &first_half_lines {
            y_coords_for_amplitude.push((p1.1 + p2.1) / 2.0);
        }
        
        for (p1, p2) in &first_half_lines {
            self.add_channel_with_neighbors(*p1, *p2, &y_coords_for_amplitude);
        }

        self.mirror_geometry(&first_half_lines);
        
        self.finalize()
    }

    fn generate_first_half(&self, splits: &[SplitType]) -> Vec<(Point2D, Point2D)> {
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
    ) -> (Vec<f64>, Vec<f64>, Vec<(Point2D, Point2D)>) {
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

    fn mirror_geometry(&mut self, first_half_lines: &[(Point2D, Point2D)]) {
        let (length, _) = self.box_dims;
        // Collect y-coordinates for the mirrored half as well
        let mut mirrored_y_coords: Vec<f64> = Vec::new();
        for (p1, p2) in first_half_lines {
            mirrored_y_coords.push((p1.1 + p2.1) / 2.0);
        }
        
        for (p1, p2) in first_half_lines {
            let mirrored_p1 = (length - p2.0, p2.1);
            let mirrored_p2 = (length - p1.0, p1.1);
            self.add_channel_with_neighbors(mirrored_p1, mirrored_p2, &mirrored_y_coords);
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
    channel_type_config: &ChannelTypeConfig,
) -> ChannelSystem {
    let total_branches = splits.iter().map(|s| s.branch_count()).product::<usize>().max(1);
    GeometryGenerator::new(box_dims, *config, *channel_type_config, total_branches).generate(splits)
} 