//! visualizations/mod_3d.rs

use crate::geometry::mod_3d::ChannelSystem3D;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, VPos, Pos};
use std::f64::consts::PI;
use stl_io::Triangle;

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

    // Draw spheres
    for sphere in &system_3d.spheres {
        // Draw longitude lines
        for i in 0..SEGMENTS / 2 {
            let mut points = Vec::with_capacity(SEGMENTS + 1);
            let phi = (i as f64 / (SEGMENTS / 2) as f64) * PI;
            for j in 0..=SEGMENTS {
                let theta = (j as f64 / SEGMENTS as f64) * 2.0 * PI;
                let x = sphere.radius * theta.cos() * phi.sin() + sphere.center.0;
                let y = sphere.radius * theta.sin() * phi.sin() + sphere.center.1;
                let z = sphere.radius * phi.cos() + sphere.center.2;
                points.push((x, z, y));
            }
            chart.draw_series(LineSeries::new(points, &RED))?;
        }

        // Draw latitude lines
        for i in 0..SEGMENTS {
            let mut points = Vec::with_capacity(SEGMENTS + 1);
            let theta = (i as f64 / SEGMENTS as f64) * 2.0 * PI;
            for j in 0..=SEGMENTS {
                let phi = (j as f64 / SEGMENTS as f64) * PI;
                let x = sphere.radius * theta.cos() * phi.sin() + sphere.center.0;
                let y = sphere.radius * theta.sin() * phi.sin() + sphere.center.1;
                let z = sphere.radius * phi.cos() + sphere.center.2;
                points.push((x, z, y));
            }
            chart.draw_series(LineSeries::new(points, &RED))?;
        }
    }

    // Draw cones
    for cone in &system_3d.cones {
        let start = cone.start;
        let end = cone.end;
        let start_radius = cone.start_radius;
        let end_radius = cone.end_radius;

        // Calculate cone axis vector
        let axis = (end.0 - start.0, end.1 - start.1, end.2 - start.2);
        let axis_length = (axis.0 * axis.0 + axis.1 * axis.1 + axis.2 * axis.2).sqrt();

        if axis_length > 0.0 {
            // Normalized axis
            let axis_norm = (axis.0 / axis_length, axis.1 / axis_length, axis.2 / axis_length);

            // Create perpendicular vectors for circular cross-sections
            let perp1 = if axis_norm.0.abs() < 0.9 {
                (1.0, 0.0, 0.0)
            } else {
                (0.0, 1.0, 0.0)
            };

            // Cross product to get second perpendicular vector
            let perp2 = (
                axis_norm.1 * perp1.2 - axis_norm.2 * perp1.1,
                axis_norm.2 * perp1.0 - axis_norm.0 * perp1.2,
                axis_norm.0 * perp1.1 - axis_norm.1 * perp1.0,
            );

            // Normalize perp2
            let perp2_len = (perp2.0 * perp2.0 + perp2.1 * perp2.1 + perp2.2 * perp2.2).sqrt();
            let perp2_norm = (perp2.0 / perp2_len, perp2.1 / perp2_len, perp2.2 / perp2_len);

            // Cross product to get first perpendicular vector
            let perp1_norm = (
                axis_norm.1 * perp2_norm.2 - axis_norm.2 * perp2_norm.1,
                axis_norm.2 * perp2_norm.0 - axis_norm.0 * perp2_norm.2,
                axis_norm.0 * perp2_norm.1 - axis_norm.1 * perp2_norm.0,
            );

            let mut start_vertices = Vec::with_capacity(SEGMENTS);
            let mut end_vertices = Vec::with_capacity(SEGMENTS);

            // Generate circular cross-sections
            for i in 0..SEGMENTS {
                let theta = (i as f64 / SEGMENTS as f64) * 2.0 * PI;
                let cos_theta = theta.cos();
                let sin_theta = theta.sin();

                // Start circle
                let start_offset = (
                    start_radius * (cos_theta * perp1_norm.0 + sin_theta * perp2_norm.0),
                    start_radius * (cos_theta * perp1_norm.1 + sin_theta * perp2_norm.1),
                    start_radius * (cos_theta * perp1_norm.2 + sin_theta * perp2_norm.2),
                );
                start_vertices.push((
                    start.0 + start_offset.0,
                    start.1 + start_offset.1,
                    start.2 + start_offset.2,
                ));

                // End circle
                let end_offset = (
                    end_radius * (cos_theta * perp1_norm.0 + sin_theta * perp2_norm.0),
                    end_radius * (cos_theta * perp1_norm.1 + sin_theta * perp2_norm.1),
                    end_radius * (cos_theta * perp1_norm.2 + sin_theta * perp2_norm.2),
                );
                end_vertices.push((
                    end.0 + end_offset.0,
                    end.1 + end_offset.1,
                    end.2 + end_offset.2,
                ));
            }

            // Draw cone wireframe
            for i in 0..SEGMENTS {
                let j = (i + 1) % SEGMENTS;

                // Start cap
                if start_radius > 0.0 {
                    chart.draw_series(LineSeries::new(
                        vec![(start_vertices[i].0, start_vertices[i].2, start_vertices[i].1),
                             (start_vertices[j].0, start_vertices[j].2, start_vertices[j].1)],
                        &GREEN
                    ))?;
                }

                // End cap
                if end_radius > 0.0 {
                    chart.draw_series(LineSeries::new(
                        vec![(end_vertices[i].0, end_vertices[i].2, end_vertices[i].1),
                             (end_vertices[j].0, end_vertices[j].2, end_vertices[j].1)],
                        &GREEN
                    ))?;
                }

                // Side lines
                chart.draw_series(LineSeries::new(
                    vec![(start_vertices[i].0, start_vertices[i].2, start_vertices[i].1),
                         (end_vertices[i].0, end_vertices[i].2, end_vertices[i].1)],
                    &GREEN
                ))?;
            }
        }
    }

    // Draw tori
    for torus in &system_3d.tori {
        let center = torus.center;
        let major_radius = torus.major_radius;
        let minor_radius = torus.minor_radius;

        // Draw major circles (around the torus)
        for i in 0..SEGMENTS / 4 {
            let mut points = Vec::with_capacity(SEGMENTS + 1);
            let phi = (i as f64 / (SEGMENTS / 4) as f64) * PI;

            for j in 0..=SEGMENTS {
                let theta = (j as f64 / SEGMENTS as f64) * 2.0 * PI;
                let r = major_radius + minor_radius * phi.cos();
                let x = r * theta.cos() + center.0;
                let y = r * theta.sin() + center.1;
                let z = minor_radius * phi.sin() + center.2;
                points.push((x, z, y));
            }
            chart.draw_series(LineSeries::new(points, &MAGENTA))?;
        }

        // Draw minor circles (cross-sections of the tube)
        for i in 0..SEGMENTS {
            let mut points = Vec::with_capacity(SEGMENTS + 1);
            let theta = (i as f64 / SEGMENTS as f64) * 2.0 * PI;
            let circle_center_x = major_radius * theta.cos() + center.0;
            let circle_center_y = major_radius * theta.sin() + center.1;

            for j in 0..=SEGMENTS {
                let phi = (j as f64 / SEGMENTS as f64) * 2.0 * PI;
                let x = circle_center_x + minor_radius * phi.cos() * theta.cos();
                let y = circle_center_y + minor_radius * phi.cos() * theta.sin();
                let z = minor_radius * phi.sin() + center.2;
                points.push((x, z, y));
            }
            chart.draw_series(LineSeries::new(points, &MAGENTA))?;
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

/// Plot a Triangle mesh result directly as a wireframe visualization
///
/// This function visualizes the computed mesh results from CSG operations,
/// displaying the actual triangular mesh structure rather than input shapes.
///
/// # Arguments
/// * `triangles` - The Triangle mesh to visualize
/// * `output_path` - Path where the plot image will be saved
/// * `title` - Title for the plot (e.g., "Intersection Result", "Union Result")
///
/// # Returns
/// * `Ok(())` - Plot saved successfully
/// * `Err(Box<dyn std::error::Error>)` - Error during plotting
///
/// # Example
/// ```rust
/// let result_mesh = intersection(&cube_mesh, &sphere_mesh)?;
/// plot_mesh_result(&result_mesh, "outputs/intersection_result.png", "Intersection Result")?;
/// ```
pub fn plot_mesh_result(
    triangles: &[Triangle],
    output_path: &str,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if triangles.is_empty() {
        return Err("Cannot plot empty mesh".into());
    }

    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    // Calculate bounding box from triangle vertices
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    for triangle in triangles {
        for vertex in &triangle.vertices {
            min_x = min_x.min(vertex[0]);
            max_x = max_x.max(vertex[0]);
            min_y = min_y.min(vertex[1]);
            max_y = max_y.max(vertex[1]);
            min_z = min_z.min(vertex[2]);
            max_z = max_z.max(vertex[2]);
        }
    }

    // Add padding to bounds
    let padding = 2.0;
    let x_range = (min_x - padding) as f64..(max_x + padding) as f64;
    let y_range = (min_y - padding) as f64..(max_y + padding) as f64;
    let z_range = (min_z - padding) as f64..(max_z + padding) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30))
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

    // Draw triangle edges as wireframe
    for triangle in triangles {
        let v1 = triangle.vertices[0];
        let v2 = triangle.vertices[1];
        let v3 = triangle.vertices[2];

        // Draw the three edges of each triangle
        chart.draw_series(LineSeries::new(
            vec![(v1[0] as f64, v1[2] as f64, v1[1] as f64),
                 (v2[0] as f64, v2[2] as f64, v2[1] as f64)],
            &BLUE
        ))?;

        chart.draw_series(LineSeries::new(
            vec![(v2[0] as f64, v2[2] as f64, v2[1] as f64),
                 (v3[0] as f64, v3[2] as f64, v3[1] as f64)],
            &BLUE
        ))?;

        chart.draw_series(LineSeries::new(
            vec![(v3[0] as f64, v3[2] as f64, v3[1] as f64),
                 (v1[0] as f64, v1[2] as f64, v1[1] as f64)],
            &BLUE
        ))?;
    }

    // Draw axis labels
    let coord = chart.as_coord_spec();
    let label_offset = 5.0;

    // X-axis label
    let x_label_pos = coord.translate(&(max_x as f64 + label_offset, 0.0, 0.0));
    root.draw_text(
        "X (mm)",
        &TextStyle::from(("sans-serif", 16)).color(&BLACK).pos(Pos::new(HPos::Center, VPos::Center)),
        x_label_pos,
    )?;

    // Y-axis label
    let y_label_pos = coord.translate(&(0.0, 0.0, max_y as f64 + label_offset));
    root.draw_text(
        "Y (mm)",
        &TextStyle::from(("sans-serif", 16)).color(&BLACK).pos(Pos::new(HPos::Center, VPos::Center)),
        y_label_pos,
    )?;

    // Z-axis label
    let z_label_pos = coord.translate(&(0.0, max_z as f64 + label_offset, 0.0));
    root.draw_text(
        "Z (mm)",
        &TextStyle::from(("sans-serif", 16)).color(&BLACK).pos(Pos::new(HPos::Center, VPos::Center)),
        z_label_pos,
    )?;

    root.present()?;
    println!("Mesh result plot saved to {}", output_path);

    Ok(())
}