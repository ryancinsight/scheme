//! src/mesh/primitives/cuboid.rs

use crate::geometry::mod_3d::Volume;
use stl_io::{Triangle, Vector};

/// Converts a single `Volume` (cuboid) into 12 triangles.
pub fn generate(volume: &Volume) -> Vec<Triangle> {
    let v_f64 = volume.get_vertices();
    let v: Vec<Vector<f32>> = v_f64
        .iter()
        .map(|&(x, y, z)| Vector::new([x as f32, y as f32, z as f32]))
        .collect();

    vec![
        // Bottom face (-Z)
        Triangle { normal: Vector::new([0.0, 0.0, -1.0]), vertices: [v[0].clone(), v[1].clone(), v[2].clone()] },
        Triangle { normal: Vector::new([0.0, 0.0, -1.0]), vertices: [v[0].clone(), v[2].clone(), v[3].clone()] },
        // Top face (+Z)
        Triangle { normal: Vector::new([0.0, 0.0, 1.0]), vertices: [v[4].clone(), v[5].clone(), v[6].clone()] },
        Triangle { normal: Vector::new([0.0, 0.0, 1.0]), vertices: [v[4].clone(), v[6].clone(), v[7].clone()] },
        // Front face (+Y)
        Triangle { normal: Vector::new([0.0, 1.0, 0.0]), vertices: [v[3].clone(), v[2].clone(), v[6].clone()] },
        Triangle { normal: Vector::new([0.0, 1.0, 0.0]), vertices: [v[3].clone(), v[6].clone(), v[7].clone()] },
        // Back face (-Y)
        Triangle { normal: Vector::new([0.0, -1.0, 0.0]), vertices: [v[0].clone(), v[1].clone(), v[5].clone()] },
        Triangle { normal: Vector::new([0.0, -1.0, 0.0]), vertices: [v[0].clone(), v[5].clone(), v[4].clone()] },
        // Right face (+X)
        Triangle { normal: Vector::new([1.0, 0.0, 0.0]), vertices: [v[1].clone(), v[2].clone(), v[6].clone()] },
        Triangle { normal: Vector::new([1.0, 0.0, 0.0]), vertices: [v[1].clone(), v[6].clone(), v[5].clone()] },
        // Left face (-X)
        Triangle { normal: Vector::new([-1.0, 0.0, 0.0]), vertices: [v[0].clone(), v[3].clone(), v[7].clone()] },
        Triangle { normal: Vector::new([-1.0, 0.0, 0.0]), vertices: [v[0].clone(), v[7].clone(), v[4].clone()] },
    ]
} 