//! Channel type generation strategies
//!
//! This module implements the Strategy pattern for channel type generation,
//! providing a clean separation of concerns and enabling easy extension
//! with new channel types while adhering to SOLID principles.

use crate::geometry::{ChannelType, Point2D};
use crate::geometry::optimization::optimize_serpentine_parameters;
use crate::config::{ArcConfig, ChannelTypeConfig, GeometryConfig, SerpentineConfig, constants};
use crate::config_constants::ConstantsRegistry;
use crate::state_management::bilateral_symmetry::{
    SymmetryContext, BilateralSymmetryConfig, BilateralPhaseDirectionCalculator, ChannelPositionClassification
};

/// Context object for channel generation to reduce parameter coupling
///
/// This struct groups related parameters together, following the
/// Parameter Object pattern to improve method signatures and reduce coupling.
#[derive(Debug, Clone)]
pub struct ChannelGenerationContext<'a> {
    /// Geometry configuration parameters
    pub geometry_config: &'a GeometryConfig,
    /// Bounding box dimensions (width, height)
    pub box_dims: (f64, f64),
    /// Total number of branches in the system
    pub total_branches: usize,
    /// Information about neighboring channels for collision avoidance
    pub neighbor_info: Option<&'a [f64]>,
}

impl<'a> ChannelGenerationContext<'a> {
    /// Create a new channel generation context
    pub fn new(
        geometry_config: &'a GeometryConfig,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&'a [f64]>,
    ) -> Self {
        Self {
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        }
    }
}

/// Trait for calculating envelope functions for serpentine channels
///
/// This trait abstracts envelope calculation logic to eliminate code duplication
/// and provide a clean interface for different envelope types.
pub trait EnvelopeCalculator {
    /// Calculate envelope value at parameter t (0.0 to 1.0)
    fn calculate_envelope(&self, t: f64, context: &EnvelopeContext) -> f64;
}

/// Context for envelope calculations
#[derive(Debug, Clone)]
pub struct EnvelopeContext {
    /// Channel length
    pub channel_length: f64,
    /// Channel direction vector (dx, dy)
    pub direction: (f64, f64),
    /// Distance to nearest node
    pub node_distance: f64,
    /// Adaptive configuration
    pub adaptive_config: crate::config::AdaptiveSerpentineConfig,
    /// Gaussian width factor
    pub gaussian_width_factor: f64,
}

/// Smooth endpoint envelope calculator
///
/// Provides smooth transitions at channel endpoints using smoothstep function.
pub struct SmoothEndpointEnvelopeCalculator;

impl EnvelopeCalculator for SmoothEndpointEnvelopeCalculator {
    fn calculate_envelope(&self, t: f64, _context: &EnvelopeContext) -> f64 {
        // Smooth endpoint envelope using smoothstep function
        // This ensures zero amplitude and zero derivative at endpoints
        let smoothstep = |x: f64| x * x * (3.0 - 2.0 * x);

        // Create smooth transitions at both ends
        let start_transition = if t < 0.1 {
            smoothstep(t / 0.1)
        } else { 1.0 };

        let end_transition = if t > 0.9 {
            smoothstep((1.0 - t) / 0.1)
        } else { 1.0 };

        start_transition * end_transition
    }
}

/// Gaussian envelope calculator with adaptive behavior
///
/// Provides Gaussian-shaped envelope with adaptive parameters based on
/// channel characteristics and proximity to nodes/walls.
pub struct AdaptiveGaussianEnvelopeCalculator;

