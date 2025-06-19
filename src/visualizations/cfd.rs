use crate::geometry::CfdResults;
use crate::visualizations::shared_utilities::visualize;
use plotters::prelude::*;
use std::collections::HashMap;

pub enum CfdPlotType {
    FlowRate,
    Pressure,
    Resistance,
}

fn plot_property(
    results: &CfdResults,
    output_path: &str,
    plot_type: &CfdPlotType,
) -> Result<(), Box<dyn std::error::Error>> {
    let system = &results.system;

    let (data, title) = match plot_type {
        CfdPlotType::FlowRate => (
            results.channel_flow_rates.clone(),
            "Flow Rate (m^3/s)",
        ),
        CfdPlotType::Pressure => {
            let mut node_pressures_on_channels = HashMap::new();
            for channel in &system.channels {
                if let Some(pressure) = results.node_pressures.get(&channel.from_node) {
                    node_pressures_on_channels.insert(channel.id, *pressure);
                }
            }
            (node_pressures_on_channels, "Pressure (Pa)")
        }
        CfdPlotType::Resistance => (
            results.channel_resistances.clone(),
            "Hydrodynamic Resistance",
        ),
    };

    let (root, mut chart) = visualize(system, output_path, title)?;

    let max_val = data
        .values()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .cloned()
        .unwrap_or(1.0);

    for channel in &system.channels {
        let p1 = system.nodes[channel.from_node].point;
        let p2 = system.nodes[channel.to_node].point;
        let value = data.get(&channel.id).cloned().unwrap_or(0.0);
        let normalized_value = value / max_val;

        let color = HSLColor(240.0 / 360.0 * (1.0 - normalized_value), 0.9, 0.5);

        chart.draw_series(LineSeries::new(vec![p1, p2], color.stroke_width(3)))?;
    }

    // Legend
    let (width, height) = root.dim_in_pixel();
    let legend_x = width as i32 - 130;
    let legend_y = height as i32 / 2 - 100;
    let legend_w = 20;
    let legend_h_seg = 15;

    for i in 0..=10 {
        let ratio = i as f64 / 10.0;
        let color = HSLColor(240.0 / 360.0 * (1.0 - ratio), 0.9, 0.5);
        let y_pos = legend_y + (i * legend_h_seg);
        root.draw(&Rectangle::new(
            [(legend_x, y_pos), (legend_x + legend_w, y_pos + legend_h_seg)],
            color.filled(),
        ))?;

        let text = format!("{:.2e}", max_val * ratio);
        root.draw_text(
            &text,
            &TextStyle::from(("sans-serif", 15).into_font()).color(&BLACK),
            (legend_x + legend_w + 5, y_pos),
        )?;
    }

    root.present()?;
    println!("CFD plot saved to {}", output_path);

    Ok(())
}

pub fn plot_cfd_results(
    results: &CfdResults,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    plot_property(
        results,
        &format!("{}/flow_rate.png", output_dir),
        &CfdPlotType::FlowRate,
    )?;
    plot_property(
        results,
        &format!("{}/pressure.png", output_dir),
        &CfdPlotType::Pressure,
    )?;
    plot_property(
        results,
        &format!("{}/resistance.png", output_dir),
        &CfdPlotType::Resistance,
    )?;
    Ok(())
}