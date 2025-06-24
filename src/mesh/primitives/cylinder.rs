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

    let axis = [
        end_center[0] - start_center[0],
        end_center[1] - start_center[1],
        end_center[2] - start_center[2],
    ];
    let mag = (axis[0].powi(2) + axis[1].powi(2) + axis[2].powi(2)).sqrt();
    let axis_norm = if mag > 1e-6 {
        [axis[0] / mag, axis[1] / mag, axis[2] / mag]
    } else {
        return triangles; // Cannot generate a zero-length cylinder
    };

    let temp_vec = if axis_norm[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };

    let u_cross = [
        axis_norm[1] * temp_vec[2] - axis_norm[2] * temp_vec[1],
        axis_norm[2] * temp_vec[0] - axis_norm[0] * temp_vec[2],
        axis_norm[0] * temp_vec[1] - axis_norm[1] * temp_vec[0],
    ];
    let u_mag = (u_cross[0].powi(2) + u_cross[1].powi(2) + u_cross[2].powi(2)).sqrt();
    let u_norm = [u_cross[0] / u_mag, u_cross[1] / u_mag, u_cross[2] / u_mag];
    
    let v_cross = [
        u_norm[1] * axis_norm[2] - u_norm[2] * axis_norm[1],
        u_norm[2] * axis_norm[0] - u_norm[0] * axis_norm[2],
        u_norm[0] * axis_norm[1] - u_norm[1] * axis_norm[0],
    ];
    let v_mag = (v_cross[0].powi(2) + v_cross[1].powi(2) + v_cross[2].powi(2)).sqrt();
    let v_norm = [v_cross[0] / v_mag, v_cross[1] / v_mag, v_cross[2] / v_mag];


    for i in 0..SEGMENTS {
        let theta = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let x_offset = r * (cos_theta * u_norm[0] + sin_theta * v_norm[0]);
        let y_offset = r * (cos_theta * u_norm[1] + sin_theta * v_norm[1]);
        let z_offset = r * (cos_theta * u_norm[2] + sin_theta * v_norm[2]);

        start_vertices.push([
            start_center[0] + x_offset,
            start_center[1] + y_offset,
            start_center[2] + z_offset,
        ]);
        end_vertices.push([
            end_center[0] + x_offset,
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

        let wall_normal_vec = [
            s1[0] - start_center[0], 
            s1[1] - start_center[1], 
            s1[2] - start_center[2]
        ];
        let mag = (wall_normal_vec[0].powi(2) + wall_normal_vec[1].powi(2) + wall_normal_vec[2].powi(2)).sqrt();
        let mut normal = if mag > 1e-6 {
            [
                wall_normal_vec[0] / mag, 
                wall_normal_vec[1] / mag, 
                wall_normal_vec[2] / mag
            ]
        } else {
            [0.0, 1.0, 0.0]
        };

        if flip_normals {
            normal[0] *= -1.0;
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
    
    let axis = [
        end_center[0] - start_center[0],
        end_center[1] - start_center[1],
        end_center[2] - start_center[2],
    ];
    let mag = (axis[0].powi(2) + axis[1].powi(2) + axis[2].powi(2)).sqrt();
    let axis_norm = if mag > 1e-6 {
        [axis[0] / mag, axis[1] / mag, axis[2] / mag]
    } else {
        return triangles;
    };

    let start_normal = Vector::new([-axis_norm[0], -axis_norm[1], -axis_norm[2]]);
    let end_normal = Vector::new(axis_norm);

    triangles.extend(generate_single_cap(start_center, r, start_normal));
    triangles.extend(generate_single_cap(end_center, r, end_normal));

    triangles
}

/// Uses a simple fan triangulation to generate a mesh for a single cylinder cap.
fn generate_single_cap(center: [f32; 3], radius: f32, normal: Vector<f32>) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    let axis_norm = [normal[0], normal[1], normal[2]];

    let temp_vec = if axis_norm[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };

    let u_cross = [
        axis_norm[1] * temp_vec[2] - axis_norm[2] * temp_vec[1],
        axis_norm[2] * temp_vec[0] - axis_norm[0] * temp_vec[2],
        axis_norm[0] * temp_vec[1] - axis_norm[1] * temp_vec[0],
    ];
    let u_mag = (u_cross[0].powi(2) + u_cross[1].powi(2) + u_cross[2].powi(2)).sqrt();
    let u_norm = [u_cross[0] / u_mag, u_cross[1] / u_mag, u_cross[2] / u_mag];

    let v_cross = [
        u_norm[1] * axis_norm[2] - u_norm[2] * axis_norm[1],
        u_norm[2] * axis_norm[0] - u_norm[0] * axis_norm[2],
        u_norm[0] * axis_norm[1] - u_norm[1] * axis_norm[0],
    ];
    let v_mag = (v_cross[0].powi(2) + v_cross[1].powi(2) + v_cross[2].powi(2)).sqrt();
    let v_norm = [v_cross[0] / v_mag, v_cross[1] / v_mag, v_cross[2] / v_mag];

    let mut cap_vertices = Vec::with_capacity(SEGMENTS);
    for i in 0..SEGMENTS {
        let theta = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let u_coord = radius * theta.cos();
        let v_coord = radius * theta.sin();

        cap_vertices.push(Vector::new([
            center[0] + u_coord * u_norm[0] + v_coord * v_norm[0],
            center[1] + u_coord * u_norm[1] + v_coord * v_norm[1],
            center[2] + u_coord * u_norm[2] + v_coord * v_norm[2],
        ]));
    }

    let center_v = Vector::new(center);
    for i in 0..SEGMENTS {
        let p1 = cap_vertices[i];
        let p2 = cap_vertices[(i + 1) % SEGMENTS];

        let edge1 = Vector::new([p1[0] - center_v[0], p1[1] - center_v[1], p1[2] - center_v[2]]);
        let edge2 = Vector::new([p2[0] - center_v[0], p2[1] - center_v[1], p2[2] - center_v[2]]);
        let cross = super::cuboid::cross_product(edge1, edge2);

        if super::cuboid::dot_product(cross, normal) > 0.0 {
            triangles.push(Triangle {
                normal: normal.clone(),
                vertices: [center_v, p1, p2],
            });
        } else {
            triangles.push(Triangle {
                normal: normal.clone(),
                vertices: [center_v, p2, p1],
            });
        }
    }

    triangles
}