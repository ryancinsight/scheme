use crate::geometry::ChannelSystem;
use nalgebra::{DMatrix, DVector};
use std::collections::{HashMap, HashSet};
use crate::config::CfdConfig;
use crate::error::SimulationError;

pub fn calculate_node_pressures(
    system: &ChannelSystem,
    channel_resistances: &HashMap<usize, f64>,
    config: &CfdConfig,
) -> Result<HashMap<usize, f64>, SimulationError> {
    let mut node_pressures = HashMap::new();
    let mut boundary_nodes = HashSet::new();
    let min_x = 0.0;
    let max_x = system.box_dims.0;

    for node in &system.nodes {
        if (node.point.0 - min_x).abs() < 1e-9 {
            node_pressures.insert(node.id, config.inlet_pressure);
            boundary_nodes.insert(node.id);
        } else if (node.point.0 - max_x).abs() < 1e-9 {
            node_pressures.insert(node.id, config.outlet_pressure);
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

            for channel in system
                .channels
                .iter()
                .filter(|c| c.from_node == node_id || c.to_node == node_id)
            {
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

        if let Some(x) = a.lu().solve(&b) {
            for (i, &node_id) in internal_nodes.iter().enumerate() {
                node_pressures.insert(node_id, x[i]);
            }
        } else {
            return Err(SimulationError::LinearSystemError);
        }
    }

    Ok(node_pressures)
} 