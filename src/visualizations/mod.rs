//! visualizations/mod.rs - 2D Schematic Visualization
//!
//! This module provides visualization capabilities for 2D microfluidic schematics,
//! including plotting of channel layouts, bifurcation patterns, and trifurcation designs.
//!
//! # Architecture
//!
//! The visualization module follows the Dependency Inversion Principle by defining
//! abstract traits for rendering operations and providing concrete implementations
//! for specific backends (like plotters). This allows for easy extension with
//! new rendering backends without changing the core visualization logic.
//!
//! # Modules
//!
//! - `traits`: Abstract interfaces for visualization operations
//! - `plotters_backend`: Concrete implementation using the plotters library
//! - `schematic`: High-level schematic rendering functions
//! - `shared_utilities`: Common utilities for visualization operations

/// High-level schematic rendering functions
pub mod schematic;
/// Shared utilities for visualization operations
pub mod shared_utilities;
pub mod traits;
pub mod plotters_backend;

pub use schematic::plot_geometry;
pub use traits::{SchematicRenderer, RenderConfig, OutputFormat, Color, LineStyle, TextStyle, ChannelTypeStyles};
pub use plotters_backend::{PlottersRenderer, create_plotters_renderer, plot_geometry_with_plotters};