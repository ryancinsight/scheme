//! src/config.rs - 2D Schematic Configuration
//!
//! Configuration structures for 2D microfluidic schematic generation in Scheme.

#[derive(Clone, Copy, Debug)]
pub struct GeometryConfig {
    pub wall_clearance: f64,
    pub channel_width: f64,
    pub channel_height: f64,
}

impl Default for GeometryConfig {
    fn default() -> Self {
        Self {
            wall_clearance: 4.0,
            channel_width: 6.0,
            channel_height: 1.0,
        }
    }
}

// CFD and 3D conversion configurations removed - Scheme focuses on 2D schematics only