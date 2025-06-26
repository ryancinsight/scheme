//! geometry/mod.rs - 2D Microfluidic Schematic Geometry
//!
//! This module provides 2D geometry types and generation functions for
//! microfluidic schematic design, including bifurcation and trifurcation patterns.

pub mod generator;
pub mod mod_2d;

pub use self::{
    generator::create_geometry,
    mod_2d::{Channel, ChannelSystem, Node, Point2D, SplitType},
};

pub type Point = Point2D;