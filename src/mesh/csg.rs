//! src/mesh/csg.rs
use crate::geometry::mod_3d::{Cylinder, Volume};
use crate::mesh::generator::generate_cylinder_walls;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use stl_io::{Triangle, Vector};
use std::f32::consts::PI;

const SEGMENTS: usize = 32;

/// Subtracts a cylinder from a volume, creating a hollowed-out mesh.
/// Assumes the cylinder is aligned with the X-axis and passes through the volume.
pub fn subtract_cylinder_from_volume(volume: &Volume, cylinder: &Cylinder) -> Vec<Triangle> {
    let mut triangles = Vec::new();

    // 1. Add the inner walls from the cylinder, with normals flipped to face inward.
    triangles.extend(generate_cylinder_walls(cylinder, true));

    // 2. Add the four box faces that are not intersected by the cylinder.
    triangles.extend(generate_non_intersected_faces(volume));

    // 3. Generate the front and back faces with circular holes.
    // Assumes cylinder is X-aligned and fully penetrates the box.
    let front_face_x = volume.max_corner.0;
    let back_face_x = volume.min_corner.0;

    triangles.extend(generate_face_with_hole(volume, cylinder, back_face_x));
    triangles.extend(generate_face_with_hole(volume, cylinder, front_face_x));

    triangles
}

/// Generates the 4 outer faces of the volume that are not pierced by the cylinder.
fn generate_non_intersected_faces(volume: &Volume) -> Vec<Triangle> {
    let v_f64 = volume.get_vertices();
    let v: Vec<Vector<f32>> = v_f64
        .iter()
        .map(|&(x, y, z)| Vector::new([x as f32, y as f32, z as f32]))
        .collect();

    vec![
        // Top face (+Z)
        Triangle { normal: Vector::new([0.0, 0.0, 1.0]), vertices: [v[4].clone(), v[5].clone(), v[6].clone()] },
        Triangle { normal: Vector::new([0.0, 0.0, 1.0]), vertices: [v[4].clone(), v[6].clone(), v[7].clone()] },
        // Bottom face (-Z)
        Triangle { normal: Vector::new([0.0, 0.0, -1.0]), vertices: [v[0].clone(), v[2].clone(), v[1].clone()] },
        Triangle { normal: Vector::new([0.0, 0.0, -1.0]), vertices: [v[0].clone(), v[3].clone(), v[2].clone()] },
        // Front face (+Y)
        Triangle { normal: Vector::new([0.0, 1.0, 0.0]), vertices: [v[3].clone(), v[2].clone(), v[6].clone()] },
        Triangle { normal: Vector::new([0.0, 1.0, 0.0]), vertices: [v[3].clone(), v[6].clone(), v[7].clone()] },
        // Back face (-Y)
        Triangle { normal: Vector::new([0.0, -1.0, 0.0]), vertices: [v[0].clone(), v[5].clone(), v[1].clone()] },
        Triangle { normal: Vector::new([0.0, -1.0, 0.0]), vertices: [v[0].clone(), v[4].clone(), v[5].clone()] },
    ]
}

/// Uses a 2D triangulation library to generate a face with a hole.
fn generate_face_with_hole(
    volume: &Volume,
    cylinder: &Cylinder,
    x_coord: f64,
) -> Vec<Triangle> {
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f32>>::new();

    // Define the outer boundary (the box face)
    let y_min = volume.min_corner.1 as f32;
    let y_max = volume.max_corner.1 as f32;
    let z_min = volume.min_corner.2 as f32;
    let z_max = volume.max_corner.2 as f32;

    let v1 = cdt.insert(Point2::new(y_min, z_min)).unwrap();
    let v2 = cdt.insert(Point2::new(y_max, z_min)).unwrap();
    let v3 = cdt.insert(Point2::new(y_max, z_max)).unwrap();
    let v4 = cdt.insert(Point2::new(y_min, z_max)).unwrap();
    cdt.add_constraint(v1, v2);
    cdt.add_constraint(v2, v3);
    cdt.add_constraint(v3, v4);
    cdt.add_constraint(v4, v1);

    // Define the inner boundary (the circular hole)
    let r = cylinder.radius as f32;
    let center_y = cylinder.start.1 as f32;
    let center_z = cylinder.start.2 as f32;
    let mut hole_vertices = Vec::new();
    for i in 0..SEGMENTS {
        let theta = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let y = center_y + r * theta.cos();
        let z = center_z + r * theta.sin();
        hole_vertices.push(cdt.insert(Point2::new(y, z)).unwrap());
    }
    for i in 0..SEGMENTS {
        cdt.add_constraint(hole_vertices[i], hole_vertices[(i + 1) % SEGMENTS]);
    }

    // Triangulate and build the mesh
    let mut triangles = Vec::new();
    let normal_x = if x_coord == volume.min_corner.0 { -1.0 } else { 1.0 };
    let normal = Vector::new([normal_x, 0.0, 0.0]);

    for face in cdt.inner_faces() {
        let vertices_handles = face.vertices();
        let p1_2d = vertices_handles[0].position();
        let p2_2d = vertices_handles[1].position();
        let p3_2d = vertices_handles[2].position();

        // Check if the triangle is outside the hole
        let centroid = Point2::new(
            (p1_2d.x + p2_2d.x + p3_2d.x) / 3.0,
            (p1_2d.y + p2_2d.y + p3_2d.y) / 3.0,
        );
        let dist_sq = (centroid.x - center_y).powi(2) + (centroid.y - center_z).powi(2);
        if dist_sq < r * r {
            continue; // Skip triangles inside the hole
        }

        // Lift 2D points back to 3D and create the triangle
        let v1_3d = Vector::new([x_coord as f32, p1_2d.x, p1_2d.y]);
        let v2_3d = Vector::new([x_coord as f32, p2_2d.x, p2_2d.y]);
        let v3_3d = Vector::new([x_coord as f32, p3_2d.x, p3_2d.y]);

        if normal_x > 0.0 {
            triangles.push(Triangle {
                normal: normal.clone(),
                vertices: [v1_3d, v2_3d, v3_3d],
            });
        } else {
            // Reverse winding order for the back face to keep normal correct
            triangles.push(Triangle {
                normal: normal.clone(),
                vertices: [v1_3d, v3_3d, v2_3d],
            });
        }
    }

    triangles
} 