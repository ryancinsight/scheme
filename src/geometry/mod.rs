//! geometry/mod.rs - 2D Microfluidic Schematic Geometry
//!
//! This module provides 2D geometry types and generation functions for
//! microfluidic schematic design, including bifurcation and trifurcation patterns.
//!
//! # Architecture
//!
//! The geometry module is organized into several submodules:
//! - `types`: Core geometric types and data structures
//! - `strategies`: Channel type generation strategies (Strategy pattern)
//! - `generator`: Main geometry generation logic
//!
//! # Design Patterns
//!
//! This module implements several design patterns:
//! - **Strategy Pattern**: For channel type generation (`strategies` module)
//! - **Factory Pattern**: For creating appropriate strategies
//! - **Builder Pattern**: For constructing complex geometries

pub mod builders;
pub mod enhanced_generator;
pub mod generator;
pub mod metadata;
pub mod optimization;
pub mod strategies;
pub mod types;

pub use self::{
    generator::create_geometry,
    types::{Channel, ChannelSystem, ChannelType, Node, Point2D, SplitType},
};

pub type Point = Point2D;