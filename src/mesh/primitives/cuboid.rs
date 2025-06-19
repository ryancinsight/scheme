//! src/mesh/primitives/cuboid.rs

use crate::geometry::mod_3d::Volume;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use stl_io::{Triangle, Vector};

/// Computes the cross product of two vectors.
pub(crate) fn cross_product(a: Vector<f32>, b: Vector<f32>) -> Vector<f32> {
    Vector::new([
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ])
}

/// Computes the dot product of two vectors.
pub(crate) fn dot_product(a: Vector<f32>, b: Vector<f32>) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Generates a triangulated mesh for a single rectangular face.
pub(crate) fn generate_face(
    p1: Vector<f32>,
    p2: Vector<f32>,
    p3: Vector<f32>,
    p4: Vector<f32>,
    normal: Vector<f32>,
) -> Vec<Triangle> {
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f32>>::new();

    // Project the 3D face vertices to a 2D plane based on the normal.
    let (project, lift): (
        Box<dyn Fn(Vector<f32>) -> Point2<f32>>,
        Box<dyn Fn(Point2<f32>) -> Vector<f32>>,
    ) = if normal[2].abs() > 0.9 {
        // XY plane
        let z = p1[2];
        (
            Box::new(move |p| Point2::new(p[0], p[1])),
            Box::new(move |p| Vector::new([p.x, p.y, z])),
        )
    } else if normal[1].abs() > 0.9 {
        // XZ plane
        let y = p1[1];
        (
            Box::new(move |p| Point2::new(p[0], p[2])),
            Box::new(move |p| Vector::new([p.x, y, p.y])),
        )
    } else {
        // YZ plane
        let x = p1[0];
        (
            Box::new(move |p| Point2::new(p[1], p[2])),
            Box::new(move |p| Vector::new([x, p.x, p.y])),
        )
    };

    // Add the face boundary as constraints to the triangulation.
    let h1 = cdt.insert(project(p1)).unwrap();
    let h2 = cdt.insert(project(p2)).unwrap();
    let h3 = cdt.insert(project(p3)).unwrap();
    let h4 = cdt.insert(project(p4)).unwrap();
    cdt.add_constraint(h1, h2);
    cdt.add_constraint(h2, h3);
    cdt.add_constraint(h3, h4);
    cdt.add_constraint(h4, h1);

    let mut triangles = Vec::new();
    for face in cdt.inner_faces() {
        let v = face.vertices();
        let v1_3d = lift(v[0].position());
        let v2_3d = lift(v[1].position());
        let v3_3d = lift(v[2].position());

        // Ensure the winding order is correct for the face's normal.
        let edge1 = Vector::new([v2_3d[0] - v1_3d[0], v2_3d[1] - v1_3d[1], v2_3d[2] - v1_3d[2]]);
        let edge2 = Vector::new([v3_3d[0] - v1_3d[0], v3_3d[1] - v1_3d[1], v3_3d[2] - v1_3d[2]]);
        let cross = cross_product(edge1, edge2);
        if dot_product(cross, normal) > 0.0 {
            triangles.push(Triangle {
                normal,
                vertices: [v1_3d, v2_3d, v3_3d],
            });
        } else {
            triangles.push(Triangle {
                normal,
                vertices: [v1_3d, v3_3d, v2_3d],
            });
        }
    }
    triangles
}

/// Converts a single `Volume` (cuboid) into 12 triangles using Delaunay triangulation.
pub fn generate(volume: &Volume) -> Vec<Triangle> {
    let v_f64 = volume.get_vertices();
    let v: Vec<Vector<f32>> = v_f64
        .iter()
        .map(|&(x, y, z)| Vector::new([x as f32, y as f32, z as f32]))
        .collect();

    let mut triangles = Vec::new();

    // Top face (+Z)
    triangles.extend(generate_face(v[4], v[5], v[6], v[7], Vector::new([0.0, 0.0, 1.0])));
    // Bottom face (-Z)
    triangles.extend(generate_face(v[0], v[3], v[2], v[1], Vector::new([0.0, 0.0, -1.0])));
    // Right face (+Y)
    triangles.extend(generate_face(v[3], v[7], v[6], v[2], Vector::new([0.0, 1.0, 0.0])));
    // Left face (-Y)
    triangles.extend(generate_face(v[0], v[1], v[5], v[4], Vector::new([0.0, -1.0, 0.0])));
    // Front face (+X)
    triangles.extend(generate_face(v[1], v[2], v[6], v[5], Vector::new([1.0, 0.0, 0.0])));
    // Back face (-X)
    triangles.extend(generate_face(v[0], v[4], v[7], v[3], Vector::new([-1.0, 0.0, 0.0])));
    
    triangles
} 