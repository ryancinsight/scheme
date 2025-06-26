//! visualizations/mod.rs - 2D Schematic Visualization
//!
//! This module provides visualization capabilities for 2D microfluidic schematics,
//! including plotting of channel layouts, bifurcation patterns, and trifurcation designs.

pub mod schematic;
pub mod shared_utilities;

pub use schematic::plot_geometry;