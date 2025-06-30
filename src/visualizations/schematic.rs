use crate::geometry::ChannelSystem;
use crate::visualizations::shared_utilities::visualize;
use plotters::prelude::*;

pub fn plot_geometry(
    system: &ChannelSystem,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (root, mut chart, _y_scale) = visualize(system, output_path, "Channel Schematic")?;

    let lines = system.get_lines();

    chart.draw_series(
        lines.iter().map(|(p1, p2)| PathElement::new(vec![*p1, *p2], &BLACK))
    )?;

    root.present()?;

    println!("Schematic plot saved to {output_path}");
    Ok(())
} 