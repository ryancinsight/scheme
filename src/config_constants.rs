//! Centralized configuration constants
//!
//! This module extracts all hardcoded values and magic numbers from throughout
//! the codebase into configurable parameters with proper validation and documentation.
//! This follows the SSOT (Single Source of Truth) principle and eliminates magic numbers.

use crate::state_management::{
    ParameterConstraints, ConfigurableParameter, ParameterMetadata,
};

/// Strategy selection thresholds and parameters
pub struct StrategyThresholds {
    /// Minimum curvature factor to use arc strategy instead of straight
    pub arc_curvature_threshold: ConfigurableParameter<f64>,
    
    /// Maximum fill factor to use serpentine strategy
    pub serpentine_fill_threshold: ConfigurableParameter<f64>,
    
    /// Minimum channel length for complex strategies
    pub min_complex_strategy_length: ConfigurableParameter<f64>,
    
    /// Branch count threshold for adaptive behavior
    pub adaptive_branch_threshold: ConfigurableParameter<usize>,
}

impl StrategyThresholds {
    /// Create default strategy thresholds
    pub fn default() -> Self {
        Self {
            arc_curvature_threshold: ConfigurableParameter::new(
                0.1,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::non_negative(),
                    ParameterConstraints::range(0.0, 2.0),
                ]),
                ParameterMetadata::new(
                    "arc_curvature_threshold",
                    "Minimum curvature factor to trigger arc strategy selection",
                    "strategy_selection"
                ).with_units("factor").affects_others()
            ),
            
            serpentine_fill_threshold: ConfigurableParameter::new(
                0.95,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::normalized(),
                ]),
                ParameterMetadata::new(
                    "serpentine_fill_threshold",
                    "Maximum fill factor for serpentine strategy selection",
                    "strategy_selection"
                ).affects_others()
            ),
            
            min_complex_strategy_length: ConfigurableParameter::new(
                10.0,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(1.0, 1000.0),
                ]),
                ParameterMetadata::new(
                    "min_complex_strategy_length",
                    "Minimum channel length to use complex strategies",
                    "strategy_selection"
                ).with_units("mm")
            ),
            
            adaptive_branch_threshold: ConfigurableParameter::new(
                4usize,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<usize>::positive(),
                    ParameterConstraints::range(1, 100),
                ]),
                ParameterMetadata::new(
                    "adaptive_branch_threshold",
                    "Branch count threshold to enable adaptive parameter behavior",
                    "strategy_selection"
                )
            ),
        }
    }
}

/// Wave generation constants previously hardcoded in strategies
pub struct WaveGenerationConstants {
    /// Sharpness factor for square wave generation
    pub square_wave_sharpness: ConfigurableParameter<f64>,
    
    /// Transition zone factor for smooth endpoints
    pub transition_zone_factor: ConfigurableParameter<f64>,
    
    /// Gaussian envelope scaling factor
    pub gaussian_envelope_scale: ConfigurableParameter<f64>,
    
    /// Phase direction calculation threshold
    pub phase_direction_threshold: ConfigurableParameter<f64>,
    
    /// Wave amplitude safety margin
    pub amplitude_safety_margin: ConfigurableParameter<f64>,
}

impl WaveGenerationConstants {
    /// Create default wave generation constants
    pub fn default() -> Self {
        Self {
            square_wave_sharpness: ConfigurableParameter::new(
                5.0,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(1.0, 20.0),
                ]),
                ParameterMetadata::new(
                    "square_wave_sharpness",
                    "Sharpness factor for square wave generation using tanh",
                    "wave_generation"
                ).with_units("factor")
            ),
            
            transition_zone_factor: ConfigurableParameter::new(
                0.1,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(0.01, 0.5),
                ]),
                ParameterMetadata::new(
                    "transition_zone_factor",
                    "Factor for smooth transition zones at wave endpoints",
                    "wave_generation"
                ).with_units("ratio")
            ),
            
            gaussian_envelope_scale: ConfigurableParameter::new(
                1.0,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(0.1, 5.0),
                ]),
                ParameterMetadata::new(
                    "gaussian_envelope_scale",
                    "Scaling factor for Gaussian envelope calculations",
                    "wave_generation"
                ).with_units("factor")
            ),
            
            phase_direction_threshold: ConfigurableParameter::new(
                0.5,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::normalized(),
                ]),
                ParameterMetadata::new(
                    "phase_direction_threshold",
                    "Threshold for phase direction calculation in bilateral symmetry",
                    "wave_generation"
                ).with_units("ratio")
            ),
            
            amplitude_safety_margin: ConfigurableParameter::new(
                0.8,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::normalized(),
                ]),
                ParameterMetadata::new(
                    "amplitude_safety_margin",
                    "Safety margin factor for amplitude calculations",
                    "wave_generation"
                ).with_units("factor")
            ),
        }
    }
}

