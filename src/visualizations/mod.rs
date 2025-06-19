//! visualizations/mod.rs

pub mod cfd;
pub mod schematic;
pub mod shared_utilities;
pub mod mod_3d;

pub use cfd::plot_cfd_results;
pub use schematic::plot_geometry;
pub use mod_3d::plot_3d_system; 