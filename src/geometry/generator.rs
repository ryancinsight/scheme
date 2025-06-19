use super::{Channel, ChannelSystem, Node, Point, SplitType};

pub fn create_geometry(box_dims: (f64, f64), splits: &[SplitType]) -> ChannelSystem {
    const WALL_CLEARANCE: f64 = 4.0;
    const CHANNEL_WIDTH: f64 = 1.0;
    const CHANNEL_HEIGHT: f64 = 1.0;

    let (length, width) = box_dims;
    let mut nodes = Vec::new();
    let mut channels = Vec::new();
    let mut node_counter = 0;
    let mut channel_counter = 0;

    // Using integer keys for HashMap to avoid floating point issues
    let mut point_to_node_id: std::collections::HashMap<(i64, i64), usize> =
        std::collections::HashMap::new();

    fn point_to_key(p: Point) -> (i64, i64) {
        ((p.0 * 1e9) as i64, (p.1 * 1e9) as i64)
    }

    let get_or_create_node = |p: Point,
                              nodes: &mut Vec<Node>,
                              point_to_node_id: &mut std::collections::HashMap<
        (i64, i64),
        usize,
    >,
                              counter: &mut usize|
     -> usize {
        let key = point_to_key(p);
        if let Some(id) = point_to_node_id.get(&key) {
            return *id;
        }
        let id = *counter;
        nodes.push(Node { id, point: p });
        point_to_node_id.insert(key, id);
        *counter += 1;
        id
    };

    let add_channel = |p1: Point,
                       p2: Point,
                       nodes: &mut Vec<Node>,
                       channels: &mut Vec<Channel>,
                       point_to_node_id: &mut std::collections::HashMap<(i64, i64), usize>,
                       node_c: &mut usize,
                       channel_c: &mut usize| {
        let from_id = get_or_create_node(p1, nodes, point_to_node_id, node_c);
        let to_id = get_or_create_node(p2, nodes, point_to_node_id, node_c);
        let id = *channel_c;
        channels.push(Channel {
            id,
            from_node: from_id,
            to_node: to_id,
            width: CHANNEL_WIDTH,
            height: CHANNEL_HEIGHT,
        });
        *channel_c += 1;
    };

    let num_splits = splits.len() as u32;

    if num_splits == 0 {
        let mut lines = Vec::new();
        lines.push(((0.0, width / 2.0), (length, width / 2.0)));
        add_channel(
            (0.0, width / 2.0),
            (length, width / 2.0),
            &mut nodes,
            &mut channels,
            &mut point_to_node_id,
            &mut node_counter,
            &mut channel_counter,
        );
        return ChannelSystem {
            box_dims,
            lines,
            nodes,
            channels,
        };
    }

    let effective_width = width - (2.0 * WALL_CLEARANCE);
    let half_l = length / 2.0;
    let num_segments_per_half = num_splits as f64 * 2.0 + 1.0;
    let dx = half_l / num_segments_per_half;

    let mut y_coords: Vec<f64> = vec![width / 2.0];
    let mut y_ranges: Vec<f64> = vec![effective_width];
    let mut current_x = 0.0;

    let mut first_half_lines = Vec::new();

    for (_, split_type) in splits.iter().enumerate() {
        let mut next_y_coords = Vec::new();
        let mut next_y_ranges = Vec::new();

        for y in &y_coords {
            first_half_lines.push(((current_x, *y), (current_x + dx, *y)));
        }
        current_x += dx;

        for (j, y_center) in y_coords.iter().enumerate() {
            let y_range = y_ranges[j];

            match split_type {
                SplitType::Bifurcation => {
                    let y_separation = y_range / 2.0;
                    let y_upper = y_center + y_separation / 2.0;
                    let y_lower = y_center - y_separation / 2.0;

                    first_half_lines.push(((current_x, *y_center), (current_x + dx, y_upper)));
                    first_half_lines.push(((current_x, *y_center), (current_x + dx, y_lower)));

                    next_y_coords.push(y_upper);
                    next_y_coords.push(y_lower);
                    next_y_ranges.push(y_separation);
                    next_y_ranges.push(y_separation);
                }
                SplitType::Trifurcation => {
                    let y_separation = y_range / 3.0;
                    let y_upper = y_center + y_separation;
                    let y_lower = y_center - y_separation;

                    first_half_lines.push(((current_x, *y_center), (current_x + dx, y_upper)));
                    first_half_lines.push(((current_x, *y_center), (current_x + dx, *y_center)));
                    first_half_lines.push(((current_x, *y_center), (current_x + dx, y_lower)));

                    next_y_coords.push(y_upper);
                    next_y_coords.push(*y_center);
                    next_y_coords.push(y_lower);
                    next_y_ranges.push(y_separation);
                    next_y_ranges.push(y_separation);
                    next_y_ranges.push(y_separation);
                }
            }
        }
        y_coords = next_y_coords;
        y_ranges = next_y_ranges;
        current_x += dx;
    }

    for y in &y_coords {
        first_half_lines.push(((current_x, *y), (half_l, *y)));
    }

    for (p1, p2) in &first_half_lines {
        add_channel(
            *p1,
            *p2,
            &mut nodes,
            &mut channels,
            &mut point_to_node_id,
            &mut node_counter,
            &mut channel_counter,
        );
    }

    for (p1, p2) in &first_half_lines {
        let mirrored_p1 = (length - p2.0, p2.1);
        let mirrored_p2 = (length - p1.0, p1.1);
        add_channel(
            mirrored_p1,
            mirrored_p2,
            &mut nodes,
            &mut channels,
            &mut point_to_node_id,
            &mut node_counter,
            &mut channel_counter,
        );
    }

    let lines = channels
        .iter()
        .map(|c| (nodes[c.from_node].point, nodes[c.to_node].point))
        .collect();

    ChannelSystem {
        box_dims,
        lines,
        nodes,
        channels,
    }
} 