//! src/mesh/operations/difference.rs

use crate::geometry::mod_3d::{ChannelSystem3D, Cylinder, Volume};
use crate::mesh::primitives::cuboid;
use crate::mesh::primitives::cylinder::generate_walls;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use stl_io::{Triangle, Vector};
use std::collections::HashMap;
use std::f32::consts::PI;

const SEGMENTS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Face {
    Back,  // -X
    Front, // +X
    Left,  // -Y
    Right, // +Y
    Bottom,// -Z
    Top,   // +Z
}

pub fn difference(system: &ChannelSystem3D) -> Result<Vec<Triangle>, &'static str> {
    let mut triangles = Vec::new();

    for cylinder in &system.cylinders {
        triangles.extend(generate_walls(cylinder, true));
    }

    let piercings = find_all_piercings(system);
    let all_faces = [
        Face::Back,
        Face::Front,
        Face::Left,
        Face::Right,
        Face::Bottom,
        Face::Top,
    ];

    for face_id in &all_faces {
        if let Some(holes) = piercings.get(face_id) {
            triangles.extend(generate_pierced_face(&system.box_volume, holes, *face_id)?);
        } else {
            triangles.extend(generate_solid_unpierced_face(
                &system.box_volume,
                *face_id,
            ));
        }
    }

    Ok(triangles)
}

fn find_all_piercings(system: &ChannelSystem3D) -> HashMap<Face, Vec<&Cylinder>> {
    let mut piercings: HashMap<Face, Vec<&Cylinder>> = HashMap::new();
    let min = system.box_volume.min_corner;
    let max = system.box_volume.max_corner;

    for cyl in &system.cylinders {
        let start = cyl.start;
        let end = cyl.end;
        
        // Check start point
        if (start.0 - min.0).abs() < 1e-6 { piercings.entry(Face::Back).or_default().push(cyl); }
        if (start.0 - max.0).abs() < 1e-6 { piercings.entry(Face::Front).or_default().push(cyl); }
        if (start.1 - min.1).abs() < 1e-6 { piercings.entry(Face::Left).or_default().push(cyl); }
        if (start.1 - max.1).abs() < 1e-6 { piercings.entry(Face::Right).or_default().push(cyl); }
        if (start.2 - min.2).abs() < 1e-6 { piercings.entry(Face::Bottom).or_default().push(cyl); }
        if (start.2 - max.2).abs() < 1e-6 { piercings.entry(Face::Top).or_default().push(cyl); }
        
        // Check end point
        if (end.0 - min.0).abs() < 1e-6 { piercings.entry(Face::Back).or_default().push(cyl); }
        if (end.0 - max.0).abs() < 1e-6 { piercings.entry(Face::Front).or_default().push(cyl); }
        if (end.1 - min.1).abs() < 1e-6 { piercings.entry(Face::Left).or_default().push(cyl); }
        if (end.1 - max.1).abs() < 1e-6 { piercings.entry(Face::Right).or_default().push(cyl); }
        if (end.2 - min.2).abs() < 1e-6 { piercings.entry(Face::Bottom).or_default().push(cyl); }
        if (end.2 - max.2).abs() < 1e-6 { piercings.entry(Face::Top).or_default().push(cyl); }
    }
    
    for holes in piercings.values_mut() {
        holes.sort_by_key(|c| c as *const _ as usize);
        holes.dedup_by_key(|c| c as *const _ as usize);
    }

    piercings
}

fn generate_solid_unpierced_face(volume: &Volume, face: Face) -> Vec<Triangle> {
    let v_f64 = volume.get_vertices();
    let v: Vec<Vector<f32>> = v_f64
        .iter()
        .map(|&(x, y, z)| Vector::new([x as f32, y as f32, z as f32]))
        .collect();

    match face {
        Face::Top => cuboid::generate_face(v[4], v[5], v[6], v[7], Vector::new([0.0, 0.0, 1.0])),
        Face::Bottom => cuboid::generate_face(v[0], v[3], v[2], v[1], Vector::new([0.0, 0.0, -1.0])),
        Face::Right => cuboid::generate_face(v[3], v[7], v[6], v[2], Vector::new([0.0, 1.0, 0.0])),
        Face::Left => cuboid::generate_face(v[0], v[1], v[5], v[4], Vector::new([0.0, -1.0, 0.0])),
        Face::Front => cuboid::generate_face(v[1], v[2], v[6], v[5], Vector::new([1.0, 0.0, 0.0])),
        Face::Back => cuboid::generate_face(v[0], v[4], v[7], v[3], Vector::new([-1.0, 0.0, 0.0])),
    }
}

