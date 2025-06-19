use crate::geometry::ChannelSystem;
use std::collections::HashMap;
use crate::config::CfdConfig;

pub fn calculate_hydrodynamic_resistance(
    channel_length: f64,
    channel_width: f64,
    channel_height: f64,
    config: &CfdConfig,
) -> f64 {
    if channel_width <= 0.0 || channel_height <= 0.0 {
        return f64::INFINITY;
    }
    // Equation from https://arxiv.org/html/2406.15562v2
    (12.0 * config.dynamic_viscosity * channel_length)
        / (channel_width
            * channel_height.powi(3)
            * (1.0 - 0.63 * channel_height / channel_width))
}

pub fn calculate_all_resistances(
    system: &ChannelSystem,
    config: &CfdConfig,
) -> HashMap<usize, f64> {
    system
        .channels
        .iter()
        .map(|channel| {
            let p1 = system.nodes[channel.from_node].point;
            let p2 = system.nodes[channel.to_node].point;
            let length = ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt();
            let resistance =
                calculate_hydrodynamic_resistance(length, channel.width, channel.height, config);
            (channel.id, resistance)
        })
        .collect()
} 