impl EnvelopeCalculator for AdaptiveGaussianEnvelopeCalculator {
    fn calculate_envelope(&self, t: f64, context: &EnvelopeContext) -> f64 {
        let dx = context.direction.0;
        let dy = context.direction.1;
        let channel_length = context.channel_length;
        let node_distance = context.node_distance;

        // Determine if this is primarily a horizontal channel
        let is_horizontal = dx.abs() > dy.abs();
        let horizontal_ratio = dx.abs() / node_distance;

        // For horizontal channels (middle sections), we want less aggressive tapering
        let middle_section_factor = if is_horizontal && horizontal_ratio > context.adaptive_config.horizontal_ratio_threshold {
            context.adaptive_config.middle_section_amplitude_factor +
            (1.0 - context.adaptive_config.middle_section_amplitude_factor) * horizontal_ratio
        } else {
            1.0
        };

        // Distance-based normalization
        let distance_normalization = if context.adaptive_config.enable_distance_based_scaling {
            (node_distance / context.adaptive_config.node_distance_normalization).min(1.0).max(0.1)
        } else {
            1.0
        };

        // Calculate effective sigma based on distance and section type
        let base_sigma = channel_length / context.gaussian_width_factor;
        let effective_sigma = base_sigma * distance_normalization * middle_section_factor;

        // Center the envelope
        let center = 0.5;

        // Calculate Gaussian envelope
        let gaussian = (-0.5 * ((t - center) / (effective_sigma / channel_length)).powi(2)).exp();

        // For middle sections, add a plateau in the center to maintain full amplitude
        if is_horizontal && horizontal_ratio > context.adaptive_config.horizontal_ratio_threshold {
            let plateau_width = context.adaptive_config.plateau_width_factor;
            let plateau_start = 0.5 - plateau_width / 2.0;
            let plateau_end = 0.5 + plateau_width / 2.0;

            if t >= plateau_start && t <= plateau_end {
                // In the plateau region, blend between Gaussian and full amplitude
                let plateau_factor = 1.0 - ((t - 0.5).abs() / (plateau_width / 2.0));
                gaussian.max(context.adaptive_config.plateau_amplitude_factor +
                    (1.0 - context.adaptive_config.plateau_amplitude_factor) * plateau_factor)
            } else {
                gaussian
            }
        } else {
            gaussian
        }
    }
}

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
///
/// This configuration controls the smooth transition zones at the endpoints
/// of straight channels to eliminate sharp corners when connecting to other
/// channel types.
///
/// # Examples
///
/// ```rust
/// use scheme::geometry::strategies::SmoothTransitionConfig;
///
/// // Create with default values
/// let config = SmoothTransitionConfig::default();
///
/// // Create with custom values for subtle transitions
/// let subtle = SmoothTransitionConfig {
///     transition_length_factor: 0.1,
///     transition_amplitude_factor: 0.2,
///     transition_smoothness: 15,
///     wave_multiplier: 1.5,
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SmoothTransitionConfig {
    /// Length of transition zone as fraction of channel length (0.0 to 0.5)
    pub transition_length_factor: f64,
    /// Maximum amplitude of transition waves relative to channel width (0.0 to 1.0)
    pub transition_amplitude_factor: f64,
    /// Number of points to use for transition smoothing (5 to 100)
    pub transition_smoothness: usize,
    /// Wave multiplier for transition waves (0.5 to 10.0, where 2.0 = one complete wave)
    pub wave_multiplier: f64,
}

impl Default for SmoothTransitionConfig {
    fn default() -> Self {
        Self {
            transition_length_factor: 0.15, // 15% of channel length for transitions
            transition_amplitude_factor: 0.3, // 30% of channel width for amplitude
            transition_smoothness: 20, // 20 points per transition zone
            wave_multiplier: 2.0, // One complete wave across the channel
        }
    }
}

