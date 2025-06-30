//! src/config.rs - 2D Schematic Configuration
//!
//! Configuration structures for 2D microfluidic schematic generation in Scheme.

use crate::geometry::ChannelType;

#[derive(Clone, Copy, Debug)]
pub struct GeometryConfig {
    pub wall_clearance: f64,
    pub channel_width: f64,
    pub channel_height: f64,
}

impl Default for GeometryConfig {
    fn default() -> Self {
        Self {
            wall_clearance: 0.5,
            channel_width: 1.0,
            channel_height: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SerpentineConfig {
    pub fill_factor: f64,         // Fraction of available vertical space to fill (0.0 to 1.0)
    pub wavelength_factor: f64, // Multiplier for channel width to determine wavelength
    pub gaussian_width_factor: f64, // Controls width of Gaussian envelope (sigma = length / gaussian_width_factor)
    pub wave_density_factor: f64, // Controls how many waves appear relative to channel length (higher = more waves)
}

impl Default for SerpentineConfig {
    fn default() -> Self {
        Self {
            fill_factor: 0.8,
            wavelength_factor: 3.0,  // Reduced for more waves  
            gaussian_width_factor: 6.0,
            wave_density_factor: 2.5, // Increased for more waves by default
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChannelTypeConfig {
    AllStraight,
    AllSerpentine(SerpentineConfig),
    MixedByPosition { middle_zone_fraction: f64, serpentine_config: SerpentineConfig },
    Custom(fn(from: (f64, f64), to: (f64, f64), box_dims: (f64, f64)) -> ChannelType),
}

impl Default for ChannelTypeConfig {
    fn default() -> Self {
        ChannelTypeConfig::MixedByPosition {
            middle_zone_fraction: 0.4,
            serpentine_config: SerpentineConfig::default(),
        }
    }
}

// CFD and 3D conversion configurations removed - Scheme focuses on 2D schematics only