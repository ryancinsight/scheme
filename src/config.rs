//! src/config.rs

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

#[derive(Clone, Copy, Debug)]
pub struct CfdConfig {
    pub dynamic_viscosity: f64,
    pub inlet_pressure: f64,
    pub outlet_pressure: f64,
}

impl Default for CfdConfig {
    fn default() -> Self {
        Self {
            dynamic_viscosity: 0.001, // Water at 20°C (Pa·s)
            inlet_pressure: 1.0,
            outlet_pressure: 0.0,
        }
    }
} 