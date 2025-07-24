//! Channel type generation strategies
//!
//! This module implements the Strategy pattern for channel type generation,
//! providing a clean separation of concerns and enabling easy extension
//! with new channel types while adhering to SOLID principles.

use crate::geometry::{ChannelType, Point2D};
use crate::geometry::optimization::optimize_serpentine_parameters;
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

/// Strategy for creating smooth straight channels with transition zones
#[derive(Debug, Clone)]
pub struct SmoothStraightChannelStrategy {
    /// Configuration for transition zones
    pub transition_config: SmoothTransitionConfig,
}

/// Configuration for smooth transition zones in straight channels
#[derive(Debug, Clone, Copy)]
pub struct SmoothTransitionConfig {
    /// Length of transition zone as fraction of channel length (0.0 to 0.5)
    pub transition_length_factor: f64,
    /// Maximum amplitude of transition waves relative to channel width
    pub transition_amplitude_factor: f64,
    /// Number of points to use for transition smoothing
    pub transition_smoothness: usize,
}

impl Default for SmoothTransitionConfig {
    fn default() -> Self {
        Self {
            transition_length_factor: 0.15, // 15% of channel length for transitions
            transition_amplitude_factor: 0.3, // 30% of channel width for amplitude
            transition_smoothness: 20, // 20 points per transition zone
        }
    }
}

impl SmoothStraightChannelStrategy {
    pub fn new(config: SmoothTransitionConfig) -> Self {
        Self {
            transition_config: config,
        }
    }
}

impl ChannelTypeStrategy for SmoothStraightChannelStrategy {
    fn create_channel(
        &self,
        from: Point2D,
        to: Point2D,
        geometry_config: &GeometryConfig,
        _box_dims: (f64, f64),
        _total_branches: usize,
        _neighbor_info: Option<&[f64]>,
    ) -> ChannelType {
        let path = self.generate_smooth_straight_path(from, to, geometry_config);
        ChannelType::SmoothStraight { path }
    }
}

impl SmoothStraightChannelStrategy {
    /// Generate a smooth straight path with transition zones at endpoints
    fn generate_smooth_straight_path(
        &self,
        p1: Point2D,
        p2: Point2D,
        geometry_config: &GeometryConfig,
    ) -> Vec<Point2D> {
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let channel_length = (dx * dx + dy * dy).sqrt();

        // For very short channels, just return straight line
        if channel_length < geometry_config.channel_width * 2.0 {
            return vec![p1, p2];
        }

        let transition_length = channel_length * self.transition_config.transition_length_factor;
        let max_amplitude = geometry_config.channel_width * self.transition_config.transition_amplitude_factor;

        // Calculate total points: transition + middle + transition
        let transition_points = self.transition_config.transition_smoothness;
        let middle_points = 10; // Simple straight section
        let total_points = transition_points * 2 + middle_points;

        let mut path = Vec::with_capacity(total_points);

        // Perpendicular direction for wave displacement
        let perp_x = -dy / channel_length;
        let perp_y = dx / channel_length;

        for i in 0..total_points {
            let t = i as f64 / (total_points - 1) as f64;

            // Base position along the line
            let base_x = p1.0 + t * dx;
            let base_y = p1.1 + t * dy;

            // Calculate smooth transition amplitude
            let amplitude = self.calculate_transition_amplitude(t, transition_length / channel_length, max_amplitude);

            // Apply small wave for smooth transition
            let wave_phase = std::f64::consts::PI * 2.0 * t; // One complete wave across the channel
            let wave_amplitude = amplitude * wave_phase.sin();

            let x = base_x + wave_amplitude * perp_x;
            let y = base_y + wave_amplitude * perp_y;

            // Ensure exact endpoints
            if i == 0 {
                path.push(p1);
            } else if i == total_points - 1 {
                path.push(p2);
            } else {
                path.push((x, y));
            }
        }

        path
    }

