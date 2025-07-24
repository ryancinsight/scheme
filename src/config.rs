//! config.rs - 2D Schematic Configuration
//!
//! This module provides comprehensive configuration management for 2D microfluidic
//! schematic generation. It centralizes all configuration logic and eliminates
//! Single Source of Truth (SSOT) violations by providing validated configuration
//! types with clear constraints and relationships.
//!
//! # Design Principles
//!
//! - **Single Source of Truth**: All configuration values are defined once
//! - **Validation**: All configurations are validated at creation time
//! - **Immutability**: Configurations are immutable after creation
//! - **Composability**: Complex configurations are built from simpler ones
//! - **Discoverability**: Presets and builders make common configurations easy

use crate::geometry::ChannelType;
use crate::error::{ConfigurationError, ConfigurationResult};

/// Configuration constants for geometry validation and defaults
pub mod constants {
    /// Minimum allowed wall clearance (mm)
    pub const MIN_WALL_CLEARANCE: f64 = 0.1;
    /// Maximum allowed wall clearance (mm)
    pub const MAX_WALL_CLEARANCE: f64 = 100.0;
    /// Default wall clearance (mm)
    pub const DEFAULT_WALL_CLEARANCE: f64 = 0.5;

    /// Minimum allowed channel width (mm)
    pub const MIN_CHANNEL_WIDTH: f64 = 0.01;
    /// Maximum allowed channel width (mm)
    pub const MAX_CHANNEL_WIDTH: f64 = 1000.0;
    /// Default channel width (mm)
    pub const DEFAULT_CHANNEL_WIDTH: f64 = 1.0;

    /// Minimum allowed channel height (mm)
    pub const MIN_CHANNEL_HEIGHT: f64 = 0.01;
    /// Maximum allowed channel height (mm)
    pub const MAX_CHANNEL_HEIGHT: f64 = 1000.0;
    /// Default channel height (mm)
    pub const DEFAULT_CHANNEL_HEIGHT: f64 = 0.5;

    /// Minimum fill factor for serpentine channels
    pub const MIN_FILL_FACTOR: f64 = 0.1;
    /// Maximum fill factor for serpentine channels
    pub const MAX_FILL_FACTOR: f64 = 0.95;
    /// Default fill factor for serpentine channels
    pub const DEFAULT_FILL_FACTOR: f64 = 0.8;

    /// Minimum wavelength factor for serpentine channels
    pub const MIN_WAVELENGTH_FACTOR: f64 = 1.0;
    /// Maximum wavelength factor for serpentine channels
    pub const MAX_WAVELENGTH_FACTOR: f64 = 10.0;
    /// Default wavelength factor for serpentine channels
    pub const DEFAULT_WAVELENGTH_FACTOR: f64 = 3.0;

    /// Minimum Gaussian width factor for serpentine channels
    pub const MIN_GAUSSIAN_WIDTH_FACTOR: f64 = 2.0;
    /// Maximum Gaussian width factor for serpentine channels
    pub const MAX_GAUSSIAN_WIDTH_FACTOR: f64 = 20.0;
    /// Default Gaussian width factor for serpentine channels
    pub const DEFAULT_GAUSSIAN_WIDTH_FACTOR: f64 = 6.0;

    /// Minimum wave density factor for serpentine channels
    pub const MIN_WAVE_DENSITY_FACTOR: f64 = 0.5;
    /// Maximum wave density factor for serpentine channels
    pub const MAX_WAVE_DENSITY_FACTOR: f64 = 10.0;
    /// Default wave density factor for serpentine channels
    pub const DEFAULT_WAVE_DENSITY_FACTOR: f64 = 2.5;

    /// Minimum curvature factor for arc channels
    pub const MIN_CURVATURE_FACTOR: f64 = 0.0;
    /// Maximum curvature factor for arc channels
    pub const MAX_CURVATURE_FACTOR: f64 = 2.0;
    /// Default curvature factor for arc channels
    pub const DEFAULT_CURVATURE_FACTOR: f64 = 0.3;

    /// Minimum smoothness for arc channels
    pub const MIN_SMOOTHNESS: usize = 3;
    /// Maximum smoothness for arc channels
    pub const MAX_SMOOTHNESS: usize = 1000;
    /// Default smoothness for arc channels
    pub const DEFAULT_SMOOTHNESS: usize = 20;