impl SmoothTransitionConfig {
    /// Create a new smooth transition configuration with validation
    pub fn new(
        transition_length_factor: f64,
        transition_amplitude_factor: f64,
        transition_smoothness: usize,
        wave_multiplier: f64,
    ) -> Result<Self, String> {
        let config = Self {
            transition_length_factor,
            transition_amplitude_factor,
            transition_smoothness,
            wave_multiplier,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.transition_length_factor < 0.0 || self.transition_length_factor > 0.5 {
            return Err("transition_length_factor must be between 0.0 and 0.5".to_string());
        }

        if self.transition_amplitude_factor < 0.0 || self.transition_amplitude_factor > 1.0 {
            return Err("transition_amplitude_factor must be between 0.0 and 1.0".to_string());
        }

        if self.transition_smoothness < 5 || self.transition_smoothness > 100 {
            return Err("transition_smoothness must be between 5 and 100".to_string());
        }

        if self.wave_multiplier < 0.5 || self.wave_multiplier > 10.0 {
            return Err("wave_multiplier must be between 0.5 and 10.0".to_string());
        }

        Ok(())
    }

    /// Create a subtle transition configuration
    pub fn subtle() -> Self {
        Self {
            transition_length_factor: 0.1,
            transition_amplitude_factor: 0.2,
            transition_smoothness: 15,
            wave_multiplier: 1.5,
        }
    }

    /// Create a pronounced transition configuration
    pub fn pronounced() -> Self {
        Self {
            transition_length_factor: 0.25,
            transition_amplitude_factor: 0.5,
            transition_smoothness: 30,
            wave_multiplier: 3.0,
        }
    }

    /// Create a high-quality transition configuration for detailed work
    pub fn high_quality() -> Self {
        Self {
            transition_length_factor: 0.2,
            transition_amplitude_factor: 0.4,
            transition_smoothness: 50,
            wave_multiplier: 2.0,
        }
    }

    /// Create a fast transition configuration for quick generation
    pub fn fast() -> Self {
        Self {
            transition_length_factor: 0.15,
            transition_amplitude_factor: 0.3,
            transition_smoothness: 10,
            wave_multiplier: 2.0,
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
        let middle_points = geometry_config.generation.smooth_straight_middle_points;
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
            let wave_phase = std::f64::consts::PI * self.transition_config.wave_multiplier * t;
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
        let context = ChannelGenerationContext::new(
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        );

        let path = if self.config.optimization_enabled {
            self.generate_optimized_serpentine_path(from, to, &context)
        } else {
            self.generate_serpentine_path(from, to, &context)
        };
        ChannelType::Serpentine { path }
    }
}

impl SerpentineChannelStrategy {
    /// Calculate wave amplitude based on wave shape and phase
    fn calculate_wave_amplitude(&self, wave_phase: f64, phase_offset: f64) -> f64 {
        use crate::config::WaveShape;

        match self.config.wave_shape {
            WaveShape::Sine => {
                // Smooth sine wave
                (wave_phase + phase_offset).sin()
            }
            WaveShape::Square => {
                // Square wave with smooth transitions
                let sine_value = (wave_phase + phase_offset).sin();
                // Use tanh to create smooth square wave transitions
                let constants = ConstantsRegistry::new();
                let sharpness = constants.get_square_wave_sharpness(); // Configurable transition sharpness
                (sharpness * sine_value).tanh()
            }
        }
    }

    /// Generate a serpentine path between two points
    fn generate_serpentine_path(
        &self,
        p1: Point2D,
        p2: Point2D,
        context: &ChannelGenerationContext,
    ) -> Vec<Point2D> {
        let n_points = context.geometry_config.generation.serpentine_points;
        let mut path = Vec::with_capacity(n_points);

        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let channel_length = (dx * dx + dy * dy).sqrt();
        let _angle = dy.atan2(dx);

        let constants = ConstantsRegistry::new();
        let _branch_factor = (context.total_branches as f64).powf(constants.get_branch_factor_exponent()).max(1.0);

        // Calculate number of periods to ensure complete wave cycles
        let base_wavelength = self.config.wavelength_factor * context.geometry_config.channel_width;

        // For smooth endpoint transitions, use half-periods to ensure zero amplitude at endpoints
        // Scale the number of periods with channel length and ensure minimum complete cycles
        let length_based_periods = (channel_length / base_wavelength) * self.config.wave_density_factor;
        let base_periods = length_based_periods.max(1.0); // Minimum 1 complete cycle
        // Round to nearest integer number of half-periods to ensure sin(π*n) = 0 at endpoints
        let half_periods = (base_periods * 2.0).round().max(1.0);

        // Calculate amplitude with neighbor awareness
        let amplitude = self.calculate_amplitude(p1, p2, context);

        // Calculate wave phase direction for perfect mirror symmetry
        let phase_direction = self.calculate_wave_phase_direction(p1, p2, context.box_dims);

        // Gaussian envelope parameters
        // Note: sigma and center are now calculated in the improved envelope function

        for i in 0..n_points {
            let t = i as f64 / (n_points - 1) as f64;

            // Base position along the line
            let base_x = p1.0 + t * dx;
            let base_y = p1.1 + t * dy;

            // Use envelope calculators for clean separation of concerns
            let smooth_calculator = SmoothEndpointEnvelopeCalculator;
            let gaussian_calculator = AdaptiveGaussianEnvelopeCalculator;

            let envelope_context = EnvelopeContext {
                channel_length,
                direction: (dx, dy),
                node_distance: channel_length, // Simplified for now
                adaptive_config: self.config.adaptive_config,
                gaussian_width_factor: self.config.gaussian_width_factor,
            };

            let smooth_envelope = smooth_calculator.calculate_envelope(t, &envelope_context);
            let gaussian_envelope = gaussian_calculator.calculate_envelope(t, &envelope_context);
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

            let wave_amplitude = amplitude * envelope * self.calculate_wave_amplitude(wave_phase, phase_offset);

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
        context: &ChannelGenerationContext,
    ) -> Vec<Point2D> {
        // Run optimization to find best parameters
        let optimization_result = optimize_serpentine_parameters(
            p1,
            p2,
            context.geometry_config,
            &self.config,
            context.box_dims,
            context.neighbor_info,
        );

        // Create optimized configuration without full clone
        let optimized_config = SerpentineConfig {
            wavelength_factor: optimization_result.params.wavelength_factor,
            wave_density_factor: optimization_result.params.wave_density_factor,
            fill_factor: optimization_result.params.fill_factor,
            gaussian_width_factor: self.config.gaussian_width_factor,
            wave_phase_direction: self.config.wave_phase_direction,
            wave_shape: self.config.wave_shape,
            optimization_enabled: false, // Disable nested optimization
            target_fill_ratio: self.config.target_fill_ratio,
            optimization_profile: self.config.optimization_profile,
            adaptive_config: self.config.adaptive_config,
        };

        // Generate path with optimized parameters using temporary strategy
        let temp_strategy = SerpentineChannelStrategy::new(optimized_config);
        temp_strategy.generate_serpentine_path(p1, p2, context)
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
        let context = ChannelGenerationContext::new(
            geometry_config,
            box_dims,
            total_branches,
            neighbor_info,
        );
        self.generate_serpentine_path(p1, p2, &context)
    }



    /// Calculate appropriate amplitude for serpentine channels
    fn calculate_amplitude(
        &self,
        p1: Point2D,
        p2: Point2D,
        context: &ChannelGenerationContext,
    ) -> f64 {
        let channel_center_y = (p1.1 + p2.1) / 2.0;
        let box_height = context.box_dims.1;

        // Calculate safe amplitude based on neighbors or box boundaries
        let max_safe_amplitude = if let Some(neighbors) = context.neighbor_info {
            if neighbors.is_empty() {
                // No neighbors, use box boundaries
                let distance_to_top = box_height - channel_center_y;
                let distance_to_bottom = channel_center_y;
                let wall_distance = distance_to_top.min(distance_to_bottom);
                if self.config.adaptive_config.enable_wall_proximity_scaling {
                    wall_distance * 0.8
                } else {
                    wall_distance * self.config.fill_factor
                }
            } else {
                // Find closest neighbor distances
                let mut min_distance = f64::INFINITY;
                for &neighbor_y in neighbors {
                    let distance = (neighbor_y - channel_center_y).abs();
                    if distance > 1e-6 {
                        min_distance = min_distance.min(distance);
                    }
                }
                let neighbor_amplitude = if self.config.adaptive_config.enable_neighbor_avoidance {
                    (min_distance / 2.0) * 0.8 // Conservative neighbor avoidance
                } else {
                    (min_distance / 2.0) * self.config.fill_factor // Use fill factor
                };
                neighbor_amplitude
            }
        } else {
            // Fallback to box boundaries
            let distance_to_top = box_height - channel_center_y;
            let distance_to_bottom = channel_center_y;
            let wall_distance = distance_to_top.min(distance_to_bottom);
            if self.config.adaptive_config.enable_wall_proximity_scaling {
                wall_distance * 0.8
            } else {
                wall_distance * self.config.fill_factor
            }
        };

        // Apply branch factor and fill factor
        let constants = ConstantsRegistry::new();
        let branch_factor = (context.total_branches as f64).powf(constants.get_branch_factor_exponent()).max(1.0);
        let fill_factor = self.config.fill_factor / branch_factor;
        let enhanced_fill_factor = (fill_factor * constants.get_fill_factor_enhancement()).min(0.95);
        
        (max_safe_amplitude * enhanced_fill_factor).max(context.geometry_config.channel_width * 0.5)
    }

    /// Calculate wave phase direction for perfect bilateral mirror symmetry using enhanced symmetry system
    fn calculate_wave_phase_direction(&self, p1: Point2D, p2: Point2D, box_dims: (f64, f64)) -> f64 {
        // If wave phase direction is explicitly set, use it
        if self.config.wave_phase_direction.abs() > 1e-6 {
            return self.config.wave_phase_direction;
        }

        // Use enhanced bilateral symmetry system for perfect symmetry
        let symmetry_config = BilateralSymmetryConfig {
            enable_vertical_symmetry: true,
            enable_horizontal_symmetry: true,
            symmetry_tolerance: 1e-6,
            enable_adaptive_symmetry: true,
            enforcement_strength: 1.0,
        };

        // Create a temporary channel generation context for symmetry calculation
        let temp_context = crate::state_management::adaptive::ChannelGenerationContext::new(
            GeometryConfig::default(), // This will be replaced with actual config in full integration
            box_dims,
            4, // Default branch count
            None,
        ).with_endpoints(p1, p2);

        // Create symmetry context
        let symmetry_context = SymmetryContext::new(temp_context, symmetry_config);

        // Use bilateral phase direction calculator for perfect symmetry
        let phase_calculator = BilateralPhaseDirectionCalculator::default();

        match phase_calculator.calculate_phase_direction(&symmetry_context) {
            Ok(phase_direction) => phase_direction,
            Err(_) => {
                // Fallback to legacy calculation if enhanced system fails
                self.calculate_legacy_phase_direction(p1, p2, box_dims)
            }
        }
    }

    /// Legacy phase direction calculation for fallback
    fn calculate_legacy_phase_direction(&self, p1: Point2D, p2: Point2D, box_dims: (f64, f64)) -> f64 {
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
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> ChannelType {
        let path = self.generate_arc_path_with_collision_prevention(from, to, box_dims, total_branches, neighbor_info);
        ChannelType::Arc { path }
    }
}

impl ArcChannelStrategy {
    /// Generate a smooth arc path with collision prevention
    fn generate_arc_path_with_collision_prevention(
        &self,
        p1: Point2D,
        p2: Point2D,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> Vec<Point2D> {
        if !self.config.enable_collision_prevention {
            return self.generate_arc_path(p1, p2, box_dims);
        }

        // Calculate adaptive curvature factor based on proximity to neighbors
        let adaptive_curvature = self.calculate_adaptive_curvature(p1, p2, box_dims, total_branches, neighbor_info);

        // Create temporary config with adaptive curvature
        let adaptive_config = ArcConfig {
            curvature_factor: adaptive_curvature,
            ..self.config
        };

        // Generate path with adaptive curvature
        let temp_strategy = ArcChannelStrategy::new(adaptive_config);
        temp_strategy.generate_arc_path(p1, p2, box_dims)
    }

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

    /// Calculate adaptive curvature factor based on neighbor proximity
    fn calculate_adaptive_curvature(
        &self,
        p1: Point2D,
        p2: Point2D,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> f64 {
        if !self.config.enable_adaptive_curvature {
            return self.config.curvature_factor;
        }

        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let channel_length = (dx * dx + dy * dy).sqrt();

        // Base curvature factor
        let mut adaptive_factor = self.config.curvature_factor;

        // Calculate proximity-based reduction
        let proximity_reduction = self.calculate_proximity_reduction(p1, p2, box_dims, total_branches, neighbor_info);

        // Apply proximity reduction with limits
        adaptive_factor *= (1.0 - proximity_reduction).max(self.config.max_curvature_reduction);

        // Additional safety check for very short channels
        if channel_length < self.config.min_separation_distance * 2.0 {
            adaptive_factor *= 0.5; // Reduce curvature for very short channels
        }

        // Ensure we don't go below minimum curvature
        adaptive_factor.max(0.1)
    }

    /// Calculate proximity-based curvature reduction factor
    fn calculate_proximity_reduction(
        &self,
        p1: Point2D,
        p2: Point2D,
        box_dims: (f64, f64),
        total_branches: usize,
        neighbor_info: Option<&[f64]>,
    ) -> f64 {
        // If we don't have neighbor information, use branch density estimation
        let neighbor_distances = match neighbor_info {
            Some(distances) => distances,
            None => return self.estimate_density_based_reduction(p1, p2, box_dims, total_branches),
        };
        let mut max_reduction: f64 = 0.0;

        // Calculate channel midpoint for proximity calculations
        let _mid_x = (p1.0 + p2.0) / 2.0;
        let _mid_y = (p1.1 + p2.1) / 2.0;

        // Check proximity to each neighbor
        for &neighbor_distance in neighbor_distances {
            if neighbor_distance < self.config.min_separation_distance {
                // Calculate reduction factor based on how close the neighbor is
                let proximity_ratio = neighbor_distance / self.config.min_separation_distance;
                let reduction = (1.0 - proximity_ratio).max(0.0);
                max_reduction = max_reduction.max(reduction);
            }
        }

        // Apply maximum reduction limit
        max_reduction.min(1.0 - self.config.max_curvature_reduction)
    }

    /// Estimate curvature reduction based on branch density
    fn estimate_density_based_reduction(
        &self,
        p1: Point2D,
        p2: Point2D,
        box_dims: (f64, f64),
        total_branches: usize,
    ) -> f64 {
        // Calculate effective area per branch
        let box_area = box_dims.0 * box_dims.1;
        let area_per_branch = box_area / total_branches as f64;

        // Calculate channel length
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let channel_length = (dx * dx + dy * dy).sqrt();

        // Estimate potential arc area
        let potential_arc_area = channel_length * channel_length * self.config.curvature_factor;

        // If potential arc area is large relative to available space, reduce curvature
        if potential_arc_area > area_per_branch * 0.5 {
            let density_ratio = potential_arc_area / (area_per_branch * 0.5);
            let reduction = (density_ratio - 1.0).max(0.0).min(0.8);
            return reduction;
        }

        0.0 // No reduction needed
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
