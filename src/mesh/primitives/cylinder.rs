//! src/mesh/primitives/cylinder.rs

use crate::geometry::mod_3d::Cylinder;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use stl_io::{Triangle, Vector};
use std::f32::consts::PI;

const SEGMENTS: usize = 32;

/// Converts a single `Cylinder` into a closed mesh with walls and caps.
pub fn generate(cylinder: &Cylinder) -> Vec<Triangle> {
    let mut triangles = generate_walls(cylinder, false);
    triangles.extend(generate_caps(cylinder));
    triangles
}

/// Generates the wall triangles for a cylinder.
pub fn generate_walls(cylinder: &Cylinder, flip_normals: bool) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    let r = cylinder.radius as f32;
    let start_center = [
        cylinder.start.0 as f32,
        cylinder.start.1 as f32,
        cylinder.start.2 as f32,
    ];
    let end_center = [
        cylinder.end.0 as f32,
        cylinder.end.1 as f32,
        cylinder.end.2 as f32,
    ];

    let mut start_vertices = Vec::with_capacity(SEGMENTS);
    let mut end_vertices = Vec::with_capacity(SEGMENTS);

    for i in 0..SEGMENTS {
        let theta = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let y_offset = r * theta.cos();
        let z_offset = r * theta.sin();
        start_vertices.push([
            start_center[0],
            start_center[1] + y_offset,
            start_center[2] + z_offset,
        ]);
        end_vertices.push([
            end_center[0],
            end_center[1] + y_offset,
            end_center[2] + z_offset,
        ]);
    }

    for i in 0..SEGMENTS {
        let j = (i + 1) % SEGMENTS;
        let s1 = start_vertices[i];
        let s2 = start_vertices[j];
        let e1 = end_vertices[i];
        let e2 = end_vertices[j];

        let wall_normal_vec = [0.0, s1[1] - start_center[1], s1[2] - start_center[2]];
        let mag = (wall_normal_vec[1].powi(2) + wall_normal_vec[2].powi(2)).sqrt();
        let mut normal = if mag > 1e-6 {
            [0.0, wall_normal_vec[1] / mag, wall_normal_vec[2] / mag]
        } else {
            [0.0, 1.0, 0.0]
        };

        if flip_normals {
            normal[1] *= -1.0;
            normal[2] *= -1.0;
        }
        let normal_v = Vector::new(normal);

        triangles.push(Triangle {
            normal: normal_v.clone(),
            vertices: [Vector::new(s1), Vector::new(e1), Vector::new(e2)],
        });
        triangles.push(Triangle {
            normal: normal_v.clone(),
            vertices: [Vector::new(s1), Vector::new(e2), Vector::new(s2)],
        });
    }

    triangles
}

/// Generates the cap triangles for a cylinder.
pub fn generate_caps(cylinder: &Cylinder) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    let r = cylinder.radius as f32;

    let start_center = [
        cylinder.start.0 as f32,
        cylinder.start.1 as f32,
        cylinder.start.2 as f32,
    ];
    let end_center = [
        cylinder.end.0 as f32,
        cylinder.end.1 as f32,
        cylinder.end.2 as f32,
    ];

    let start_normal = Vector::new([-1.0, 0.0, 0.0]);
    let end_normal = Vector::new([1.0, 0.0, 0.0]);

    triangles.extend(generate_single_cap(start_center, r, start_normal));
    triangles.extend(generate_single_cap(end_center, r, end_normal));

    triangles
}

/// Uses a 2D triangulation library to generate a mesh for a single cylinder cap.
fn generate_single_cap(center: [f32; 3], radius: f32, normal: Vector<f32>) -> Vec<Triangle> {
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f32>>::new();

    // Define the circular boundary constraint
    let mut cap_vertices = Vec::new();
    for i in 0..SEGMENTS {
        let theta = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let u = radius * theta.cos(); // Local 2D coordinates (Y-axis)
        let v = radius * theta.sin(); // Local 2D coordinates (Z-axis)
        if let Ok(handle) = cdt.insert(Point2::new(u, v)) {
            cap_vertices.push(handle);
        }
    }
    for i in 0..SEGMENTS {
        cdt.add_constraint(cap_vertices[i], cap_vertices[(i + 1) % SEGMENTS]);
    }

    // Triangulate the constrained area and lift to 3D
    let mut triangles = Vec::new();
    for face in cdt.inner_faces() {
        let handles = face.vertices();
        let p1_2d = handles[0].position();
        let p2_2d = handles[1].position();
        let p3_2d = handles[2].position();

        // The 2D (u,v) plane corresponds to the 3D (y,z) plane.
        let v1_3d = Vector::new([center[0], center[1] + p1_2d.x, center[2] + p1_2d.y]);
        let v2_3d = Vector::new([center[0], center[1] + p2_2d.x, center[2] + p2_2d.y]);
        let v3_3d = Vector::new([center[0], center[1] + p3_2d.x, center[2] + p3_2d.y]);

        // Ensure correct winding order based on the normal
        if normal[0] > 0.0 { // End cap (+X normal)
            triangles.push(Triangle {
                normal: normal.clone(),
                vertices: [v1_3d, v2_3d, v3_3d],
            });
        } else { // Start cap (-X normal)
            triangles.push(Triangle {
                normal: normal.clone(),
                vertices: [v1_3d, v3_3d, v2_3d],
            });
        }
    }
    triangles
} 