use crate::geometry::ChannelSystem;
use crate::visualizations::shared_utilities::visualize;
use plotters::prelude::*;

pub fn plot_geometry(
    system: &ChannelSystem,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_root, mut chart, y_scale) = visualize(system, output_path, "Channel Schematic")?;

    for channel in &system.channels {
        let p1 = system.nodes[channel.from_node].point;
        let p2 = system.nodes[channel.to_node].point;
        let stroke_width = (channel.width * y_scale).round() as u32;
        chart.draw_series(LineSeries::new(
            vec![p1, p2],
            BLACK.stroke_width(stroke_width.max(1)),
        ))?;
    }

    println!("Schematic plot saved to {}", output_path);
    Ok(())
} 