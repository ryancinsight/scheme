//! Scheme - 2D Microfluidic Schematic Design Library
//!
//! A focused library for designing 2D microfluidic schematics with support for
//! bifurcation and trifurcation patterns, channel layout algorithms, and
//! schematic visualization.
//!
//! # Examples
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

pub mod geometry;
pub mod visualizations;
pub mod config;
pub mod error;

pub use visualizations::schematic::plot_geometry;