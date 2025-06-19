use crate::geometry::ChannelSystem;
use plotters::prelude::*;
use plotters::coord::{Shift, types::RangedCoordf64};

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

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(20)
        .margin_right(150)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..length, 0.0..width)?;

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

    chart.draw_series(std::iter::once(Rectangle::new(
        [(0.0, 0.0), (length, width)],
        BLACK.stroke_width(2),
    )))?;

    Ok((root, chart, y_scale_factor))
} 