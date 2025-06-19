use crate::geometry::ChannelSystem;
use plotters::prelude::*;
use plotters::coord::{Shift, types::RangedCoordf64};
use plotters::element::PathElement;

pub fn visualize<'a, 'b>(
    system: &'a ChannelSystem,
    output_path: &'b str,
    title: &str,
) -> Result<
    (
        DrawingArea<BitMapBackend<'b>, Shift>,
        ChartContext<'a, BitMapBackend<'b>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
        f64,
    ),
    Box<dyn std::error::Error>,
> {
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let (length, width) = system.box_dims;
    let x_buffer = length * 0.05;
    let y_buffer = width * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(20)
        .margin_right(150)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(
            -x_buffer..length + x_buffer,
            -y_buffer..width + y_buffer,
        )?;

    chart
        .configure_mesh()
        .x_desc("X (mm)")
        .y_desc("Y (mm)")
        .draw()?;

    let plotting_area = chart.plotting_area();
    let y_range = chart.y_range();
    let pixel_range = plotting_area.get_pixel_range();
    let y_pixel_height = pixel_range.1.end - pixel_range.1.start;
    let y_data_height = y_range.end - y_range.start;
    let y_scale_factor = y_pixel_height as f64 / y_data_height;

    chart.draw_series(
        system
            .box_outline
            .iter()
            .map(|&(p1, p2)| PathElement::new(vec![p1, p2], BLACK.stroke_width(2))),
    )?;

    Ok((root, chart, y_scale_factor))
} 