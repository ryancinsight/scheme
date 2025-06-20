//! src/mesh/mod.rs

pub mod stl;
pub mod generator;
pub mod operations;
pub mod primitives;
mod csg;

pub use stl::write_stl;
pub use generator::generate_mesh_from_system;
pub use operations::{difference, union, intersection};
pub use csg::Csg; 