fn generate_pierced_face(
    volume: &Volume,
    cylinders: &[&Cylinder],
    face: Face,
) -> Result<Vec<Triangle>, &'static str> {
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f32>>::new();

    let (p1, p2, p3, p4, normal, const_coord, get_2d_coords, get_3d_vertex, get_const_coord) = match face {
        Face::Back => (
            (volume.min_corner.1, volume.min_corner.2), (volume.max_corner.1, volume.min_corner.2),
            (volume.max_corner.1, volume.max_corner.2), (volume.min_corner.1, volume.max_corner.2),
            Vector::new([-1.0, 0.0, 0.0]), volume.min_corner.0,
            Box::new(|p: (f64,f64,f64)| (p.1, p.2)) as Box<dyn Fn((f64,f64,f64)) -> (f64, f64)>,
            Box::new(|p: Point2<f32>, c: f64| Vector::new([c as f32, p.x, p.y])) as Box<dyn Fn(Point2<f32>, f64) -> Vector<f32>>,
            Box::new(|p: (f64,f64,f64)| p.0) as Box<dyn Fn((f64,f64,f64)) -> f64>,
        ),
        Face::Front => (
            (volume.min_corner.1, volume.min_corner.2), (volume.max_corner.1, volume.min_corner.2),
            (volume.max_corner.1, volume.max_corner.2), (volume.min_corner.1, volume.max_corner.2),
            Vector::new([1.0, 0.0, 0.0]), volume.max_corner.0,
            Box::new(|p: (f64,f64,f64)| (p.1, p.2)) as Box<dyn Fn((f64,f64,f64)) -> (f64, f64)>,
            Box::new(|p: Point2<f32>, c: f64| Vector::new([c as f32, p.x, p.y])) as Box<dyn Fn(Point2<f32>, f64) -> Vector<f32>>,
            Box::new(|p: (f64,f64,f64)| p.0) as Box<dyn Fn((f64,f64,f64)) -> f64>,
        ),
        Face::Left => (
            (volume.min_corner.0, volume.min_corner.2), (volume.max_corner.0, volume.min_corner.2),
            (volume.max_corner.0, volume.max_corner.2), (volume.min_corner.0, volume.max_corner.2),
            Vector::new([0.0, -1.0, 0.0]), volume.min_corner.1,
            Box::new(|p: (f64,f64,f64)| (p.0, p.2)) as Box<dyn Fn((f64,f64,f64)) -> (f64, f64)>,
            Box::new(|p: Point2<f32>, c: f64| Vector::new([p.x, c as f32, p.y])) as Box<dyn Fn(Point2<f32>, f64) -> Vector<f32>>,
            Box::new(|p: (f64,f64,f64)| p.1) as Box<dyn Fn((f64,f64,f64)) -> f64>,
        ),
        Face::Right => (
            (volume.min_corner.0, volume.min_corner.2), (volume.max_corner.0, volume.min_corner.2),
            (volume.max_corner.0, volume.max_corner.2), (volume.min_corner.0, volume.max_corner.2),
            Vector::new([0.0, 1.0, 0.0]), volume.max_corner.1,
            Box::new(|p: (f64,f64,f64)| (p.0, p.2)) as Box<dyn Fn((f64,f64,f64)) -> (f64, f64)>,
            Box::new(|p: Point2<f32>, c: f64| Vector::new([p.x, c as f32, p.y])) as Box<dyn Fn(Point2<f32>, f64) -> Vector<f32>>,
            Box::new(|p: (f64,f64,f64)| p.1) as Box<dyn Fn((f64,f64,f64)) -> f64>,
        ),
        Face::Bottom => (
            (volume.min_corner.0, volume.min_corner.1), (volume.max_corner.0, volume.min_corner.1),
            (volume.max_corner.0, volume.max_corner.1), (volume.min_corner.0, volume.max_corner.1),
            Vector::new([0.0, 0.0, -1.0]), volume.min_corner.2,
            Box::new(|p: (f64,f64,f64)| (p.0, p.1)) as Box<dyn Fn((f64,f64,f64)) -> (f64, f64)>,
            Box::new(|p: Point2<f32>, c: f64| Vector::new([p.x, p.y, c as f32])) as Box<dyn Fn(Point2<f32>, f64) -> Vector<f32>>,
            Box::new(|p: (f64,f64,f64)| p.2) as Box<dyn Fn((f64,f64,f64)) -> f64>,
        ),
        Face::Top => (
            (volume.min_corner.0, volume.min_corner.1), (volume.max_corner.0, volume.min_corner.1),
            (volume.max_corner.0, volume.max_corner.1), (volume.min_corner.0, volume.max_corner.1),
            Vector::new([0.0, 0.0, 1.0]), volume.max_corner.2,
            Box::new(|p: (f64,f64,f64)| (p.0, p.1)) as Box<dyn Fn((f64,f64,f64)) -> (f64, f64)>,
            Box::new(|p: Point2<f32>, c: f64| Vector::new([p.x, p.y, c as f32])) as Box<dyn Fn(Point2<f32>, f64) -> Vector<f32>>,
            Box::new(|p: (f64,f64,f64)| p.2) as Box<dyn Fn((f64,f64,f64)) -> f64>,
        ),
    };

    let v1 = cdt.insert(Point2::new(p1.0 as f32, p1.1 as f32)).map_err(|_| "CDT insert failed")?;
    let v2 = cdt.insert(Point2::new(p2.0 as f32, p2.1 as f32)).map_err(|_| "CDT insert failed")?;
    let v3 = cdt.insert(Point2::new(p3.0 as f32, p3.1 as f32)).map_err(|_| "CDT insert failed")?;
    let v4 = cdt.insert(Point2::new(p4.0 as f32, p4.1 as f32)).map_err(|_| "CDT insert failed")?;
    cdt.add_constraint(v1, v2);
    cdt.add_constraint(v2, v3);
    cdt.add_constraint(v3, v4);
    cdt.add_constraint(v4, v1);

    for cylinder in cylinders {
        let r = cylinder.radius as f32;
        let center = if (get_const_coord(cylinder.start) - const_coord).abs() < 1e-6 { cylinder.start } else { cylinder.end };
        let (center_u, center_v) = get_2d_coords(center);

        let mut hole_vertices = Vec::new();
        for i in 0..SEGMENTS {
            let theta = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
            let u = center_u as f32 + r * theta.cos();
            let v = center_v as f32 + r * theta.sin();
            hole_vertices.push(cdt.insert(Point2::new(u, v)).map_err(|_| "CDT insert failed")?);
        }
        for i in 0..SEGMENTS {
            cdt.add_constraint(hole_vertices[i], hole_vertices[(i + 1) % SEGMENTS]);
        }
    }

    let mut triangles = Vec::new();
    for face_2d in cdt.inner_faces() {
        let v_2d = face_2d.vertices();
        let p_2d = [v_2d[0].position(), v_2d[1].position(), v_2d[2].position()];
        let centroid = Point2::new((p_2d[0].x + p_2d[1].x + p_2d[2].x) / 3.0, (p_2d[0].y + p_2d[1].y + p_2d[2].y) / 3.0);

        let in_hole = cylinders.iter().any(|cyl| {
            let r_sq = (cyl.radius as f32).powi(2);
            let center = if (get_const_coord(cyl.start) - const_coord).abs() < 1e-6 { cyl.start } else { cyl.end };
            let (center_u, center_v) = get_2d_coords(center);
            (centroid.x - center_u as f32).powi(2) + (centroid.y - center_v as f32).powi(2) < r_sq
        });

        if !in_hole {
            let v_3d = p_2d.map(|p| get_3d_vertex(p, const_coord));
            if normal[0] > 0.0 || normal[1] > 0.0 || normal[2] > 0.0 {
                triangles.push(Triangle { normal, vertices: [v_3d[0], v_3d[1], v_3d[2]] });
            } else {
                triangles.push(Triangle { normal, vertices: [v_3d[0], v_3d[2], v_3d[1]] });
            }
        }
    }

    Ok(triangles)
} 