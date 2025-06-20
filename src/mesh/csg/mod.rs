//! src/mesh/csg/mod.rs

mod plane;
mod vertex;
mod polygon;
mod node;
pub mod api;

pub use self::{
    api::Csg,
    polygon::{Polygon, PolygonShared},
    vertex::Vertex,
}; 