use crate::geometry::ChannelSystem;
use std::collections::HashMap;

pub fn calculate_channel_flow_rates(
    system: &ChannelSystem,
    node_pressures: &HashMap<usize, f64>,
    channel_resistances: &HashMap<usize, f64>,
) -> HashMap<usize, f64> {
    let mut channel_flow_rates = HashMap::new();
    for channel in &system.channels {
        let p1 = node_pressures
            .get(&channel.from_node)
            .cloned()
            .unwrap_or(0.0);
        let p2 = node_pressures
            .get(&channel.to_node)
            .cloned()
            .unwrap_or(0.0);
        let resistance = channel_resistances[&channel.id];
        let flow = if resistance.is_finite() && resistance > 0.0 {
            (p1 - p2) / resistance
        } else {
            0.0
        };
        channel_flow_rates.insert(channel.id, flow);
    }
    channel_flow_rates
} 