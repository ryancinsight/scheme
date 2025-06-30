use super::mod_2d::{Channel, ChannelSystem, ChannelType, Node, Point2D, SplitType};
use crate::config::{ChannelTypeConfig, GeometryConfig, SerpentineConfig, ArcConfig};
use std::collections::HashMap;

/// Flow phase enumeration for enhanced arc direction calculation
#[derive(Debug, Clone, Copy, PartialEq)]
enum FlowPhase {
    Diverging,  // Channels spreading outward from center
    Converging, // Channels coming together toward center
    Mixed,      // Transitional or complex flow patterns
}

/// Local position relative to the center line
#[derive(Debug, Clone, Copy, PartialEq)]
enum LocalPosition {
    Upper,  // Above center line
    Center, // Near center line
    Lower,  // Below center line
}

/// Junction context for enhanced curvature decisions
#[derive(Debug, Clone)]
struct JunctionContext {
    near_inlet_junction: bool,
    near_outlet_junction: bool,
    is_junction_connector: bool,
    vertical_displacement: f64,
}

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
            ChannelTypeConfig::AllArcs(config) => self.create_arc_type(p1, p2, config),
            ChannelTypeConfig::MixedByPosition { middle_zone_fraction, serpentine_config, arc_config } => {
                let (length, _) = self.box_dims;
                let mid_x = length / 2.0;
                let channel_mid_x = (p1.0 + p2.0) / 2.0;
                let tolerance = length * middle_zone_fraction / 2.0;
                
                if (channel_mid_x - mid_x).abs() < tolerance {
                    self.create_serpentine_type(p1, p2, serpentine_config, None)
                } else if self.is_angled_channel(p1, p2) {
                    self.create_arc_type(p1, p2, arc_config)
                } else {
                    ChannelType::Straight
                }
            }
            ChannelTypeConfig::Smart { serpentine_config, arc_config } => {
                self.determine_smart_channel_type(p1, p2, serpentine_config, arc_config)
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

    /// Create an arc channel type
    fn create_arc_type(&self, p1: Point2D, p2: Point2D, config: ArcConfig) -> ChannelType {
        let path = self.generate_arc_path(p1, p2, config);
        ChannelType::Arc { path }
    }

    /// Generate a smooth arc path between two points with enhanced directional awareness
    fn generate_arc_path(&self, p1: Point2D, p2: Point2D, config: ArcConfig) -> Vec<Point2D> {
        let mut path = Vec::with_capacity(config.smoothness + 2);
        
        // Calculate the direction and distance
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // For very short channels or zero curvature, just return a straight line
        if distance < 1e-6 || config.curvature_factor < 1e-6 {
            path.push(p1);
            path.push(p2);
            return path;
        }
        
        // Calculate the control point for the arc with enhanced directional awareness
        let mid_x = (p1.0 + p2.0) / 2.0;
        let mid_y = (p1.1 + p2.1) / 2.0;
        
        // Calculate directional arc curvature with enhanced context
        let arc_direction = self.calculate_enhanced_arc_direction(p1, p2);
        
        // Calculate perpendicular direction for arc curvature
        let perp_x = -dy / distance;
        let perp_y = dx / distance;
        
        // Apply directional multiplier
        let directed_perp_x = perp_x * arc_direction;
        let directed_perp_y = perp_y * arc_direction;
        
        // Arc height based on curvature factor and distance
        let arc_height = distance * config.curvature_factor * 0.5;
        
        // Control point offset from midpoint
        let control_x = mid_x + directed_perp_x * arc_height;
        let control_y = mid_y + directed_perp_y * arc_height;
        
        // Generate points along the quadratic Bezier curve
        path.push(p1);
        
        for i in 1..config.smoothness + 1 {
            let t = i as f64 / (config.smoothness + 1) as f64;
            let t_inv = 1.0 - t;
            
            // Quadratic Bezier formula: B(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
            let x = t_inv * t_inv * p1.0 + 2.0 * t_inv * t * control_x + t * t * p2.0;
            let y = t_inv * t_inv * p1.1 + 2.0 * t_inv * t * control_y + t * t * p2.1;
            
            path.push((x, y));
        }
        
        path.push(p2);
        path
    }

    /// Enhanced arc direction calculation with better context awareness
    fn calculate_enhanced_arc_direction(&self, p1: Point2D, p2: Point2D) -> f64 {
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        
        // Check if this is a mostly horizontal channel
        let is_mostly_horizontal = dy.abs() < dx.abs() * 0.5;
        
        if is_mostly_horizontal {
            // For mostly horizontal channels, apply subtle curvature based on position
            let box_center_y = self.box_dims.1 / 2.0;
            let channel_center_y = (p1.1 + p2.1) / 2.0;
            let is_above_center = channel_center_y > box_center_y;
            
            // Subtle curvature to maintain flow naturalness
            return if is_above_center { 0.2 } else { -0.2 };
        }
        
        // Enhanced flow phase detection
        let flow_phase = self.determine_flow_phase(p1, p2);
        let local_position = self.analyze_local_position(p1, p2);
        let junction_context = self.analyze_junction_context(p1, p2);
        
        // Calculate base direction using flow physics
        let base_direction = self.calculate_flow_based_direction(p1, p2, flow_phase, local_position);
        
        // Apply junction-aware adjustments
        let junction_adjusted = self.apply_junction_adjustments(base_direction, junction_context, p1, p2);
        
        junction_adjusted
    }

    /// Determine the flow phase with better context awareness
    fn determine_flow_phase(&self, p1: Point2D, p2: Point2D) -> FlowPhase {
        let box_center_x = self.box_dims.0 / 2.0;
        let channel_start_x = p1.0;
        let channel_end_x = p2.0;
        let channel_center_x = (p1.0 + p2.0) / 2.0;
        
        // Consider the overall network topology
        let distance_from_inlet = channel_center_x / self.box_dims.0;
        let distance_from_outlet = (self.box_dims.0 - channel_center_x) / self.box_dims.0;
        
        // Enhanced phase detection based on multiple factors
        if distance_from_inlet < 0.35 {
            // First third: clearly diverging
            FlowPhase::Diverging
        } else if distance_from_outlet < 0.35 {
            // Last third: clearly converging  
            FlowPhase::Converging
        } else {
            // Middle zone: determine based on local flow direction and context
            if (channel_end_x - channel_start_x).abs() < 1e-6 {
                // Vertical segment in middle - could be either, use position
                if channel_center_x < box_center_x { FlowPhase::Diverging } else { FlowPhase::Converging }
            } else {
                // Use flow direction to determine phase
                let flowing_toward_center = (channel_end_x > channel_start_x && channel_center_x > box_center_x) ||
                                          (channel_end_x < channel_start_x && channel_center_x < box_center_x);
                if flowing_toward_center { FlowPhase::Converging } else { FlowPhase::Diverging }
            }
        }
    }

    /// Analyze local position context
    fn analyze_local_position(&self, p1: Point2D, p2: Point2D) -> LocalPosition {
        let box_center_y = self.box_dims.1 / 2.0;
        let channel_center_y = (p1.1 + p2.1) / 2.0;
        let relative_position = (channel_center_y - box_center_y) / (self.box_dims.1 / 2.0);
        
        // Determine position with tolerance for center detection
        if relative_position.abs() < 0.15 {
            LocalPosition::Center
        } else if relative_position > 0.15 {
            LocalPosition::Upper
        } else {
            LocalPosition::Lower
        }
    }

    /// Analyze junction context for better curvature decisions
    fn analyze_junction_context(&self, p1: Point2D, p2: Point2D) -> JunctionContext {
        let junction_proximity_threshold = self.box_dims.0 * 0.1; // 10% of box width
        
        let distance_from_left = p1.0.min(p2.0);
        let distance_from_right = self.box_dims.0 - p1.0.max(p2.0);
        
        let near_inlet_junction = distance_from_left < junction_proximity_threshold;
        let near_outlet_junction = distance_from_right < junction_proximity_threshold;
        
        // Check if this channel has significant vertical displacement (junction connection)
        let vertical_displacement = (p2.1 - p1.1).abs();
        let horizontal_displacement = (p2.0 - p1.0).abs();
        let is_junction_connector = vertical_displacement > horizontal_displacement * 0.3;
        
        JunctionContext {
            near_inlet_junction,
            near_outlet_junction,
            is_junction_connector,
            vertical_displacement,
        }
    }

    /// Calculate flow-based direction using fluid dynamics principles
    fn calculate_flow_based_direction(&self, p1: Point2D, p2: Point2D, flow_phase: FlowPhase, local_position: LocalPosition) -> f64 {
        let dx = p2.0 - p1.0;
        let flowing_right = dx > 0.0;
        
        match (flow_phase, local_position, flowing_right) {
            // Diverging phase: channels spread outward from center
            (FlowPhase::Diverging, LocalPosition::Upper, true) => 1.0,   // Upper: curve upward when flowing right
            (FlowPhase::Diverging, LocalPosition::Upper, false) => -1.0, // Upper: curve downward when flowing left
            (FlowPhase::Diverging, LocalPosition::Lower, true) => -1.0,  // Lower: curve downward when flowing right
            (FlowPhase::Diverging, LocalPosition::Lower, false) => 1.0,  // Lower: curve upward when flowing left
            (FlowPhase::Diverging, LocalPosition::Center, _) => 0.0,     // Center: minimal curvature
            
            // Converging phase: channels come together toward center
            (FlowPhase::Converging, LocalPosition::Upper, true) => -1.0,  // Upper: curve downward when flowing right
            (FlowPhase::Converging, LocalPosition::Upper, false) => 1.0,  // Upper: curve upward when flowing left
            (FlowPhase::Converging, LocalPosition::Lower, true) => 1.0,   // Lower: curve upward when flowing right
            (FlowPhase::Converging, LocalPosition::Lower, false) => -1.0, // Lower: curve downward when flowing left
            (FlowPhase::Converging, LocalPosition::Center, _) => 0.0,     // Center: minimal curvature
            
            // Mixed phase: use reduced curvature
            (FlowPhase::Mixed, LocalPosition::Upper, _) => 0.3,
            (FlowPhase::Mixed, LocalPosition::Lower, _) => -0.3,
            (FlowPhase::Mixed, LocalPosition::Center, _) => 0.0,
        }
    }

    /// Apply junction-specific adjustments to the base direction
    fn apply_junction_adjustments(&self, base_direction: f64, junction_context: JunctionContext, _p1: Point2D, _p2: Point2D) -> f64 {
        let mut adjusted_direction = base_direction;
        
        // Reduce curvature near junctions to maintain smooth transitions
        if junction_context.near_inlet_junction || junction_context.near_outlet_junction {
            adjusted_direction *= 0.6; // Reduce curvature by 40% near junctions
        }
        
        // For junction connectors (channels with significant vertical displacement)
        if junction_context.is_junction_connector {
            // Apply enhanced curvature for smoother junction transitions
            let enhancement_factor = (junction_context.vertical_displacement / self.box_dims.1).min(1.0);
            adjusted_direction *= 1.0 + enhancement_factor * 0.5;
        }
        
        // Ensure reasonable bounds
        adjusted_direction.max(-1.5).min(1.5)
    }

    /// Calculate the direction multiplier for arc curvature based on flow context
    /// (Backward compatibility method)
    fn calculate_arc_direction(&self, p1: Point2D, p2: Point2D, _is_diverging: bool, _is_above_center: bool) -> f64 {
        // Use the enhanced version for all calculations now
        self.calculate_enhanced_arc_direction(p1, p2)
    }

    /// Check if a channel is angled (not horizontal or vertical)
    fn is_angled_channel(&self, p1: Point2D, p2: Point2D) -> bool {
        let dx = (p2.0 - p1.0).abs();
        let dy = (p2.1 - p1.1).abs();
        let tolerance = 1e-6;
        
        // Channel is angled if it's neither purely horizontal nor purely vertical
        dx > tolerance && dy > tolerance
    }

    /// Determine channel type using smart logic
    fn determine_smart_channel_type(&self, p1: Point2D, p2: Point2D, serpentine_config: SerpentineConfig, arc_config: ArcConfig) -> ChannelType {
        let (length, _) = self.box_dims;
        let channel_mid_x = (p1.0 + p2.0) / 2.0;
        
        // Determine if this is in the first half, middle, or second half
        let first_third = length / 3.0;
        let second_third = 2.0 * length / 3.0;
        
        if channel_mid_x < first_third || channel_mid_x > second_third {
            // First and last thirds: use arcs for angled channels, straight for others
            if self.is_angled_channel(p1, p2) {
                // Check if this might be a center channel in a trifurcation
                if self.is_center_channel_in_trifurcation(p1, p2) {
                    ChannelType::Straight
                } else {
                    self.create_arc_type(p1, p2, arc_config)
                }
            } else {
                ChannelType::Straight
            }
        } else {
            // Middle third: use serpentine channels
            self.create_serpentine_type(p1, p2, serpentine_config, None)
        }
    }

    /// Check if this channel is likely a center channel in a trifurcation
    fn is_center_channel_in_trifurcation(&self, p1: Point2D, p2: Point2D) -> bool {
        let y_mid = (p1.1 + p2.1) / 2.0;
        let box_mid_y = self.box_dims.1 / 2.0;
        let tolerance = self.box_dims.1 * 0.1; // 10% tolerance
        
        // If the channel is near the center vertically, it might be a center channel
        (y_mid - box_mid_y).abs() < tolerance
    }

    fn add_channel_with_neighbors(&mut self, p1: Point2D, p2: Point2D, neighbor_y_coords: &[f64]) {
        let channel_type = match self.channel_type_config {
            ChannelTypeConfig::AllStraight => ChannelType::Straight,
            ChannelTypeConfig::AllSerpentine(config) => {
                self.create_serpentine_type(p1, p2, config, Some(neighbor_y_coords))
            },
            ChannelTypeConfig::AllArcs(config) => {
                self.create_arc_type(p1, p2, config)
            },
            ChannelTypeConfig::MixedByPosition { middle_zone_fraction, serpentine_config, arc_config } => {
                let (length, _) = self.box_dims;
                let mid_x = length / 2.0;
                let channel_mid_x = (p1.0 + p2.0) / 2.0;
                let tolerance = length * middle_zone_fraction / 2.0;
                
                if (channel_mid_x - mid_x).abs() < tolerance {
                    self.create_serpentine_type(p1, p2, serpentine_config, Some(neighbor_y_coords))
                } else if self.is_angled_channel(p1, p2) {
                    self.create_arc_type(p1, p2, arc_config)
                } else {
                    ChannelType::Straight
                }
            }
            ChannelTypeConfig::Smart { serpentine_config, arc_config } => {
                let smart_type = self.determine_smart_channel_type(p1, p2, serpentine_config, arc_config);
                match smart_type {
                    ChannelType::Serpentine { .. } => {
                        self.create_serpentine_type(p1, p2, serpentine_config, Some(neighbor_y_coords))
                    }
                    other => other,
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