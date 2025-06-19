use crate::geometry::{CfdResults, ChannelSystem};
use nalgebra::{DMatrix, DVector};
use std::collections::HashMap;

const DYNAMIC_VISCOSITY: f64 = 0.001; // Viscosity of water at 20°C (Pa·s)
const INLET_PRESSURE: f64 = 1.0; // Arbitrary inlet pressure
const OUTLET_PRESSURE: f64 = 0.0; // Arbitrary outlet pressure

fn calculate_hydrodynamic_resistance(
    channel_length: f64,
    channel_width: f64,
    channel_height: f64,
) -> f64 {
    if channel_width <= 0.0 || channel_height <= 0.0 {
        return f64::INFINITY;
    }
    // Equation from https://arxiv.org/html/2406.15562v2
    (12.0 * DYNAMIC_VISCOSITY * channel_length)
        / (channel_width
            * channel_height.powi(3)
            * (1.0 - 0.63 * channel_height / channel_width))
}

pub fn run_simulation(system: &ChannelSystem) -> CfdResults {
    let channel_resistances: HashMap<usize, f64> = system
        .channels
        .iter()
        .map(|channel| {
            let p1 = system.nodes[channel.from_node].point;
            let p2 = system.nodes[channel.to_node].point;
            let length = ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt();
            let resistance =
                calculate_hydrodynamic_resistance(length, channel.width, channel.height);
            (channel.id, resistance)
        })
        .collect();

    let mut node_pressures = HashMap::new();
    let mut boundary_nodes = std::collections::HashSet::new();
    let min_x = 0.0;
    let max_x = system.box_dims.0;

    for node in &system.nodes {
        if (node.point.0 - min_x).abs() < 1e-9 {
            node_pressures.insert(node.id, INLET_PRESSURE);
            boundary_nodes.insert(node.id);
        } else if (node.point.0 - max_x).abs() < 1e-9 {
            node_pressures.insert(node.id, OUTLET_PRESSURE);
            boundary_nodes.insert(node.id);
        }
    }

    let internal_nodes: Vec<_> = system
        .nodes
        .iter()
        .filter(|n| !boundary_nodes.contains(&n.id))
        .map(|n| n.id)
        .collect();

    let node_to_matrix_idx: HashMap<usize, usize> = internal_nodes
        .iter()
        .enumerate()
        .map(|(i, &node_id)| (node_id, i))
        .collect();

    let n = internal_nodes.len();
    if n > 0 {
        let mut a = DMatrix::<f64>::zeros(n, n);
        let mut b = DVector::<f64>::zeros(n);

        for (i, &node_id) in internal_nodes.iter().enumerate() {
            let mut a_ii = 0.0;
            let mut b_i = 0.0;

            for channel in system.channels.iter().filter(|c| c.from_node == node_id || c.to_node == node_id) {
                let other_node_id = if channel.from_node == node_id {
                    channel.to_node
                } else {
                    channel.from_node
                };

                let resistance = channel_resistances[&channel.id];
                if resistance.is_finite() && resistance > 0.0 {
                    let conductance = 1.0 / resistance;
                    a_ii += conductance;

                    if boundary_nodes.contains(&other_node_id) {
                        b_i += conductance * node_pressures[&other_node_id];
                    } else if let Some(&j) = node_to_matrix_idx.get(&other_node_id) {
                        a[(i, j)] = -conductance;
                    }
                }
            }
            a[(i, i)] = a_ii;
            b[i] = b_i;
        }

        if let Some(solver) = a.lu().try_inverse() {
            let x = solver * b;
            for (i, &node_id) in internal_nodes.iter().enumerate() {
                node_pressures.insert(node_id, x[i]);
            }
        }
    }

    let mut channel_flow_rates = HashMap::new();
    for channel in &system.channels {
        let p1 = node_pressures.get(&channel.from_node).cloned().unwrap_or(0.0);
        let p2 = node_pressures.get(&channel.to_node).cloned().unwrap_or(0.0);
        let resistance = channel_resistances[&channel.id];
        let flow = if resistance.is_finite() && resistance > 0.0 {
            (p1 - p2) / resistance
        } else {
            0.0
        };
        channel_flow_rates.insert(channel.id, flow);
    }

    CfdResults {
        system: system.clone(),
        node_pressures,
        channel_flow_rates,
        channel_resistances,
    }
} 