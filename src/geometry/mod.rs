//! geometry/mod.rs

pub mod generator;
pub mod mod_3d;
pub mod mod_2d;
pub mod converter;

pub use self::{
    converter::convert_2d_to_3d,
    generator::create_geometry,
    mod_2d::{CfdResults, Channel, ChannelSystem, Node, Point2D, SplitType},
    mod_3d::{ChannelSystem3D, Cylinder, Point3D, Volume},
};

pub type Point = Point2D;