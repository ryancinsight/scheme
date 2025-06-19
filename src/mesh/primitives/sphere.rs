//! src/mesh/primitives/sphere.rs

use crate::geometry::mod_3d::Sphere;
use stl_io::{Triangle, Vector};
use std::f64::consts::PI;

pub fn generate(sphere: &Sphere, stacks: u32, sectors: u32) -> Vec<Triangle> {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();

    for i in 0..=stacks {
        let stack_angle = PI / 2.0 - (i as f64 * PI / stacks as f64);
        let xy = sphere.radius * stack_angle.cos();
        let z = sphere.radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = j as f64 * 2.0 * PI / sectors as f64;
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            let vertex = Vector::new([
                (sphere.center.0 + x) as f32,
                (sphere.center.1 + y) as f32,
                (sphere.center.2 + z) as f32,
            ]);
            vertices.push(vertex);

            let normal = Vector::new([x as f32, y as f32, z as f32]);
            normals.push(normal);
        }
    }

    let mut triangles = Vec::new();
    for i in 0..stacks {
        for j in 0..sectors {
            let first = (i * (sectors + 1) + j) as usize;
            let second = first + sectors as usize + 1;

            let v1 = vertices[first];
            let v2 = vertices[first + 1];
            let v3 = vertices[second];
            let v4 = vertices[second + 1];

            let n1 = normals[first];

            // Triangles are created with vertices in counter-clockwise order for correct normals
            triangles.push(Triangle {
                normal: n1,
                vertices: [v1, v2, v4],
            });
            triangles.push(Triangle {
                normal: n1, // Simplified normal, could average them
                vertices: [v1, v4, v3],
            });
        }
    }

    triangles
} 