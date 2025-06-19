use crate::geometry::ChannelSystem;
use crate::visualizations::shared_utilities::setup_chart;
use plotters::prelude::*;

pub fn plot_geometry(
    system: &ChannelSystem,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = setup_chart(&root, system.box_dims)?;

    for line in &system.lines {
        chart.draw_series(LineSeries::new(
            vec![(line.0 .0, line.0 .1), (line.1 .0, line.1 .1)],
            BLACK.stroke_width(2),
        ))?;
    }

    root.present()?;
    println!("Schematic plot saved to {}", output_path);
    Ok(())
} 