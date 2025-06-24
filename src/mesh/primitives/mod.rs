//! src/mesh/primitives/mod.rs

pub mod cuboid;
pub mod cylinder;
pub mod sphere;
pub mod cone;
pub mod torus;

pub use cuboid::generate as generate_cuboid;
pub use cylinder::generate as generate_cylinder;
pub use sphere::generate as generate_sphere;
pub use cone::generate as generate_cone;
pub use torus::generate as generate_torus;