//! src/mesh/primitives/cylinder.rs

use crate::geometry::mod_3d::Cylinder;
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

    let start_normal = Vector::new([-1.0, 0.0, 0.0]);
    let end_normal = Vector::new([1.0, 0.0, 0.0]);

    for i in 0..SEGMENTS {
        let j = (i + 1) % SEGMENTS;
        triangles.push(Triangle {
            normal: start_normal.clone(),
            vertices: [
                Vector::new(start_center),
                Vector::new(start_vertices[j]),
                Vector::new(start_vertices[i]),
            ],
        });
        triangles.push(Triangle {
            normal: end_normal.clone(),
            vertices: [
                Vector::new(end_center),
                Vector::new(end_vertices[i]),
                Vector::new(end_vertices[j]),
            ],
        });
    }
    triangles
} 