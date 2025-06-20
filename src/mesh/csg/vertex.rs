//! src/mesh/csg/vertex.rs

use nalgebra::Vector3;

#[derive(Clone, Debug, PartialEq)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl Vertex {
    pub fn new(pos: Vector3<f32>, normal: Vector3<f32>) -> Self {
        Self { pos, normal }
    }

    pub fn flip(&mut self) {
        self.normal = -self.normal;
    }

    pub fn interpolate(&self, other: &Vertex, t: f32) -> Vertex {
        Vertex {
            pos: self.pos.lerp(&other.pos, t),
            normal: self.normal.lerp(&other.normal, t).normalize(),
        }
    }
} 