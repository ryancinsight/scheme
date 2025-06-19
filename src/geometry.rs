//! geometry.rs

pub type Point = (f64, f64);

pub struct ChannelSystem {
    pub box_dims: (f64, f64),
    pub lines: Vec<(Point, Point)>,
}

pub enum SplitType {
    Bifurcation,
    Trifurcation,
}

pub fn create_geometry(box_dims: (f64, f64), num_splits: u32, split_type: SplitType) -> ChannelSystem {
    const WALL_CLEARANCE: f64 = 4.0;
    let (length, width) = box_dims;
    let mut lines = Vec::new();

    if num_splits == 0 {
        lines.push(((0.0, width / 2.0), (length, width / 2.0)));
        return ChannelSystem { box_dims, lines };
    }

    let effective_width = width - (2.0 * WALL_CLEARANCE);

    let half_l = length / 2.0;
    let num_segments_per_half = num_splits as f64 * 2.0 + 1.0;
    let dx = half_l / num_segments_per_half;

    let mut y_coords: Vec<f64> = vec![width / 2.0];
    let mut y_ranges: Vec<f64> = vec![effective_width];
    let mut current_x = 0.0;

    let mut first_half_lines = Vec::new();

    for _ in 0..num_splits {
        let mut next_y_coords = Vec::new();
        let mut next_y_ranges = Vec::new();

        for y in &y_coords {
            first_half_lines.push(((current_x, *y), (current_x + dx, *y)));
        }
        current_x += dx;

        for (i, y_center) in y_coords.iter().enumerate() {
            let y_range = y_ranges[i];

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

    let mut second_half_lines = Vec::new();
    for line in &first_half_lines {
        let (p1, p2) = *line;
        let mirrored_p1 = (length - p2.0, p2.1);
        let mirrored_p2 = (length - p1.0, p1.1);
        second_half_lines.push((mirrored_p1, mirrored_p2));
    }

    lines.extend(first_half_lines);
    lines.extend(second_half_lines);

    ChannelSystem { box_dims, lines }
} 