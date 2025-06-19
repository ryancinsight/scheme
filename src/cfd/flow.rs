use crate::geometry::{ChannelSystem, Node};
use std::collections::{HashMap, VecDeque};

pub type FlowResults = HashMap<usize, f64>;

pub fn trace_flow(system: &ChannelSystem, initial_flow_rate: f64) -> FlowResults {
    let mut flow_results: FlowResults = HashMap::new();
    if system.nodes.is_empty() || system.channels.is_empty() {
        return flow_results;
    }

    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut in_degree: HashMap<usize, usize> = HashMap::new();
    let mut channel_map: HashMap<(usize, usize), usize> = HashMap::new();

    for node in &system.nodes {
        in_degree.insert(node.id, 0);
        adj.insert(node.id, Vec::new());
    }

    for channel in &system.channels {
        adj.entry(channel.from_node)
            .or_default()
            .push(channel.to_node);
        in_degree
            .entry(channel.to_node)
            .and_modify(|d| *d += 1)
            .or_insert(1);
        channel_map.insert((channel.from_node, channel.to_node), channel.id);
    }

    let Some(inlet_node) = find_inlet_node(&system.nodes) else {
        return flow_results;
    };

    let mut node_flow: HashMap<usize, f64> = HashMap::new();
    let mut queue: VecDeque<usize> = VecDeque::new();

    for node_id in in_degree.keys() {
        if in_degree.get(node_id) == Some(&0) {
            queue.push_back(*node_id);
        }
    }
    
    node_flow.insert(inlet_node.id, initial_flow_rate);

    while let Some(u) = queue.pop_front() {
        let flow_to_distribute = node_flow.get(&u).cloned().unwrap_or(0.0);
        
        if let Some(neighbors) = adj.get(&u) {
            let num_splits = neighbors.len();
            if num_splits > 0 {
                let outgoing_flow = flow_to_distribute / num_splits as f64;
                for &v in neighbors {
                    if let Some(channel_id) = channel_map.get(&(u, v)) {
                        flow_results.insert(*channel_id, outgoing_flow);
                    }
                    *node_flow.entry(v).or_insert(0.0) += outgoing_flow;
                    
                    if let Some(degree) = in_degree.get_mut(&v) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
        }
    }

    flow_results
}

fn find_inlet_node(nodes: &[Node]) -> Option<&Node> {
    nodes
        .iter()
        .min_by(|a, b| a.point.0.partial_cmp(&b.point.0).unwrap())
} 