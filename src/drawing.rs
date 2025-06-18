//! drawing.rs

use crate::geometry::ChannelSystem;
use plotters::prelude::*;

pub fn plot_geometry(system: &ChannelSystem, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (box_length, box_width) = system.box_dims;
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Channel Geometry", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..box_length, 0.0..box_width)?;

    chart.configure_mesh().draw()?;

    // Draw the bounding box
    chart.draw_series(LineSeries::new(
        vec![
            (0.0, 0.0),
            (box_length, 0.0),
            (box_length, box_width),
            (0.0, box_width),
            (0.0, 0.0),
        ],
        &BLACK,
    ))?;

    // Draw the channel lines
    for line in &system.lines {
        chart.draw_series(LineSeries::new(vec![line.0, line.1], &RED))?;
    }

    root.present()?;
    println!("Geometry plot saved to {}", output_path);
    Ok(())
} 