//! src/mesh/csgrs_bridge.rs
//!
//! Bridge module for integrating workspace csgrs with pyvismil
//!
//! This module provides conversion functions between pyvismil's Triangle-based
//! mesh representation and csgrs's CSG objects, enabling the use of csgrs's
//! robust CSG operations within pyvismil's architecture.

use csgrs::csg::CSG;
use stl_io::{Triangle, Vector};
use nalgebra::Point3;
use std::collections::HashMap;

/// Type alias for csgrs CSG with no metadata
type CsgrsCSG = CSG<()>;

/// Convert a collection of STL triangles to a csgrs CSG object
/// 
/// This function takes pyvismil's Triangle mesh format and converts it
/// to a csgrs CSG object that can be used for boolean operations.
/// 
/// # Arguments
/// * `triangles` - Collection of STL triangles representing the mesh
/// 
/// # Returns
/// * `Result<CsgrsCSG, &'static str>` - The resulting CSG object or error
pub fn triangles_to_csg(triangles: &[Triangle]) -> Result<CsgrsCSG, &'static str> {
    if triangles.is_empty() {
        return Err("Cannot create CSG from empty triangle collection");
    }

    // Convert STL triangles to csgrs polygons
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_map = HashMap::new();
    let mut next_index = 0u32;

    for triangle in triangles {
        let mut triangle_indices = [0u32; 3];
        
        for (i, vertex) in triangle.vertices.iter().enumerate() {
            let point = Point3::new(vertex[0], vertex[1], vertex[2]);
            
            // Use a simple vertex deduplication based on position
            let key = (
                (vertex[0] * 1000000.0) as i64,
                (vertex[1] * 1000000.0) as i64,
                (vertex[2] * 1000000.0) as i64,
            );
            
            let vertex_index = if let Some(&existing_index) = vertex_map.get(&key) {
                existing_index
            } else {
                vertices.push(point);
                vertex_map.insert(key, next_index);
                let current_index = next_index;
                next_index += 1;
                current_index
            };
            
            triangle_indices[i] = vertex_index;
        }
        
        indices.push(triangle_indices);
    }

    // Create CSG from vertices and indices using csgrs's mesh creation methods
    // Note: This is a simplified approach - csgrs may have more sophisticated
    // mesh import methods that we should use when available
    create_csg_from_mesh(&vertices, &indices)
}

/// Convert a csgrs CSG object back to STL triangles
///
/// This function extracts the mesh data from a csgrs CSG object and converts
/// it back to pyvismil's Triangle format for further processing or export.
///
/// # Arguments
/// * `csg` - The csgrs CSG object to convert
///
/// # Returns
/// * `Result<Vec<Triangle>, &'static str>` - The resulting triangles or error
pub fn csg_to_triangles(csg: &CsgrsCSG) -> Result<Vec<Triangle>, &'static str> {
    // Use csgrs's tessellation and vertex extraction to get triangle data
    let tessellated_csg = csg.tessellate();
    let polygons = &tessellated_csg.polygons;

    let mut triangles = Vec::new();

    for polygon in polygons {
        // Each polygon should be a triangle after tessellation
        if polygon.vertices.len() != 3 {
            continue; // Skip non-triangular polygons
        }

        // Extract vertices and normal
        let v1 = &polygon.vertices[0];
        let v2 = &polygon.vertices[1];
        let v3 = &polygon.vertices[2];

        // Convert to STL Triangle format
        let triangle = Triangle {
            normal: stl_io::Vector::new([
                v1.normal.x as f32,
                v1.normal.y as f32,
                v1.normal.z as f32
            ]),
            vertices: [
                stl_io::Vector::new([v1.pos.x as f32, v1.pos.y as f32, v1.pos.z as f32]),
                stl_io::Vector::new([v2.pos.x as f32, v2.pos.y as f32, v2.pos.z as f32]),
                stl_io::Vector::new([v3.pos.x as f32, v3.pos.y as f32, v3.pos.z as f32]),
            ],
        };

        triangles.push(triangle);
    }

    Ok(triangles)
}

/// Perform CSG difference operation using csgrs
/// 
/// This is the main function that replaces pyvismil's internal CSG difference
/// operation with csgrs's robust implementation.
/// 
/// # Arguments
/// * `mesh_a` - The base mesh (what to subtract FROM)
/// * `mesh_b` - The tool mesh (what to subtract)
/// 
/// # Returns
/// * `Result<Vec<Triangle>, &'static str>` - The resulting mesh after subtraction
pub fn csgrs_difference(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    if mesh_a.is_empty() {
        return Ok(Vec::new());
    }
    if mesh_b.is_empty() {
        return Ok(mesh_a.to_vec());
    }

    // Convert both meshes to csgrs CSG objects
    let csg_a = triangles_to_csg(mesh_a)?;
    let csg_b = triangles_to_csg(mesh_b)?;

    // Perform the difference operation using csgrs
    let result_csg = csg_a.difference(&csg_b);

    // Convert the result back to triangles
    csg_to_triangles(&result_csg)
}

/// Helper function to create a CSG object from vertices and indices
///
/// This function uses csgrs's polyhedron method to create a CSG object
/// from triangle mesh data.
fn create_csg_from_mesh(vertices: &[Point3<f32>], indices: &[[u32; 3]]) -> Result<CsgrsCSG, &'static str> {
    if vertices.is_empty() || indices.is_empty() {
        return Err("Cannot create CSG from empty vertices or indices");
    }

    // Convert vertices to the format expected by csgrs::polyhedron (f64)
    let points: Vec<[f64; 3]> = vertices
        .iter()
        .map(|v| [v.x as f64, v.y as f64, v.z as f64])
        .collect();

    // Convert triangle indices to face format
    let faces: Vec<Vec<usize>> = indices
        .iter()
        .map(|tri| vec![tri[0] as usize, tri[1] as usize, tri[2] as usize])
        .collect();

    // Create CSG using csgrs's polyhedron method
    let csg = CsgrsCSG::polyhedron(&points, &faces, None);
    Ok(csg)
}



/// Convenience function for other CSG operations using csgrs
pub mod operations {
    use super::*;

    /// Union operation using csgrs
    pub fn csgrs_union(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
        if mesh_a.is_empty() {
            return Ok(mesh_b.to_vec());
        }
        if mesh_b.is_empty() {
            return Ok(mesh_a.to_vec());
        }

        let csg_a = triangles_to_csg(mesh_a)?;
        let csg_b = triangles_to_csg(mesh_b)?;
        let result_csg = csg_a.union(&csg_b);
        csg_to_triangles(&result_csg)
    }

    /// Intersection operation using csgrs
    pub fn csgrs_intersect(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
        if mesh_a.is_empty() || mesh_b.is_empty() {
            return Ok(Vec::new());
        }

        let csg_a = triangles_to_csg(mesh_a)?;
        let csg_b = triangles_to_csg(mesh_b)?;
        let result_csg = csg_a.intersection(&csg_b);
        csg_to_triangles(&result_csg)
    }
}
