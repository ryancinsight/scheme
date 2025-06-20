//! src/mesh/csg/polygon.rs

use super::vertex::Vertex;
use super::plane::Plane;
use std::sync::Arc;

// Using Arc for shared properties to avoid copying large data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolygonShared {
    // Potentially store material, texture, etc.
}

#[derive(Clone, Debug)]
pub struct Polygon {
    pub vertices: Vec<Vertex>,
    pub shared: Arc<PolygonShared>,
    pub plane: Plane,
}

impl Polygon {
    pub fn new(vertices: Vec<Vertex>, shared: Arc<PolygonShared>) -> Self {
        let plane = Plane::from_points(
            &vertices[0].pos,
            &vertices[1].pos,
            &vertices[2].pos,
        );
        Self { vertices, shared, plane }
    }

    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.flip();
        }
        self.plane.flip();
    }
} 