//! src/mesh/primitives/mod.rs

pub mod cuboid;
pub mod cylinder;
pub mod sphere;

pub use cuboid::generate as generate_cuboid;
pub use cylinder::generate as generate_cylinder;
pub use sphere::generate as generate_sphere; 