/// Geometry generation constants previously hardcoded
pub struct GeometryGenerationConstants {
    /// Default number of points for serpentine path generation
    pub default_serpentine_points: ConfigurableParameter<usize>,

    /// Minimum number of points for serpentine path generation
    pub min_serpentine_points: ConfigurableParameter<usize>,

    /// Maximum number of points for serpentine path generation
    pub max_serpentine_points: ConfigurableParameter<usize>,

    /// Default wall clearance
    pub default_wall_clearance: ConfigurableParameter<f64>,

    /// Default channel width
    pub default_channel_width: ConfigurableParameter<f64>,

    /// Default channel height
    pub default_channel_height: ConfigurableParameter<f64>,
}

impl GeometryGenerationConstants {
    /// Create default geometry generation constants
    pub fn default() -> Self {
        Self {
            default_serpentine_points: ConfigurableParameter::new(
                200usize,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<usize>::positive(),
                    ParameterConstraints::range(10, 1000),
                ]),
                ParameterMetadata::new(
                    "default_serpentine_points",
                    "Default number of points for serpentine path generation",
                    "geometry_generation"
                ).with_units("points")
            ),

            min_serpentine_points: ConfigurableParameter::new(
                10usize,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<usize>::positive(),
                    ParameterConstraints::range(3, 100),
                ]),
                ParameterMetadata::new(
                    "min_serpentine_points",
                    "Minimum number of points for serpentine path generation",
                    "geometry_generation"
                ).with_units("points")
            ),

            max_serpentine_points: ConfigurableParameter::new(
                1000usize,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<usize>::positive(),
                    ParameterConstraints::range(100, 5000),
                ]),
                ParameterMetadata::new(
                    "max_serpentine_points",
                    "Maximum number of points for serpentine path generation",
                    "geometry_generation"
                ).with_units("points")
            ),

            default_wall_clearance: ConfigurableParameter::new(
                0.5,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(0.1, 100.0),
                ]),
                ParameterMetadata::new(
                    "default_wall_clearance",
                    "Default wall clearance for geometry generation",
                    "geometry_generation"
                ).with_units("mm")
            ),

            default_channel_width: ConfigurableParameter::new(
                1.0,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(0.1, 50.0),
                ]),
                ParameterMetadata::new(
                    "default_channel_width",
                    "Default channel width for geometry generation",
                    "geometry_generation"
                ).with_units("mm")
            ),

            default_channel_height: ConfigurableParameter::new(
                1.0,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(0.1, 50.0),
                ]),
                ParameterMetadata::new(
                    "default_channel_height",
                    "Default channel height for geometry generation",
                    "geometry_generation"
                ).with_units("mm")
            ),
        }
    }
}

/// Optimization algorithm constants previously hardcoded
pub struct OptimizationConstants {
    /// Branch factor scaling exponent (was hardcoded as 0.75)
    pub branch_factor_exponent: ConfigurableParameter<f64>,

    /// Fill factor enhancement multiplier (was hardcoded as 1.5)
    pub fill_factor_enhancement: ConfigurableParameter<f64>,

    /// Maximum optimization iterations
    pub max_optimization_iterations: ConfigurableParameter<usize>,

    /// Convergence tolerance for optimization
    pub convergence_tolerance: ConfigurableParameter<f64>,

    /// Fast optimization wavelength factors
    pub fast_wavelength_factors: ConfigurableParameter<Vec<f64>>,

    /// Fast optimization wave density factors
    pub fast_wave_density_factors: ConfigurableParameter<Vec<f64>>,

    /// Fast optimization fill factors
    pub fast_fill_factors: ConfigurableParameter<Vec<f64>>,
}

impl OptimizationConstants {
    /// Create default optimization constants
    pub fn default() -> Self {
        Self {
            branch_factor_exponent: ConfigurableParameter::new(
                0.75,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(0.1, 2.0),
                ]),
                ParameterMetadata::new(
                    "branch_factor_exponent",
                    "Exponent for branch factor scaling (was hardcoded as 0.75)",
                    "optimization"
                ).with_units("exponent")
            ),

