//! src/mesh/operations/mod.rs

pub mod difference;
pub mod intersection;
pub mod subtract;
pub mod union;
pub mod xor;

pub use difference::difference;
pub use intersection::intersection;
pub use subtract::subtract;
pub use union::union;
pub use xor::xor; 