    /// Strategy thresholds for smart channel type selection
    pub mod strategy_thresholds {
        /// Threshold for long horizontal channels (fraction of box width)
        pub const LONG_HORIZONTAL_THRESHOLD: f64 = 0.3;
        /// Threshold for minimum arc length (fraction of box width)
        pub const MIN_ARC_LENGTH_THRESHOLD: f64 = 0.1;
        /// Threshold for horizontal vs angled channel detection
        pub const HORIZONTAL_ANGLE_THRESHOLD: f64 = 0.3;
        /// Threshold for angled channel detection (slope)
        pub const ANGLED_CHANNEL_SLOPE_THRESHOLD: f64 = 0.1;
        /// Default middle zone fraction for mixed by position
        pub const DEFAULT_MIDDLE_ZONE_FRACTION: f64 = 0.4;
    }
}

/// Configuration for basic geometry parameters
#[derive(Clone, Copy, Debug)]
pub struct GeometryConfig {
    pub wall_clearance: f64,
    pub channel_width: f64,
    pub channel_height: f64,
}

impl GeometryConfig {
    /// Create a new geometry configuration with validation
    pub fn new(wall_clearance: f64, channel_width: f64, channel_height: f64) -> ConfigurationResult<Self> {
        let config = Self {
            wall_clearance,
            channel_width,
            channel_height,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate the geometry configuration
    pub fn validate(&self) -> ConfigurationResult<()> {
        if self.wall_clearance < constants::MIN_WALL_CLEARANCE || self.wall_clearance > constants::MAX_WALL_CLEARANCE {
            return Err(ConfigurationError::invalid_geometry_config(
                "wall_clearance",
                self.wall_clearance,
                &format!("Must be between {} and {}", constants::MIN_WALL_CLEARANCE, constants::MAX_WALL_CLEARANCE)
            ));
        }

        if self.channel_width < constants::MIN_CHANNEL_WIDTH || self.channel_width > constants::MAX_CHANNEL_WIDTH {
            return Err(ConfigurationError::invalid_geometry_config(
                "channel_width",
                self.channel_width,
                &format!("Must be between {} and {}", constants::MIN_CHANNEL_WIDTH, constants::MAX_CHANNEL_WIDTH)
            ));
        }

        if self.channel_height < constants::MIN_CHANNEL_HEIGHT || self.channel_height > constants::MAX_CHANNEL_HEIGHT {
            return Err(ConfigurationError::invalid_geometry_config(
                "channel_height",
                self.channel_height,
                &format!("Must be between {} and {}", constants::MIN_CHANNEL_HEIGHT, constants::MAX_CHANNEL_HEIGHT)
            ));
        }

        Ok(())
    }
}

impl Default for GeometryConfig {
    fn default() -> Self {
        Self {
            wall_clearance: constants::DEFAULT_WALL_CLEARANCE,
            channel_width: constants::DEFAULT_CHANNEL_WIDTH,
            channel_height: constants::DEFAULT_CHANNEL_HEIGHT,
        }
    }
}

/// Optimization profile for serpentine channel optimization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationProfile {
    /// Fast optimization with limited parameter exploration (5-10x slower)
    Fast,
    /// Balanced optimization with moderate exploration (20-50x slower)
    Balanced,
    /// Thorough optimization with extensive exploration (100-500x slower)
    Thorough,
}

/// Configuration for serpentine (S-shaped) channels
#[derive(Debug, Clone, Copy)]
pub struct SerpentineConfig {
    /// Fraction of available vertical space to fill (0.1 to 0.95)
    pub fill_factor: f64,
    /// Multiplier for channel width to determine wavelength (1.0 to 10.0)
    pub wavelength_factor: f64,
    /// Controls width of Gaussian envelope - sigma = length / gaussian_width_factor (2.0 to 20.0)
    pub gaussian_width_factor: f64,
    /// Controls wave density relative to channel length - higher = more waves (0.5 to 10.0)
    pub wave_density_factor: f64,
    /// Controls wave phase direction for symmetry: -1.0=force inward, 1.0=force outward, 0.0=auto-symmetric
    pub wave_phase_direction: f64,
    /// Enable length optimization algorithm (default: false for backward compatibility)
    pub optimization_enabled: bool,
    /// Target fill ratio for optimization - fraction of maximum possible length to achieve (0.8 to 0.99)
    pub target_fill_ratio: f64,
    /// Optimization profile controlling speed vs quality tradeoff
    pub optimization_profile: OptimizationProfile,
}

impl SerpentineConfig {
    /// Create a new serpentine configuration with validation
    pub fn new(
        fill_factor: f64,
        wavelength_factor: f64,
        gaussian_width_factor: f64,
        wave_density_factor: f64,
    ) -> ConfigurationResult<Self> {
        let config = Self {
            fill_factor,
            wavelength_factor,
            gaussian_width_factor,
            wave_density_factor,
            wave_phase_direction: 0.0, // Auto-determine for perfect symmetry
            optimization_enabled: false, // Disabled by default for backward compatibility
            target_fill_ratio: 0.9, // Default target for optimization
            optimization_profile: OptimizationProfile::Balanced, // Default profile
        };
        config.validate()?;
        Ok(config)
    }

