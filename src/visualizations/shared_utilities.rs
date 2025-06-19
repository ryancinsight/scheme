use plotters::prelude::*;
use plotters::coord::{Shift, types::RangedCoordf64};

pub fn setup_chart<'a, 'b>(
    root: &'a DrawingArea<BitMapBackend<'b>, Shift>,
    box_dims: (f64, f64),
) -> Result<
    ChartContext<'a, BitMapBackend<'b>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    Box<dyn std::error::Error>,
> {
    let (length, width) = box_dims;
    let mut chart = ChartBuilder::on(root)
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..length, 0.0..width)?;

    chart.configure_mesh().draw()?;
    Ok(chart)
} 