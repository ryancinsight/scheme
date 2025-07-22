//! Channel type generation strategies
//!
//! This module implements the Strategy pattern for channel type generation,
//! providing a clean separation of concerns and enabling easy extension
//! with new channel types while adhering to SOLID principles.

use crate::geometry::{ChannelType, Point2D};
use crate::config::{ArcConfig, ChannelTypeConfig, GeometryConfig, SerpentineConfig, constants};

/// Strategy trait for generating different types of channels
/// 
/// This trait follows the Strategy pattern to enable different channel
/// generation algorithms while maintaining a consistent interface.
/// Each strategy is responsible for creating a specific type of channel
/// based on the provided points and configuration.
pub trait ChannelTypeStrategy {
    /// Create a channel type between two points
    /// 
    /// # Arguments
    /// * `from` - Starting point of the channel
    /// * `to` - Ending point of the channel
    /// * `geometry_config` - General geometry configuration
    /// * `box_dims` - Dimensions of the containing box
    /// * `total_branches` - Total number of branches in the system (for scaling)
    /// * `neighbor_info` - Optional information about neighboring channels
    /// 
    /// # Returns
    /// A `ChannelType` representing the generated channel
    fn create_channel(
        &self,
        from: Point2D,
        to: Point2D,
        geometry_config: &GeometryConfig,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> ChannelType;
}

/// Strategy for creating straight channels
#[derive(Debug, Clone)]
pub struct StraightChannelStrategy;

impl ChannelTypeStrategy for StraightChannelStrategy {
    fn create_channel(
        &self,
        _from: Point2D,
        _to: Point2D,
        _geometry_config: &GeometryConfig,
        _box_dims: (f64, f64),
        _total_branches: usize,
        _neighbor_info: Option<&[f64]>,
    ) -> ChannelType {
        ChannelType::Straight
    }
}

/// Strategy for creating serpentine channels
#[derive(Debug, Clone)]
pub struct SerpentineChannelStrategy {
    config: SerpentineConfig,
}

impl SerpentineChannelStrategy {
    pub fn new(config: SerpentineConfig) -> Self {
        Self { config }
    }
}

impl ChannelTypeStrategy for SerpentineChannelStrategy {
    fn create_channel(
        &self,
        from: Point2D,
        to: Point2D,
        geometry_config: &GeometryConfig,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> ChannelType {
        let path = self.generate_serpentine_path(
            from,
            to,
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        );
        ChannelType::Serpentine { path }
    }
}

impl SerpentineChannelStrategy {
    /// Generate a serpentine path between two points
    fn generate_serpentine_path(
        &self,
        p1: Point2D,
        p2: Point2D,
        geometry_config: &GeometryConfig,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> Vec<Point2D> {
        let n_points = 100;
        let mut path = Vec::with_capacity(n_points);

        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let channel_length = (dx * dx + dy * dy).sqrt();
        let _angle = dy.atan2(dx);

        let _branch_factor = (total_branches as f64).powf(0.75).max(1.0);

        // Calculate number of periods based on channel length for better scaling
        let base_wavelength = self.config.wavelength_factor * geometry_config.channel_width;
        
        // Scale the number of periods with channel length
        let length_based_periods = (channel_length / base_wavelength) * self.config.wave_density_factor;
        let periods = length_based_periods.max(1.0);

        // Calculate amplitude with neighbor awareness
        let amplitude = self.calculate_amplitude(
            p1,
            p2,
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        );

        // Gaussian envelope parameters
        let sigma = channel_length / self.config.gaussian_width_factor;
        let center = channel_length / 2.0;

        for i in 0..n_points {
            let t = i as f64 / (n_points - 1) as f64;
            let s = t * channel_length;

            // Base position along the line
            let base_x = p1.0 + t * dx;
            let base_y = p1.1 + t * dy;

            // Gaussian envelope for smooth transitions
            let envelope = (-0.5 * ((s - center) / sigma).powi(2)).exp();

            // Serpentine wave
            let wave_phase = 2.0 * std::f64::consts::PI * periods * t;
            let wave_amplitude = amplitude * envelope * wave_phase.sin();

            // Apply perpendicular offset
            let perp_x = -dy / channel_length;
            let perp_y = dx / channel_length;

            let x = base_x + wave_amplitude * perp_x;
            let y = base_y + wave_amplitude * perp_y;

            // Ensure exact endpoint alignment for first and last points
            if i == 0 {
                path.push(p1);
            } else if i == n_points - 1 {
                path.push(p2);
            } else {
                path.push((x, y));
            }
        }

        path
    }

    /// Calculate appropriate amplitude for serpentine channels
    fn calculate_amplitude(
        &self,
        p1: Point2D,
        p2: Point2D,
        geometry_config: &GeometryConfig,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> f64 {
        let channel_center_y = (p1.1 + p2.1) / 2.0;
        let box_height = box_dims.1;

        // Calculate safe amplitude based on neighbors or box boundaries
        let max_safe_amplitude = if let Some(neighbors) = neighbor_info {
            if neighbors.is_empty() {
                // No neighbors, use box boundaries
                let distance_to_top = box_height - channel_center_y;
                let distance_to_bottom = channel_center_y;
                distance_to_top.min(distance_to_bottom) * 0.8
            } else {
                // Find closest neighbor distances
                let mut min_distance = f64::INFINITY;
                for &neighbor_y in neighbors {
                    let distance = (neighbor_y - channel_center_y).abs();
                    if distance > 1e-6 {
                        min_distance = min_distance.min(distance);
                    }
                }
                (min_distance / 2.0) * 0.8
            }
        } else {
            // Fallback to box boundaries
            let distance_to_top = box_height - channel_center_y;
            let distance_to_bottom = channel_center_y;
            distance_to_top.min(distance_to_bottom) * 0.8
        };

        // Apply branch factor and fill factor
        let branch_factor = (total_branches as f64).powf(0.75).max(1.0);
        let fill_factor = self.config.fill_factor / branch_factor;
        let enhanced_fill_factor = (fill_factor * 1.5).min(0.95);
        
        (max_safe_amplitude * enhanced_fill_factor).max(geometry_config.channel_width * 0.5)
    }
}

/// Strategy for creating arc channels
#[derive(Debug, Clone)]
pub struct ArcChannelStrategy {
    config: ArcConfig,
}

impl ArcChannelStrategy {
    pub fn new(config: ArcConfig) -> Self {
        Self { config }
    }
}

impl ChannelTypeStrategy for ArcChannelStrategy {
    fn create_channel(
        &self,
        from: Point2D,
        to: Point2D,
        _geometry_config: &GeometryConfig,
        box_dims: (f64, f64),
        _total_branches: usize,
        _neighbor_info: Option<&[f64]>,
    ) -> ChannelType {
        let path = self.generate_arc_path(from, to, box_dims);
        ChannelType::Arc { path }
    }
}

impl ArcChannelStrategy {
    /// Generate a smooth arc path between two points
    fn generate_arc_path(&self, p1: Point2D, p2: Point2D, box_dims: (f64, f64)) -> Vec<Point2D> {
        let mut path = Vec::with_capacity(self.config.smoothness + 2);
        
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // For very short channels or zero curvature, return straight line
        if distance < 1e-6 || self.config.curvature_factor < 1e-6 {
            path.push(p1);
            path.push(p2);
            return path;
        }
        
        // Calculate control point for the arc
        let mid_x = (p1.0 + p2.0) / 2.0;
        let mid_y = (p1.1 + p2.1) / 2.0;
        
        // Calculate directional arc curvature
        let arc_direction = self.calculate_arc_direction(p1, p2, box_dims);
        
        // Calculate perpendicular direction for arc curvature
        let perp_x = -dy / distance;
        let perp_y = dx / distance;
        
        // Apply directional multiplier
        let directed_perp_x = perp_x * arc_direction;
        let directed_perp_y = perp_y * arc_direction;
        
        // Arc height based on curvature factor and distance
        let arc_height = distance * self.config.curvature_factor * 0.5;
        
        // Control point offset from midpoint
        let control_x = mid_x + directed_perp_x * arc_height;
        let control_y = mid_y + directed_perp_y * arc_height;
        
        // Generate points along the quadratic Bezier curve
        path.push(p1);
        
        for i in 1..self.config.smoothness + 1 {
            let t = i as f64 / (self.config.smoothness + 1) as f64;
            let t_inv = 1.0 - t;
            
            // Quadratic Bezier formula: B(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
            let x = t_inv * t_inv * p1.0 + 2.0 * t_inv * t * control_x + t * t * p2.0;
            let y = t_inv * t_inv * p1.1 + 2.0 * t_inv * t * control_y + t * t * p2.1;
            
            path.push((x, y));
        }
        
        path.push(p2);
        path
    }

    /// Calculate arc direction based on channel position and context
    fn calculate_arc_direction(&self, p1: Point2D, p2: Point2D, box_dims: (f64, f64)) -> f64 {
        // If curvature direction is explicitly set, use it
        if self.config.curvature_direction.abs() > 1e-6 {
            return self.config.curvature_direction;
        }

        // Auto-determine curvature direction for perfect mirror symmetry
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let box_center_x = box_dims.0 / 2.0;
        let box_center_y = box_dims.1 / 2.0;

        // Determine if this is a split (left half) or merge (right half) based on x-position
        let channel_center_x = (p1.0 + p2.0) / 2.0;
        let is_left_half = channel_center_x < box_center_x;

        // Check if this is a mostly horizontal channel
        let is_mostly_horizontal = dy.abs() < dx.abs() * 0.5;

        if is_mostly_horizontal {
            // For mostly horizontal channels, apply subtle curvature based on position
            let channel_center_y = (p1.1 + p2.1) / 2.0;
            let is_above_center = channel_center_y > box_center_y;

            if is_above_center {
                -1.0 // Curve downward for channels above center
            } else {
                1.0 // Curve upward for channels below center
            }
        } else {
            // For angled channels (splits/merges), create perfect mirror symmetry
            // Key insight: Within each split, branches should curve AWAY from each other
            // This creates the symmetric "bow-tie" or "lens" shape

            // Determine if this branch is above or below the center of its split group
            // For splits (left half): branches curve away from the split center
            // For merges (right half): branches curve away from the merge center (same pattern)

            if is_left_half {
                // Left half (splits): curve away from split center
                if dy > 0.0 {
                    // Upper branch: curve upward (away from center)
                    1.0
                } else {
                    // Lower branch: curve downward (away from center)
                    -1.0
                }
            } else {
                // Right half (merges): mirror the split pattern for perfect symmetry
                if dy < 0.0 {
                    // Upper branch (flowing toward center): curve upward (away from merge center)
                    1.0
                } else {
                    // Lower branch (flowing toward center): curve downward (away from merge center)
                    -1.0
                }
            }
        }
    }
}

/// Factory for creating channel type strategies based on configuration
///
/// This factory implements the Factory pattern to create appropriate
/// channel type strategies based on the provided configuration.
/// It encapsulates the logic for determining which strategy to use
/// and handles complex configurations like Smart and MixedByPosition.
pub struct ChannelTypeFactory;

impl ChannelTypeFactory {
    /// Create a strategy based on the channel type configuration
    ///
    /// # Arguments
    /// * `config` - The channel type configuration
    /// * `from` - Starting point of the channel (for context-aware strategies)
    /// * `to` - Ending point of the channel (for context-aware strategies)
    /// * `box_dims` - Dimensions of the containing box (for context-aware strategies)
    ///
    /// # Returns
    /// A boxed trait object implementing `ChannelTypeStrategy`
    pub fn create_strategy(
        config: &ChannelTypeConfig,
        from: Point2D,
        to: Point2D,
        box_dims: (f64, f64),
    ) -> Box<dyn ChannelTypeStrategy> {
        match config {
            ChannelTypeConfig::AllStraight => Box::new(StraightChannelStrategy),

            ChannelTypeConfig::AllSerpentine(serpentine_config) => {
                Box::new(SerpentineChannelStrategy::new(*serpentine_config))
            }

            ChannelTypeConfig::AllArcs(arc_config) => {
                Box::new(ArcChannelStrategy::new(*arc_config))
            }

            ChannelTypeConfig::MixedByPosition {
                middle_zone_fraction,
                serpentine_config,
                arc_config
            } => {
                let (length, _) = box_dims;
                let mid_x = length / 2.0;
                let channel_mid_x = (from.0 + to.0) / 2.0;
                let tolerance = length * middle_zone_fraction / 2.0;

                if (channel_mid_x - mid_x).abs() < tolerance {
                    Box::new(SerpentineChannelStrategy::new(*serpentine_config))
                } else if Self::is_angled_channel(from, to) {
                    Box::new(ArcChannelStrategy::new(*arc_config))
                } else {
                    Box::new(StraightChannelStrategy)
                }
            }

            ChannelTypeConfig::Smart { serpentine_config, arc_config } => {
                Self::create_smart_strategy(from, to, box_dims, *serpentine_config, *arc_config)
            }

            ChannelTypeConfig::Custom(func) => {
                // For custom functions, we create a wrapper strategy
                let channel_type = func(from, to, box_dims);
                Box::new(CustomChannelStrategy::new(channel_type))
            }
        }
    }

    /// Create a smart strategy based on channel characteristics
    fn create_smart_strategy(
        from: Point2D,
        to: Point2D,
        box_dims: (f64, f64),
        serpentine_config: SerpentineConfig,
        arc_config: ArcConfig,
    ) -> Box<dyn ChannelTypeStrategy> {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let length = (dx * dx + dy * dy).sqrt();

        // Smart logic: use serpentine for longer horizontal channels,
        // arcs for angled channels, straight for short channels
        if length > box_dims.0 * constants::strategy_thresholds::LONG_HORIZONTAL_THRESHOLD
            && dy.abs() < dx.abs() * constants::strategy_thresholds::HORIZONTAL_ANGLE_THRESHOLD {
            // Long horizontal channel - use serpentine
            Box::new(SerpentineChannelStrategy::new(serpentine_config))
        } else if Self::is_angled_channel(from, to)
            && length > box_dims.0 * constants::strategy_thresholds::MIN_ARC_LENGTH_THRESHOLD {
            // Angled channel of reasonable length - use arc
            Box::new(ArcChannelStrategy::new(arc_config))
        } else {
            // Default to straight
            Box::new(StraightChannelStrategy)
        }
    }

    /// Check if a channel is significantly angled
    fn is_angled_channel(from: Point2D, to: Point2D) -> bool {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;

        if dx.abs() < 1e-6 {
            return dy.abs() > 1e-6; // Vertical channel
        }

        let slope = dy / dx;
        slope.abs() > constants::strategy_thresholds::ANGLED_CHANNEL_SLOPE_THRESHOLD
    }
}

/// Strategy wrapper for custom channel type functions
///
/// This strategy wraps custom channel type functions to fit into
/// the strategy pattern while maintaining backward compatibility.
#[derive(Debug, Clone)]
pub struct CustomChannelStrategy {
    channel_type: ChannelType,
}

impl CustomChannelStrategy {
    pub fn new(channel_type: ChannelType) -> Self {
        Self { channel_type }
    }
}

impl ChannelTypeStrategy for CustomChannelStrategy {
    fn create_channel(
        &self,
        _from: Point2D,
        _to: Point2D,
        _geometry_config: &GeometryConfig,
        _box_dims: (f64, f64),
        _total_branches: usize,
        _neighbor_info: Option<&[f64]>,
    ) -> ChannelType {
        self.channel_type.clone()
    }
}
