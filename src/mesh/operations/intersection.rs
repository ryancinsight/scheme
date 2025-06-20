//! src/mesh/operations/intersection.rs

use crate::mesh::{Csg, csg::{Polygon, PolygonShared, Vertex}};
use nalgebra::Vector3;
use std::sync::Arc;
use stl_io::Triangle;

/// Computes the intersection of two meshes using the custom CSG implementation.
pub fn intersection(
    mesh_a: &[Triangle],
    mesh_b: &[Triangle],
) -> Result<Vec<Triangle>, &'static str> {
    let polygons_a = triangles_to_polygons(mesh_a)?;
    let polygons_b = triangles_to_polygons(mesh_b)?;

    if polygons_a.is_empty() || polygons_b.is_empty() {
        return Ok(Vec::new());
    }

    let csg_a = Csg::from_polygons(polygons_a);
    let csg_b = Csg::from_polygons(polygons_b);

    let result_csg = csg_a.intersect(&csg_b);
    let result_polygons = result_csg.to_polygons();

    Ok(polygons_to_triangles(&result_polygons))
}

fn triangles_to_polygons(triangles: &[Triangle]) -> Result<Vec<Polygon>, &'static str> {
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

fn polygons_to_triangles(polygons: &[Polygon]) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    for poly in polygons {
        for i in 1..poly.vertices.len() - 1 {
            let v1 = &poly.vertices[0].pos;
            let v2 = &poly.vertices[i].pos;
            let v3 = &poly.vertices[i + 1].pos;

            triangles.push(Triangle {
                normal: stl_io::Vector::new([poly.plane.normal.x, poly.plane.normal.y, poly.plane.normal.z]),
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