//! lib.rs

pub mod cfd;
pub mod geometry;
pub mod visualizations;
pub mod config;
pub mod error;
pub mod mesh;

pub use visualizations::{plot_cfd_results, plot_geometry}; 