    /// Calculate transition amplitude that smoothly goes to zero at endpoints
    fn calculate_transition_amplitude(&self, t: f64, transition_factor: f64, max_amplitude: f64) -> f64 {
        // Create smooth transitions at both ends
        let start_transition = if t < transition_factor {
            // Smooth ramp up from 0 to 1 using smoothstep
            let local_t = t / transition_factor;
            local_t * local_t * (3.0 - 2.0 * local_t)
        } else {
            1.0
        };

        let end_transition = if t > (1.0 - transition_factor) {
            // Smooth ramp down from 1 to 0 using smoothstep
            let local_t = (1.0 - t) / transition_factor;
            local_t * local_t * (3.0 - 2.0 * local_t)
        } else {
            1.0
        };

        max_amplitude * start_transition * end_transition
    }
}

/// Strategy for creating serpentine channels
#[derive(Debug, Clone)]
pub struct SerpentineChannelStrategy {
    config: SerpentineConfig,
}

impl SerpentineChannelStrategy {
    /// Create a new serpentine channel strategy with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration parameters for serpentine channel generation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::strategies::SerpentineChannelStrategy;
    /// use scheme::config::SerpentineConfig;
    ///
    /// let strategy = SerpentineChannelStrategy::new(SerpentineConfig::default());
    /// ```
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
        let path = if self.config.optimization_enabled {
            self.generate_optimized_serpentine_path(
                from,
                to,
                geometry_config,
                box_dims,
                total_branches,
                neighbor_info,
            )
        } else {
            self.generate_serpentine_path(
                from,
                to,
                geometry_config,
                box_dims,
                total_branches,
                neighbor_info,
            )
        };
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

        // Calculate number of periods to ensure complete wave cycles
        let base_wavelength = self.config.wavelength_factor * geometry_config.channel_width;

        // For smooth endpoint transitions, use half-periods to ensure zero amplitude at endpoints
        // Scale the number of periods with channel length and ensure minimum complete cycles
        let length_based_periods = (channel_length / base_wavelength) * self.config.wave_density_factor;
        let base_periods = length_based_periods.max(1.0); // Minimum 1 complete cycle
        // Round to nearest integer number of half-periods to ensure sin(π*n) = 0 at endpoints
        let half_periods = (base_periods * 2.0).round().max(1.0);

        // Calculate amplitude with neighbor awareness
        let amplitude = self.calculate_amplitude(
            p1,
            p2,
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        );

        // Calculate wave phase direction for perfect mirror symmetry
        let phase_direction = self.calculate_wave_phase_direction(p1, p2, box_dims);

        // Gaussian envelope parameters
        // Note: sigma and center are now calculated in the improved envelope function

