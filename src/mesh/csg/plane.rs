//! src/mesh/csg/plane.rs

use nalgebra::Vector3;
use super::polygon::Polygon;

const EPSILON: f32 = 1e-5;

#[derive(Clone, Debug)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub w: f32,
}

impl Plane {
    pub fn new(normal: Vector3<f32>, w: f32) -> Self {
        Self { normal, w }
    }

    pub fn from_points(a: &Vector3<f32>, b: &Vector3<f32>, c: &Vector3<f32>) -> Self {
        let normal = (b - a).cross(&(c - a)).normalize();
        let w = normal.dot(a);
        Self { normal, w }
    }

    pub fn flip(&mut self) {
        self.normal = -self.normal;
        self.w = -self.w;
    }

    pub fn split_polygon(
        &self,
        polygon: &Polygon,
        co_planar_front: &mut Vec<Polygon>,
        co_planar_back: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        #[derive(PartialEq, Eq)]
        enum PointType {
            Coplanar,
            Front,
            Back,
        }

        let mut polygon_type = 0;
        let mut point_types = Vec::new();

        for v in &polygon.vertices {
            let t = self.normal.dot(&v.pos) - self.w;
            let p_type = if t < -EPSILON {
                PointType::Back
            } else if t > EPSILON {
                PointType::Front
            } else {
                PointType::Coplanar
            };
            polygon_type |= match p_type {
                PointType::Coplanar => 0,
                PointType::Front => 1,
                PointType::Back => 2,
            };
            point_types.push(p_type);
        }

        match polygon_type {
            0 => { // Coplanar
                if self.normal.dot(&polygon.plane.normal) > 0.0 {
                    co_planar_front.push(polygon.clone());
                } else {
                    co_planar_back.push(polygon.clone());
                }
            }
            1 => { // Front
                front.push(polygon.clone());
            }
            2 => { // Back
                back.push(polygon.clone());
            }
            3 => { // Spanning
                let mut f_vertices = Vec::new();
                let mut b_vertices = Vec::new();

                for i in 0..polygon.vertices.len() {
                    let j = (i + 1) % polygon.vertices.len();
                    let ti = &point_types[i];
                    let tj = &point_types[j];
                    let vi = &polygon.vertices[i];
                    let vj = &polygon.vertices[j];

                    if *ti != PointType::Back { f_vertices.push(vi.clone()); }
                    if *ti != PointType::Front { b_vertices.push(vi.clone()); }

                    if (*ti == PointType::Front && *tj == PointType::Back) || (*ti == PointType::Back && *tj == PointType::Front) {
                        let t = (self.w - self.normal.dot(&vi.pos)) / self.normal.dot(&(vj.pos - vi.pos));
                        let v = vi.interpolate(vj, t);
                        f_vertices.push(v.clone());
                        b_vertices.push(v);
                    }
                }

                if !f_vertices.is_empty() {
                    front.push(Polygon::new(f_vertices, polygon.shared.clone()));
                }
                if !b_vertices.is_empty() {
                    back.push(Polygon::new(b_vertices, polygon.shared.clone()));
                }
            }
            _ => (), // Should not happen
        }
    }
} 