    /// Create a new serpentine configuration with optimization enabled
    pub fn new_with_optimization(
        fill_factor: f64,
        wavelength_factor: f64,
        gaussian_width_factor: f64,
        wave_density_factor: f64,
        target_fill_ratio: f64,
    ) -> ConfigurationResult<Self> {
        let config = Self {
            fill_factor,
            wavelength_factor,
            gaussian_width_factor,
            wave_density_factor,
            wave_phase_direction: 0.0, // Auto-determine for perfect symmetry
            optimization_enabled: true,
            target_fill_ratio,
            optimization_profile: OptimizationProfile::Balanced, // Default profile
        };
        config.validate()?;
        Ok(config)
    }

    /// Create a new serpentine configuration with optimization and profile
    pub fn new_with_optimization_profile(
        fill_factor: f64,
        wavelength_factor: f64,
        gaussian_width_factor: f64,
        wave_density_factor: f64,
        target_fill_ratio: f64,
        optimization_profile: OptimizationProfile,
    ) -> ConfigurationResult<Self> {
        let config = Self {
            fill_factor,
            wavelength_factor,
            gaussian_width_factor,
            wave_density_factor,
            wave_phase_direction: 0.0, // Auto-determine for perfect symmetry
            optimization_enabled: true,
            target_fill_ratio,
            optimization_profile,
        };
        config.validate()?;
        Ok(config)
    }

