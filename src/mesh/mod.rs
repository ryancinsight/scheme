//! src/mesh/mod.rs

pub mod stl;
pub mod generator;
pub mod csg;
mod primitives;

pub use stl::write_stl;
pub use generator::generate_mesh_from_system;
pub use csg::subtract_cylinder_from_volume; 