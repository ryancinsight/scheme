use crate::geometry::ChannelSystem;
use crate::visualizations::shared_utilities::setup_chart;
use plotters::coord::Shift;
use plotters::prelude::*;
use std::collections::HashMap;

fn flow_to_color(flow: f64, min_flow: f64, max_flow: f64) -> RGBColor {
    if max_flow <= min_flow {
        return BLUE;
    }
    let ratio = (flow - min_flow) / (max_flow - min_flow);
    let r = (255.0 * ratio) as u8;
    let b = (255.0 * (1.0 - ratio)) as u8;
    RGBColor(r, 0, b)
}

fn draw_legend(
    root: &DrawingArea<BitMapBackend, Shift>,
    min_flow: f64,
    max_flow: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = root.dim_in_pixel();
    let legend_width = 150;
    let legend_height = 20;
    let margin = 20;

    let legend_x = (width - legend_width - margin) as i32;
    let legend_y = (height - legend_height - margin) as i32;

    let gradient_points = (0..=legend_width).map(|i| {
        let ratio = i as f64 / legend_width as f64;
        let flow = min_flow + ratio * (max_flow - min_flow);
        let color = flow_to_color(flow, min_flow, max_flow);
        (i, color)
    });

    for (i, color) in gradient_points {
        root.draw(&PathElement::new(
            vec![
                (legend_x + i as i32, legend_y),
                (legend_x + i as i32, legend_y + legend_height as i32),
            ],
            color.filled(),
        ))?;
    }

    root.draw(&Rectangle::new(
        [(legend_x, legend_y), (legend_x + legend_width as i32, legend_y + legend_height as i32)],
        BLACK.stroke_width(1),
    ))?;

    let text_style = TextStyle::from(("sans-serif", 15).into_font()).color(&BLACK);

    root.draw_text(
        &format!("{:.2}", min_flow),
        &text_style,
        (legend_x, legend_y + legend_height as i32 + 5),
    )?;

    root.draw_text(
        &format!("{:.2}", max_flow),
        &text_style,
        (legend_x + legend_width as i32 - 30, legend_y + legend_height as i32 + 5),
    )?;

    Ok(())
}

pub fn plot_cfd_result(
    system: &ChannelSystem,
    output_path: &str,
    flow_data: &HashMap<usize, f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = setup_chart(&root, system.box_dims)?;

    let min_flow = flow_data
        .values()
        .cloned()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    let max_flow = flow_data
        .values()
        .cloned()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);

    let mut line_to_channel_id = HashMap::new();
    for channel in &system.channels {
        let p1 = system.nodes[channel.from_node].point;
        let p2 = system.nodes[channel.to_node].point;
        let key = (
            ((p1.0 * 1e9) as i64, (p1.1 * 1e9) as i64),
            ((p2.0 * 1e9) as i64, (p2.1 * 1e9) as i64),
        );
        line_to_channel_id.insert(key, channel.id);
    }

    for line in &system.lines {
        let key = (
            ((line.0 .0 * 1e9) as i64, (line.0 .1 * 1e9) as i64),
            ((line.1 .0 * 1e9) as i64, (line.1 .1 * 1e9) as i64),
        );
        let mirrored_key = (key.1, key.0);

        let channel_id = line_to_channel_id
            .get(&key)
            .or_else(|| line_to_channel_id.get(&mirrored_key));

        let flow_rate = channel_id
            .and_then(|id| flow_data.get(id))
            .cloned()
            .unwrap_or(0.0);

        let color = flow_to_color(flow_rate, min_flow, max_flow);
        let stroke_width = if max_flow > 0.0 {
            (1.0 + (flow_rate / max_flow) * 4.0) as u32
        } else {
            1
        };

        chart.draw_series(LineSeries::new(
            vec![(line.0 .0, line.0 .1), (line.1 .0, line.1 .1)],
            color.stroke_width(stroke_width),
        ))?;
    }
    
    draw_legend(&root, min_flow, max_flow)?;

    root.present()?;
    println!("CFD plot saved to {}", output_path);
    Ok(())
}