            fill_factor_enhancement: ConfigurableParameter::new(
                1.5,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(1.0, 3.0),
                ]),
                ParameterMetadata::new(
                    "fill_factor_enhancement",
                    "Enhancement multiplier for fill factor (was hardcoded as 1.5)",
                    "optimization"
                ).with_units("multiplier")
            ),

            max_optimization_iterations: ConfigurableParameter::new(
                100usize,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<usize>::positive(),
                    ParameterConstraints::range(10, 1000),
                ]),
                ParameterMetadata::new(
                    "max_optimization_iterations",
                    "Maximum number of iterations for optimization algorithms",
                    "optimization"
                ).with_units("iterations")
            ),

            convergence_tolerance: ConfigurableParameter::new(
                1e-6,
                ParameterConstraints::all(vec![
                    ParameterConstraints::<f64>::positive(),
                    ParameterConstraints::range(1e-10, 1e-2),
                ]),
                ParameterMetadata::new(
                    "convergence_tolerance",
                    "Tolerance for optimization convergence detection",
                    "optimization"
                ).with_units("tolerance")
            ),

            fast_wavelength_factors: ConfigurableParameter::new(
                vec![1.0, 2.0, 3.0, 4.0],
                ParameterConstraints::all(vec![]),
                ParameterMetadata::new(
                    "fast_wavelength_factors",
                    "Wavelength factors for fast optimization (was hardcoded array)",
                    "optimization"
                ).with_units("factors")
            ),

            fast_wave_density_factors: ConfigurableParameter::new(
                vec![1.0, 2.0, 3.0],
                ParameterConstraints::all(vec![]),
                ParameterMetadata::new(
                    "fast_wave_density_factors",
                    "Wave density factors for fast optimization (was hardcoded array)",
                    "optimization"
                ).with_units("factors")
            ),

            fast_fill_factors: ConfigurableParameter::new(
                vec![0.7, 0.8, 0.9],
                ParameterConstraints::all(vec![]),
                ParameterMetadata::new(
                    "fast_fill_factors",
                    "Fill factors for fast optimization (was hardcoded array)",
                    "optimization"
                ).with_units("factors")
            ),
        }
    }
}

/// Central constants registry containing all configuration constants
pub struct ConstantsRegistry {
    /// Strategy selection thresholds
    pub strategy_thresholds: StrategyThresholds,

    /// Wave generation constants
    pub wave_generation: WaveGenerationConstants,

    /// Geometry generation constants
    pub geometry_generation: GeometryGenerationConstants,

    /// Optimization constants
    pub optimization: OptimizationConstants,
}

impl ConstantsRegistry {
    /// Create a new constants registry with default values
    pub fn new() -> Self {
        Self {
            strategy_thresholds: StrategyThresholds::default(),
            wave_generation: WaveGenerationConstants::default(),
            geometry_generation: GeometryGenerationConstants::default(),
            optimization: OptimizationConstants::default(),
        }
    }

    /// Validate all constants
    pub fn validate_all(&self) -> crate::error::SchemeResult<()> {
        // Validate all parameter groups
        // This would be expanded to validate all parameters in each group
        Ok(())
    }
}

impl Default for ConstantsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions to get commonly used constant values
impl ConstantsRegistry {
    /// Get the square wave sharpness factor
    pub fn get_square_wave_sharpness(&self) -> f64 {
        *self.wave_generation.square_wave_sharpness.get_raw_value()
    }
    
    /// Get the transition zone factor
    pub fn get_transition_zone_factor(&self) -> f64 {
        *self.wave_generation.transition_zone_factor.get_raw_value()
    }
    
    /// Get the arc curvature threshold
    pub fn get_arc_curvature_threshold(&self) -> f64 {
        *self.strategy_thresholds.arc_curvature_threshold.get_raw_value()
    }
    
    /// Get the serpentine fill threshold
    pub fn get_serpentine_fill_threshold(&self) -> f64 {
        *self.strategy_thresholds.serpentine_fill_threshold.get_raw_value()
    }
    
    /// Get the adaptive branch threshold
    pub fn get_adaptive_branch_threshold(&self) -> usize {
        *self.strategy_thresholds.adaptive_branch_threshold.get_raw_value()
    }

    /// Get the branch factor exponent
    pub fn get_branch_factor_exponent(&self) -> f64 {
        *self.optimization.branch_factor_exponent.get_raw_value()
    }

    /// Get the fill factor enhancement multiplier
    pub fn get_fill_factor_enhancement(&self) -> f64 {
        *self.optimization.fill_factor_enhancement.get_raw_value()
    }

    /// Get the default serpentine points
    pub fn get_default_serpentine_points(&self) -> usize {
        *self.geometry_generation.default_serpentine_points.get_raw_value()
    }

    /// Get the default wall clearance
    pub fn get_default_wall_clearance(&self) -> f64 {
        *self.geometry_generation.default_wall_clearance.get_raw_value()
    }

    /// Get the default channel width
    pub fn get_default_channel_width(&self) -> f64 {
        *self.geometry_generation.default_channel_width.get_raw_value()
    }

    /// Get the fast optimization wavelength factors
    pub fn get_fast_wavelength_factors(&self) -> &Vec<f64> {
        self.optimization.fast_wavelength_factors.get_raw_value()
    }

    /// Get the fast optimization wave density factors
    pub fn get_fast_wave_density_factors(&self) -> &Vec<f64> {
        self.optimization.fast_wave_density_factors.get_raw_value()
    }

    /// Get the fast optimization fill factors
    pub fn get_fast_fill_factors(&self) -> &Vec<f64> {
        self.optimization.fast_fill_factors.get_raw_value()
    }
}