    /// Create a new serpentine configuration with explicit wave phase direction control
    pub fn new_with_phase_direction(
        fill_factor: f64,
        wavelength_factor: f64,
        gaussian_width_factor: f64,
        wave_density_factor: f64,
        wave_phase_direction: f64,
    ) -> ConfigurationResult<Self> {
        let config = Self {
            fill_factor,
            wavelength_factor,
            gaussian_width_factor,
            wave_density_factor,
            wave_phase_direction,
            optimization_enabled: false, // Disabled by default for backward compatibility
            target_fill_ratio: 0.9, // Default target for optimization
            optimization_profile: OptimizationProfile::Balanced, // Default profile
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate the serpentine configuration
    pub fn validate(&self) -> ConfigurationResult<()> {
        if self.fill_factor < constants::MIN_FILL_FACTOR || self.fill_factor > constants::MAX_FILL_FACTOR {
            return Err(ConfigurationError::invalid_serpentine_config(
                "fill_factor",
                self.fill_factor,
                &format!("Must be between {} and {}", constants::MIN_FILL_FACTOR, constants::MAX_FILL_FACTOR)
            ));
        }

        if self.wavelength_factor < constants::MIN_WAVELENGTH_FACTOR || self.wavelength_factor > constants::MAX_WAVELENGTH_FACTOR {
            return Err(ConfigurationError::invalid_serpentine_config(
                "wavelength_factor",
                self.wavelength_factor,
                &format!("Must be between {} and {}", constants::MIN_WAVELENGTH_FACTOR, constants::MAX_WAVELENGTH_FACTOR)
            ));
        }

        if self.gaussian_width_factor < constants::MIN_GAUSSIAN_WIDTH_FACTOR || self.gaussian_width_factor > constants::MAX_GAUSSIAN_WIDTH_FACTOR {
            return Err(ConfigurationError::invalid_serpentine_config(
                "gaussian_width_factor",
                self.gaussian_width_factor,
                &format!("Must be between {} and {}", constants::MIN_GAUSSIAN_WIDTH_FACTOR, constants::MAX_GAUSSIAN_WIDTH_FACTOR)
            ));
        }

        if self.wave_density_factor < constants::MIN_WAVE_DENSITY_FACTOR || self.wave_density_factor > constants::MAX_WAVE_DENSITY_FACTOR {
            return Err(ConfigurationError::invalid_serpentine_config(
                "wave_density_factor",
                self.wave_density_factor,
                &format!("Must be between {} and {}", constants::MIN_WAVE_DENSITY_FACTOR, constants::MAX_WAVE_DENSITY_FACTOR)
            ));
        }

        if self.wave_phase_direction.abs() > 1.0 {
            return Err(ConfigurationError::invalid_serpentine_config(
                "wave_phase_direction",
                self.wave_phase_direction,
                "Must be between -1.0 and 1.0"
            ));
        }

        if self.target_fill_ratio < 0.8 || self.target_fill_ratio > 0.99 {
            return Err(ConfigurationError::invalid_serpentine_config(
                "target_fill_ratio",
                self.target_fill_ratio,
                "Must be between 0.8 and 0.99"
            ));
        }

        Ok(())
    }
}

/// Configuration for arc (curved) channels
#[derive(Debug, Clone, Copy)]
pub struct ArcConfig {
    /// Controls how curved the arc is - 0.0 = straight, 1.0 = semicircle (0.0 to 2.0)
    pub curvature_factor: f64,
    /// Number of points to generate along the arc - higher = smoother (3 to 1000)
    pub smoothness: usize,
    /// Controls the direction of arc curvature: 1.0 = outward, -1.0 = inward, 0.0 = auto
    pub curvature_direction: f64,
}

impl ArcConfig {
    /// Create a new arc configuration with validation
    pub fn new(curvature_factor: f64, smoothness: usize) -> ConfigurationResult<Self> {
        let config = Self {
            curvature_factor,
            smoothness,
            curvature_direction: 0.0, // Auto-determine direction
        };
        config.validate()?;
        Ok(config)
    }

    /// Create a new arc configuration with explicit curvature direction
    pub fn new_with_direction(curvature_factor: f64, smoothness: usize, curvature_direction: f64) -> ConfigurationResult<Self> {
        let config = Self {
            curvature_factor,
            smoothness,
            curvature_direction,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate the arc configuration
    pub fn validate(&self) -> ConfigurationResult<()> {
        if self.curvature_factor < constants::MIN_CURVATURE_FACTOR || self.curvature_factor > constants::MAX_CURVATURE_FACTOR {
            return Err(ConfigurationError::invalid_arc_config(
                "curvature_factor",
                self.curvature_factor,
                &format!("Must be between {} and {}", constants::MIN_CURVATURE_FACTOR, constants::MAX_CURVATURE_FACTOR)
            ));
        }

        if self.smoothness < constants::MIN_SMOOTHNESS || self.smoothness > constants::MAX_SMOOTHNESS {
            return Err(ConfigurationError::InvalidArcConfig {
                field: "smoothness".to_string(),
                value: self.smoothness as f64,
                constraint: format!("Must be between {} and {}", constants::MIN_SMOOTHNESS, constants::MAX_SMOOTHNESS),
            });
        }

        if self.curvature_direction.abs() > 1.0 {
            return Err(ConfigurationError::InvalidArcConfig {
                field: "curvature_direction".to_string(),
                value: self.curvature_direction,
                constraint: "Must be between -1.0 and 1.0".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for SerpentineConfig {
    fn default() -> Self {
        Self {
            fill_factor: constants::DEFAULT_FILL_FACTOR,
            wavelength_factor: constants::DEFAULT_WAVELENGTH_FACTOR,
            gaussian_width_factor: constants::DEFAULT_GAUSSIAN_WIDTH_FACTOR,
            wave_density_factor: constants::DEFAULT_WAVE_DENSITY_FACTOR,
            wave_phase_direction: 0.0, // Auto-determine for perfect symmetry
            optimization_enabled: false, // Disabled by default for backward compatibility
            target_fill_ratio: 0.9, // Default target for optimization
            optimization_profile: OptimizationProfile::Balanced, // Default profile
        }
    }
}

impl Default for ArcConfig {
    fn default() -> Self {
        Self {
            curvature_factor: constants::DEFAULT_CURVATURE_FACTOR,
            smoothness: constants::DEFAULT_SMOOTHNESS,
            curvature_direction: 0.0, // Auto-determine direction
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChannelTypeConfig {
    AllStraight,
    AllSerpentine(SerpentineConfig),
    AllArcs(ArcConfig),
    MixedByPosition { 
        middle_zone_fraction: f64, 
        serpentine_config: SerpentineConfig,
        arc_config: ArcConfig,
    },
    Smart { 
        serpentine_config: SerpentineConfig,
        arc_config: ArcConfig,
    },
    Custom(fn(from: (f64, f64), to: (f64, f64), box_dims: (f64, f64)) -> ChannelType),
}

impl Default for ChannelTypeConfig {
    fn default() -> Self {
        ChannelTypeConfig::Smart {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
        }
    }
}

/// Configuration presets for common use cases
pub mod presets {
    use super::*;

    /// Preset for microfluidic devices with fine features
    pub fn fine_features() -> GeometryConfig {
        GeometryConfig {
            wall_clearance: 0.2,
            channel_width: 0.5,
            channel_height: 0.3,
        }
    }

    /// Preset for standard microfluidic devices
    pub fn standard() -> GeometryConfig {
        GeometryConfig::default()
    }

    /// Preset for large-scale microfluidic devices
    pub fn large_scale() -> GeometryConfig {
        GeometryConfig {
            wall_clearance: 2.0,
            channel_width: 5.0,
            channel_height: 2.0,
        }
    }

    /// Preset for high-density serpentine channels
    pub fn high_density_serpentine() -> SerpentineConfig {
        SerpentineConfig {
            fill_factor: 0.9,
            wavelength_factor: 2.0,
            gaussian_width_factor: 8.0,
            wave_density_factor: 4.0,
            wave_phase_direction: 0.0, // Auto-symmetric
            optimization_enabled: false,
            target_fill_ratio: 0.9,
            optimization_profile: OptimizationProfile::Balanced,
        }
    }

    /// Preset for smooth serpentine channels
    pub fn smooth_serpentine() -> SerpentineConfig {
        SerpentineConfig {
            fill_factor: 0.6,
            wavelength_factor: 4.0,
            gaussian_width_factor: 10.0,
            wave_density_factor: 1.5,
            wave_phase_direction: 0.0, // Auto-symmetric
            optimization_enabled: false,
            target_fill_ratio: 0.9,
            optimization_profile: OptimizationProfile::Balanced,
        }
    }

    /// Preset for inward-phase serpentine channels (all waves start inward)
    pub fn inward_serpentines() -> SerpentineConfig {
        SerpentineConfig {
            fill_factor: 0.8,
            wavelength_factor: 3.0,
            gaussian_width_factor: 6.0,
            wave_density_factor: 2.0,
            wave_phase_direction: -1.0, // Force inward phase
            optimization_enabled: false,
            target_fill_ratio: 0.9,
            optimization_profile: OptimizationProfile::Balanced,
        }
    }

    /// Preset for outward-phase serpentine channels (all waves start outward)
    pub fn outward_serpentines() -> SerpentineConfig {
        SerpentineConfig {
            fill_factor: 0.8,
            wavelength_factor: 3.0,
            gaussian_width_factor: 6.0,
            wave_density_factor: 2.0,
            wave_phase_direction: 1.0, // Force outward phase
            optimization_enabled: false,
            target_fill_ratio: 0.9,
            optimization_profile: OptimizationProfile::Balanced,
        }
    }

    /// Preset for length-optimized serpentine channels
    pub fn optimized_serpentine() -> SerpentineConfig {
        SerpentineConfig {
            fill_factor: 0.8,
            wavelength_factor: 3.0,
            gaussian_width_factor: 6.0,
            wave_density_factor: 2.0,
            wave_phase_direction: 0.0, // Auto-symmetric
            optimization_enabled: true,
            target_fill_ratio: 0.95, // Aggressive optimization target
            optimization_profile: OptimizationProfile::Balanced,
        }
    }

    /// Preset for fast-optimized serpentine channels
    pub fn fast_optimized_serpentine() -> SerpentineConfig {
        SerpentineConfig {
            fill_factor: 0.8,
            wavelength_factor: 3.0,
            gaussian_width_factor: 6.0,
            wave_density_factor: 2.0,
            wave_phase_direction: 0.0, // Auto-symmetric
            optimization_enabled: true,
            target_fill_ratio: 0.9, // Moderate optimization target
            optimization_profile: OptimizationProfile::Fast,
        }
    }

    /// Preset for thorough-optimized serpentine channels
    pub fn thorough_optimized_serpentine() -> SerpentineConfig {
        SerpentineConfig {
            fill_factor: 0.8,
            wavelength_factor: 3.0,
            gaussian_width_factor: 6.0,
            wave_density_factor: 2.0,
            wave_phase_direction: 0.0, // Auto-symmetric
            optimization_enabled: true,
            target_fill_ratio: 0.98, // Very aggressive optimization target
            optimization_profile: OptimizationProfile::Thorough,
        }
    }

    /// Preset for subtle arc channels
    pub fn subtle_arcs() -> ArcConfig {
        ArcConfig {
            curvature_factor: 0.2,
            smoothness: 30,
            curvature_direction: 0.0, // Auto-determine
        }
    }

    /// Preset for pronounced arc channels
    pub fn pronounced_arcs() -> ArcConfig {
        ArcConfig {
            curvature_factor: 0.8,
            smoothness: 50,
            curvature_direction: 0.0, // Auto-determine
        }
    }

    /// Preset for inward-curving arc channels (all arcs curve toward center)
    pub fn inward_arcs() -> ArcConfig {
        ArcConfig {
            curvature_factor: 0.5,
            smoothness: 30,
            curvature_direction: -1.0, // Force inward curvature
        }
    }

    /// Preset for outward-curving arc channels (all arcs curve away from center)
    pub fn outward_arcs() -> ArcConfig {
        ArcConfig {
            curvature_factor: 0.5,
            smoothness: 30,
            curvature_direction: 1.0, // Force outward curvature
        }
    }
}

/// Builder for complex channel type configurations
pub struct ChannelTypeConfigBuilder {
    serpentine_config: SerpentineConfig,
    arc_config: ArcConfig,
    middle_zone_fraction: f64,
}

impl ChannelTypeConfigBuilder {
    /// Create a new builder with default configurations
    pub fn new() -> Self {
        Self {
            serpentine_config: SerpentineConfig::default(),
            arc_config: ArcConfig::default(),
            middle_zone_fraction: constants::strategy_thresholds::DEFAULT_MIDDLE_ZONE_FRACTION,
        }
    }

    /// Set the serpentine configuration
    pub fn with_serpentine_config(mut self, config: SerpentineConfig) -> Self {
        self.serpentine_config = config;
        self
    }

    /// Set the arc configuration
    pub fn with_arc_config(mut self, config: ArcConfig) -> Self {
        self.arc_config = config;
        self
    }

    /// Set the middle zone fraction for mixed by position
    pub fn with_middle_zone_fraction(mut self, fraction: f64) -> Self {
        self.middle_zone_fraction = fraction;
        self
    }

    /// Build a smart channel type configuration
    pub fn build_smart(self) -> ChannelTypeConfig {
        ChannelTypeConfig::Smart {
            serpentine_config: self.serpentine_config,
            arc_config: self.arc_config,
        }
    }

    /// Build a mixed by position channel type configuration
    pub fn build_mixed_by_position(self) -> ChannelTypeConfig {
        ChannelTypeConfig::MixedByPosition {
            middle_zone_fraction: self.middle_zone_fraction,
            serpentine_config: self.serpentine_config,
            arc_config: self.arc_config,
        }
    }
}

impl Default for ChannelTypeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}