        for i in 0..n_points {
            let t = i as f64 / (n_points - 1) as f64;

            // Base position along the line
            let base_x = p1.0 + t * dx;
            let base_y = p1.1 + t * dy;

            // Use smooth endpoint envelope for seamless transitions
            let smooth_envelope = self.calculate_smooth_endpoint_envelope(t);

            // Optionally combine with improved Gaussian envelope for middle sections
            let gaussian_envelope = self.calculate_improved_envelope(t, channel_length, dx, dy);
            let envelope = smooth_envelope * gaussian_envelope;

            // Serpentine wave with half-periods to ensure zero amplitude at endpoints
            let wave_phase = std::f64::consts::PI * half_periods * t;

            // Apply phase direction correctly for bilateral mirror symmetry
            // phase_direction determines the initial phase offset, not frequency scaling
            let phase_offset = if phase_direction > 0.0 {
                0.0 // Positive phase: start with sine wave (0 phase)
            } else {
                std::f64::consts::PI // Negative phase: start with inverted sine wave (π phase)
            };

            let wave_amplitude = amplitude * envelope * (wave_phase + phase_offset).sin();

            // Apply perpendicular offset
            let perp_x = -dy / channel_length;
            let perp_y = dx / channel_length;

            let x = base_x + wave_amplitude * perp_x;
            let y = base_y + wave_amplitude * perp_y;

            // Ensure exact endpoint matching for first and last points to maintain precision
            // The smooth envelope should make wave_amplitude ≈ 0 at endpoints, but we ensure exactness
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

    /// Generate an optimized serpentine path between two points
    fn generate_optimized_serpentine_path(
        &self,
        p1: Point2D,
        p2: Point2D,
        geometry_config: &GeometryConfig,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> Vec<Point2D> {
        // Run optimization to find best parameters
        let optimization_result = optimize_serpentine_parameters(
            p1,
            p2,
            geometry_config,
            &self.config,
            box_dims,
            neighbor_info,
        );

        // Create optimized configuration
        let optimized_config = SerpentineConfig {
            wavelength_factor: optimization_result.params.wavelength_factor,
            wave_density_factor: optimization_result.params.wave_density_factor,
            fill_factor: optimization_result.params.fill_factor,
            ..self.config
        };

        // Generate path with optimized parameters
        let strategy = SerpentineChannelStrategy::new(optimized_config);
        strategy.generate_serpentine_path(
            p1,
            p2,
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        )
    }

    /// Generate serpentine path for optimization purposes (public interface)
    pub fn generate_serpentine_path_for_optimization(
        &self,
        p1: Point2D,
        p2: Point2D,
        geometry_config: &GeometryConfig,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> Vec<Point2D> {
        self.generate_serpentine_path(
            p1,
            p2,
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        )
    }

    /// Calculate smooth endpoint envelope for seamless transitions
    ///
    /// This function creates a smooth envelope that ensures zero amplitude and
    /// zero derivative at the endpoints, eliminating sharp transitions.
    /// Uses smoothstep function: t²(3-2t) for C¹ continuity.
    fn calculate_smooth_endpoint_envelope(&self, t: f64) -> f64 {
        // Smoothstep function: t²(3-2t)
        // This has the properties:
        // - f(0) = 0, f(1) = 1
        // - f'(0) = 0, f'(1) = 0 (zero derivative at endpoints)
        // - Smooth C¹ continuity
        t * t * (3.0 - 2.0 * t)
    }

    /// Calculate improved Gaussian envelope with distance-based normalization
    ///
    /// This function creates a more sophisticated envelope that:
    /// 1. Normalizes based on the distance between start and end nodes
    /// 2. Provides special handling for the middle section where there's no directional change
    /// 3. Reduces amplitude near nodes to prevent intersection while maintaining full amplitude in the middle
    fn calculate_improved_envelope(&self, t: f64, channel_length: f64, dx: f64, dy: f64) -> f64 {
        // Calculate the actual distance between nodes (not just channel length)
        let node_distance = (dx * dx + dy * dy).sqrt();

        // Determine if this is primarily a horizontal channel (middle section logic)
        let is_horizontal = dx.abs() > dy.abs();
        let horizontal_ratio = dx.abs() / node_distance;

        // For horizontal channels (middle sections), we want less aggressive tapering
        // since there's no directional change at the nodes
        let middle_section_factor = if is_horizontal && horizontal_ratio > 0.8 {
            // This is a middle section - reduce the Gaussian effect
            0.3 + 0.7 * horizontal_ratio // Scale from 0.3 to 1.0 based on how horizontal it is
        } else {
            // This is a directional change section - use full Gaussian effect
            1.0
        };

        // Distance-based normalization: shorter channels need more aggressive tapering
        let distance_normalization = (node_distance / 10.0).min(1.0).max(0.1);

        // Calculate effective sigma based on distance and section type
        let base_sigma = channel_length / self.config.gaussian_width_factor;
        let effective_sigma = base_sigma * distance_normalization * middle_section_factor;

        // Center the envelope
        let center = 0.5; // Center in parameter space (t = 0.5)

        // Calculate Gaussian envelope
        let gaussian = (-0.5 * ((t - center) / (effective_sigma / channel_length)).powi(2)).exp();

        // For middle sections, add a plateau in the center to maintain full amplitude
        if is_horizontal && horizontal_ratio > 0.8 {
            let plateau_width = 0.4; // 40% of the channel has full amplitude
            let plateau_start = 0.5 - plateau_width / 2.0;
            let plateau_end = 0.5 + plateau_width / 2.0;

            if t >= plateau_start && t <= plateau_end {
                // In the plateau region, blend between Gaussian and full amplitude
                let plateau_factor = 1.0 - ((t - 0.5).abs() / (plateau_width / 2.0));
                gaussian.max(0.8 + 0.2 * plateau_factor)
            } else {
                gaussian
            }
        } else {
            gaussian
        }
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

    /// Calculate wave phase direction for perfect bilateral mirror symmetry
    fn calculate_wave_phase_direction(&self, p1: Point2D, p2: Point2D, box_dims: (f64, f64)) -> f64 {
        // If wave phase direction is explicitly set, use it
        if self.config.wave_phase_direction.abs() > 1e-6 {
            return self.config.wave_phase_direction;
        }

        // Auto-determine phase direction for perfect bilateral mirror symmetry
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
            // For horizontal channels, create perfect bilateral symmetry
            let channel_center_y = (p1.1 + p2.1) / 2.0;
            let is_above_center = channel_center_y > box_center_y;

            // Key insight: For perfect bilateral symmetry, the phase direction should be
            // consistent across the vertical centerline but opposite for upper/lower branches
            if is_above_center {
                1.0 // Upper channels: positive phase (consistent across left/right)
            } else {
                -1.0 // Lower channels: negative phase (consistent across left/right)
            }
        } else {
            // For angled channels (splits/merges), create perfect bilateral mirror symmetry
            // Key insight: The right half should be a PERFECT MIRROR of the left half

            if is_left_half {
                // Left half (splits): Use position relative to center for consistent phasing
                let channel_center_y = (p1.1 + p2.1) / 2.0;
                let is_above_center = channel_center_y > box_center_y;

                if is_above_center {
                    1.0 // Upper branch: positive phase
                } else {
                    -1.0 // Lower branch: negative phase
                }
            } else {
                // Right half (merges): MIRROR the left half exactly
                // For perfect bilateral symmetry, use the same logic as left half
                let channel_center_y = (p1.1 + p2.1) / 2.0;
                let is_above_center = channel_center_y > box_center_y;

                if is_above_center {
                    1.0 // Upper branch: positive phase (mirrors left upper)
                } else {
                    -1.0 // Lower branch: negative phase (mirrors left lower)
                }
            }
        }
    }
}

/// Strategy for creating arc channels
#[derive(Debug, Clone)]
pub struct ArcChannelStrategy {
    config: ArcConfig,
}

impl ArcChannelStrategy {
    /// Create a new arc channel strategy with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration parameters for arc channel generation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::strategies::ArcChannelStrategy;
    /// use scheme::config::ArcConfig;
    ///
    /// let strategy = ArcChannelStrategy::new(ArcConfig::default());
    /// ```
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

            ChannelTypeConfig::AllSmoothStraight(smooth_config) => {
                Box::new(SmoothStraightChannelStrategy::new(*smooth_config))
            }

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

            ChannelTypeConfig::SmoothSerpentineWithTransitions { serpentine_config, smooth_straight_config } => {
                Self::create_smooth_serpentine_strategy(from, to, box_dims, *serpentine_config, *smooth_straight_config)
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

    /// Create a smooth serpentine strategy with smooth straight junction connectors
    fn create_smooth_serpentine_strategy(
        from: Point2D,
        to: Point2D,
        box_dims: (f64, f64),
        serpentine_config: SerpentineConfig,
        smooth_straight_config: SmoothTransitionConfig,
    ) -> Box<dyn ChannelTypeStrategy> {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let length = (dx * dx + dy * dy).sqrt();

        // Use serpentine for longer horizontal channels (branches)
        // Use smooth straight for shorter channels and junction connectors
        if length > box_dims.0 * constants::strategy_thresholds::LONG_HORIZONTAL_THRESHOLD
            && dy.abs() < dx.abs() * constants::strategy_thresholds::HORIZONTAL_ANGLE_THRESHOLD {
            // Long horizontal channel - use serpentine
            Box::new(SerpentineChannelStrategy::new(serpentine_config))
        } else {
            // Junction connectors and short channels - use smooth straight
            Box::new(SmoothStraightChannelStrategy::new(smooth_straight_config))
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
    /// Create a new direct channel strategy with the given channel type
    ///
    /// This strategy directly uses the provided channel type without any
    /// configuration-based selection logic.
    ///
    /// # Arguments
    ///
    /// * `channel_type` - The specific channel type to use for all channels
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scheme::geometry::strategies::CustomChannelStrategy;
    /// use scheme::geometry::ChannelType;
    ///
    /// let strategy = CustomChannelStrategy::new(ChannelType::Straight);
    /// ```
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
