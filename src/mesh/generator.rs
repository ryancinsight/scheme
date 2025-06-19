//! src/mesh/generator.rs

use crate::geometry::mod_3d::ChannelSystem3D;
use crate::mesh::primitives::{cuboid, cylinder};
use stl_io::Triangle;

/// Converts a 3D channel system into a list of triangles for STL export.
pub fn generate_mesh_from_system(system: &ChannelSystem3D) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    if system.has_drawable_box() {
        triangles.extend(cuboid::generate(&system.box_volume));
    }
    for cylinder_geom in &system.cylinders {
        triangles.extend(cylinder::generate(cylinder_geom));
    }
    triangles
} 