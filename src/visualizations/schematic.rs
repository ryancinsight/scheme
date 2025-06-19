use crate::geometry::ChannelSystem;
use crate::visualizations::shared_utilities::visualize;
use plotters::prelude::*;

pub fn plot_geometry(
    system: &ChannelSystem,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_root, mut chart) = visualize(system, output_path, "Channel Schematic")?;

    for line in &system.lines {
        chart.draw_series(LineSeries::new(
            vec![(line.0.0, line.0.1), (line.1.0, line.1.1)],
            BLACK.stroke_width(2),
        ))?;
    }

    println!("Schematic plot saved to {}", output_path);
    Ok(())
} 