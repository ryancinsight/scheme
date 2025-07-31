//! Scheme - 2D Microfluidic Schematic Design Library
//!
//! A focused library for designing 2D microfluidic schematics with support for
//! bifurcation and trifurcation patterns, channel layout algorithms, and
//! schematic visualization.
//!
//! # Architecture
//!
//! Scheme follows modern software design principles including SOLID, CUPID, and GRASP.
//! The library is organized into several key modules:
//!
//! - **geometry**: Core geometric types and generation logic
//!   - `types`: Fundamental data structures (Point2D, Node, Channel, etc.)
//!   - `strategies`: Channel type generation strategies (Strategy pattern)
//!   - `generator`: Main geometry generation orchestration
//! - **config**: Configuration types for geometry and channel generation
//! - **visualizations**: 2D schematic rendering and export
//! - **error**: Domain-specific error types
//!
//! # Design Patterns
//!
//! The library implements several design patterns for maintainability and extensibility:
//!
//! - **Strategy Pattern**: Channel type generation is handled by pluggable strategies,
//!   making it easy to add new channel types without modifying existing code.
//! - **Factory Pattern**: The `ChannelTypeFactory` creates appropriate strategies
//!   based on configuration.
//! - **Builder Pattern**: Complex geometries are built incrementally through the
//!   geometry generator.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use scheme::{
//!     geometry::generator::create_geometry,
//!     config::{GeometryConfig, ChannelTypeConfig},
//!     geometry::SplitType
//! };
//!
//! let config = GeometryConfig::default();
//! let system = create_geometry(
//!     (200.0, 100.0),  // box dimensions
//!     &[SplitType::Bifurcation],  // split pattern
//!     &config,
//!     &ChannelTypeConfig::AllStraight,  // channel type configuration
//! );
//! ```
//!
//! ## Advanced Channel Types
//!
//! ```rust
//! use scheme::{
//!     geometry::generator::create_geometry,
//!     config::{GeometryConfig, ChannelTypeConfig, SerpentineConfig},
//!     geometry::SplitType
//! };
//!
//! let serpentine_config = SerpentineConfig {
//!     fill_factor: 0.8,
//!     wavelength_factor: 3.0,
//!     gaussian_width_factor: 6.0,
//!     wave_density_factor: 2.0,
//!     wave_phase_direction: 0.0, // Auto-symmetric
//!     optimization_enabled: false,
//!     target_fill_ratio: 0.9,
//!     optimization_profile: scheme::config::OptimizationProfile::Balanced,
//!     ..SerpentineConfig::default()
//! };
//!
//! let system = create_geometry(
//!     (300.0, 150.0),
//!     &[SplitType::Bifurcation, SplitType::Trifurcation],
//!     &GeometryConfig::default(),
//!     &ChannelTypeConfig::AllSerpentine(serpentine_config),
//! );
//! ```

pub mod geometry;
pub mod visualizations;
pub mod config;
pub mod config_constants;
pub mod error;
pub mod state_management;

pub use visualizations::schematic::plot_geometry;
pub use error::{SchemeError, SchemeResult, GeometryError, ConfigurationError, VisualizationError, StrategyError};
pub use state_management::{
    ParameterRegistry, ParameterManager, ConfigurableParameter, ParameterConstraints,
    StateManagementError, ParameterError, StateManagementResult, ConstraintError,
};