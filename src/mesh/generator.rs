//! src/mesh/generator.rs

use crate::geometry::mod_3d::ChannelSystem3D;
use crate::mesh::primitives::{generate_cuboid, generate_cylinder, generate_sphere};
use stl_io::Triangle;

/// Generates a unified mesh from all components of a `ChannelSystem3D`.
pub fn generate_mesh_from_system(system: &ChannelSystem3D) -> Vec<Triangle> {
    let mut triangles = Vec::new();

    // Generate and add the box mesh if it's drawable.
    if system.has_drawable_box() {
        triangles.extend(generate_cuboid(&system.box_volume));
    }

    // Generate and add meshes for all cylinders.
    for cylinder in &system.cylinders {
        triangles.extend(generate_cylinder(cylinder));
    }

    // Generate and add meshes for all spheres.
    for sphere in &system.spheres {
        triangles.extend(generate_sphere(sphere, 20, 20));
    }

    triangles
} 