//! visualizations/mod_3d.rs

use crate::geometry::mod_3d::ChannelSystem3D;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, VPos, Pos};
use std::f64::consts::PI;

pub fn plot_3d_system(
    system_3d: &ChannelSystem3D,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let (min_bound, max_bound) = system_3d.get_bounding_box();
    let (x_range, y_range, z_range) = (
        min_bound.0 - 10.0..max_bound.0 + 10.0,
        min_bound.1 - 10.0..max_bound.1 + 10.0,
        min_bound.2 - 10.0..max_bound.2 + 10.0,
    );

    let mut chart = ChartBuilder::on(&root)
        .caption("3D System View", ("sans-serif", 30))
        .build_cartesian_3d(x_range.clone(), z_range.clone(), y_range.clone())?;

    chart.with_projection(|mut pb| {
        pb.pitch = 0.4;
        pb.yaw = 0.7;
        pb.scale = 0.8;
        pb.into_matrix()
    });

    chart
        .configure_axes()
        .light_grid_style(BLACK.mix(0.15))
        .max_light_lines(3)
        .draw()?;

    if system_3d.has_drawable_box() {
        // Draw the box wireframe
        let box_vertices = system_3d.box_volume.get_vertices();
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), // bottom
            (4, 5), (5, 6), (6, 7), (7, 4), // top
            (0, 4), (1, 5), (2, 6), (3, 7), // sides
        ];

        for (start, end) in &edges {
            let p1 = box_vertices[*start];
            let p2 = box_vertices[*end];
            chart.draw_series(LineSeries::new(
                vec![(p1.0, p1.2, p1.1), (p2.0, p2.2, p2.1)],
                &BLACK,
            ))?;
        }
    }

    // Draw cylinders
    const SEGMENTS: usize = 32;
    for cylinder in &system_3d.cylinders {
        let r = cylinder.radius;
        let start_center = cylinder.start;
        let end_center = cylinder.end;

        let mut start_vertices = Vec::with_capacity(SEGMENTS);
        let mut end_vertices = Vec::with_capacity(SEGMENTS);

        for i in 0..SEGMENTS {
            let theta = (i as f64 / SEGMENTS as f64) * 2.0 * PI;
            let y_offset = r * theta.cos();
            let z_offset = r * theta.sin();

            start_vertices.push((start_center.0, start_center.1 + y_offset, start_center.2 + z_offset));
            end_vertices.push((end_center.0, end_center.1 + y_offset, end_center.2 + z_offset));
        }

        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            // Cap lines
            chart.draw_series(LineSeries::new(vec![(start_vertices[i].0, start_vertices[i].2, start_vertices[i].1), (start_vertices[j].0, start_vertices[j].2, start_vertices[j].1)], &BLUE))?;
            chart.draw_series(LineSeries::new(vec![(end_vertices[i].0, end_vertices[i].2, end_vertices[i].1), (end_vertices[j].0, end_vertices[j].2, end_vertices[j].1)], &BLUE))?;
            // Wall lines
            chart.draw_series(LineSeries::new(vec![(start_vertices[i].0, start_vertices[i].2, start_vertices[i].1), (end_vertices[i].0, end_vertices[i].2, end_vertices[i].1)], &BLUE))?;
        }
    }
    
    // Draw axis labels manually by projecting 3D points to 2D screen coordinates.
    let coord = chart.as_coord_spec();
    let (x_max, y_max, z_max) = (max_bound.0, max_bound.1, max_bound.2);
    let label_offset = 10.0;
    
    // X-axis label
    let x_label_pos = coord.translate(&(x_max + label_offset, 0.0, 0.0));
    root.draw_text(
        "X (mm)",
        &TextStyle::from(("sans-serif", 16)).color(&BLACK).pos(Pos::new(HPos::Center, VPos::Center)),
        x_label_pos,
    )?;
    
    // Y-axis label (depth)
    let y_label_pos = coord.translate(&(0.0, 0.0, y_max + label_offset));
    root.draw_text(
        "Y (mm)",
        &TextStyle::from(("sans-serif", 16)).color(&BLACK).pos(Pos::new(HPos::Center, VPos::Center)),
        y_label_pos,
    )?;

    // Z-axis label (vertical)
    let z_label_pos = coord.translate(&(0.0, z_max + label_offset, 0.0));
    root.draw_text(
        "Z (mm)",
        &TextStyle::from(("sans-serif", 16)).color(&BLACK).pos(Pos::new(HPos::Center, VPos::Center)),
        z_label_pos,
    )?;

    root.present()?;
    println!("3D plot saved to {}", output_path);

    Ok(())
} 