//! src/mesh/mod.rs

pub mod stl;
pub mod generator;
pub mod operations;
pub mod primitives;
pub mod csg;
pub mod csgrs_bridge;

pub use stl::write_stl;
pub use generator::generate_mesh_from_system;
pub use operations::{difference, union, intersection, subtract, xor, subtract_csgrs, union_csgrs, intersection_csgrs};
pub use operations::difference::difference_csgrs;
pub use csg::Csg;
pub use csgrs_bridge::{csgrs_difference, triangles_to_csg, csg_to_triangles};