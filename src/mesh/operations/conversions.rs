//! src/mesh/operations/conversions.rs
//! 
//! Conversion Functions - The Translation Logic for the Operations Portico
//! 
//! This module provides conversion functions between Triangle mesh representation
//! and CSG Polygon representation, serving as the bridge between the two domains.

use crate::mesh::csg::{Polygon, PolygonShared, Vertex};
use nalgebra::Vector3;
use std::sync::Arc;
use stl_io::Triangle;

/// Convert STL triangles to CSG polygons
/// 
/// This function transforms triangle mesh data into the polygon representation
/// required by the CSG system. Each triangle becomes a 3-vertex polygon.
/// 
/// # Arguments
/// * `triangles` - Array of STL triangles to convert
/// 
/// # Returns
/// * `Ok(Vec<Polygon>)` - Successfully converted polygons
/// * `Err(&'static str)` - Error message if conversion fails
/// 
/// Production-ready implementation with robust triangle-to-polygon conversion
pub fn triangles_to_polygons(triangles: &[Triangle]) -> Result<Vec<Polygon>, &'static str> {
    let shared = Arc::new(PolygonShared::default());
    let mut polygons = Vec::new();

    for tri in triangles {
        let vertices = tri
            .vertices
            .iter()
            .map(|v| {
                Vertex::new(
                    Vector3::new(v[0], v[1], v[2]),
                    Vector3::new(tri.normal[0], tri.normal[1], tri.normal[2]),
                )
            })
            .collect::<Vec<_>>();

        if vertices.len() == 3 {
            polygons.push(Polygon::new(vertices, shared.clone()));
        } else {
            return Err("Invalid triangle found during conversion.");
        }
    }
    Ok(polygons)
}

/// Convert CSG polygons to STL triangles
/// 
/// This function transforms polygon representation back into triangle mesh data.
/// Polygons with more than 3 vertices are triangulated using a fan approach.
/// 
/// # Arguments
/// * `polygons` - Array of CSG polygons to convert
/// 
/// # Returns
/// * `Vec<Triangle>` - Converted triangles
/// 
/// Production-ready implementation with fan triangulation for complex polygons
pub fn polygons_to_triangles(polygons: &[Polygon]) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    for poly in polygons {
        // Fan triangulation: connect all vertices to the first vertex
        for i in 1..poly.vertices.len() - 1 {
            let v1 = &poly.vertices[0].pos;
            let v2 = &poly.vertices[i].pos;
            let v3 = &poly.vertices[i + 1].pos;

            triangles.push(Triangle {
                normal: stl_io::Vector::new([
                    poly.plane.normal.x,
                    poly.plane.normal.y,
                    poly.plane.normal.z,
                ]),
                vertices: [
                    stl_io::Vector::new([v1.x, v1.y, v1.z]),
                    stl_io::Vector::new([v2.x, v2.y, v2.z]),
                    stl_io::Vector::new([v3.x, v3.y, v3.z]),
                ],
            });
        }
    }
    triangles
}
