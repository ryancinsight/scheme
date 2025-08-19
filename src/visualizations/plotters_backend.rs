//! visualizations/plotters_backend.rs - Plotters Implementation
//!
//! This module provides a concrete implementation of the visualization traits
//! using the plotters library. This demonstrates how the abstraction allows
//! for different rendering backends while maintaining the same interface.

use crate::geometry::{ChannelSystem, Point2D};
use crate::error::{VisualizationError, VisualizationResult};
use crate::visualizations::traits::{
    SchematicRenderer, GeometricDrawer, VisualizationEngine,
    RenderConfig, OutputFormat, Color, LineStyle, TextStyle
};
use crate::config_constants::ConstantsRegistry;
use plotters::prelude::*;
use plotters::coord::{Shift, types::RangedCoordf64};
use plotters::style::Color as PlottersColor;
use std::path::Path;

/// Plotters-based implementation of the schematic renderer
pub struct PlottersRenderer;

impl SchematicRenderer for PlottersRenderer {
    fn render_system(
        &self,
        system: &ChannelSystem,
        output_path: &str,
        config: &RenderConfig,
    ) -> VisualizationResult<()> {
        // Validate input
        if system.channels.is_empty() && system.nodes.is_empty() {
            return Err(VisualizationError::EmptyChannelSystem);
        }

        self.validate_output_path(output_path)?;

        // Determine output format from file extension
        let output_format = self.detect_output_format(output_path)?;

        match output_format {
            OutputFormat::PNG | OutputFormat::JPEG => {
                self.render_bitmap(system, output_path, config)
            }
            OutputFormat::SVG => {
                self.render_svg(system, output_path, config)
            }
            OutputFormat::PDF => {
                Err(VisualizationError::UnsupportedFormat {
                    format: "PDF".to_string(),
                    message: "PDF output is not yet implemented".to_string(),
                })
            }
        }
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::PNG, OutputFormat::JPEG, OutputFormat::SVG]
    }
}

impl PlottersRenderer {
    /// Detect output format from file extension
    fn detect_output_format(&self, output_path: &str) -> VisualizationResult<OutputFormat> {
        let path = Path::new(output_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| VisualizationError::invalid_output_path(
                output_path,
                "File must have a valid extension"
            ))?;

        match extension.as_str() {
            "png" => Ok(OutputFormat::PNG),
            "jpg" | "jpeg" => Ok(OutputFormat::JPEG),
            "svg" => Ok(OutputFormat::SVG),
            "pdf" => Ok(OutputFormat::PDF),
            _ => Err(VisualizationError::invalid_output_path(
                output_path,
                &format!("Unsupported file extension: .{}", extension)
            ))
        }
    }

    /// Render to bitmap formats (PNG, JPEG)
    fn render_bitmap(
        &self,
        system: &ChannelSystem,
        output_path: &str,
        config: &RenderConfig,
    ) -> VisualizationResult<()> {
        // Create the drawing backend
        let root = BitMapBackend::new(output_path, (config.width, config.height))
            .into_drawing_area();

        root.fill(&convert_color(&config.background_color))
            .map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;

        self.render_with_backend(system, config, root, output_path)
    }

    /// Render to SVG format
    fn render_svg(
        &self,
        system: &ChannelSystem,
        output_path: &str,
        config: &RenderConfig,
    ) -> VisualizationResult<()> {
        // Create the SVG backend
        let root = SVGBackend::new(output_path, (config.width, config.height))
            .into_drawing_area();

        root.fill(&convert_color(&config.background_color))
            .map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;

        self.render_with_backend(system, config, root, output_path)
    }

    /// Common rendering logic for both bitmap and SVG backends
    fn render_with_backend<DB: DrawingBackend>(
        &self,
        system: &ChannelSystem,
        config: &RenderConfig,
        root: DrawingArea<DB, Shift>,
        output_path: &str,
    ) -> VisualizationResult<()> {
        // Set up coordinate system
        let (length, width) = system.box_dims;
        let x_buffer = length * config.margin_fraction;
        let y_buffer = width * config.margin_fraction;

        let constants = ConstantsRegistry::new();
        let mut chart = ChartBuilder::on(&root)
            .caption(&config.title, (config.title_style.font_family.as_str(), config.title_style.font_size as i32))
            .margin(constants.get_default_chart_margin())
            .margin_right(constants.get_default_chart_right_margin())
            .x_label_area_size(constants.get_default_x_label_area_size())
            .y_label_area_size(constants.get_default_y_label_area_size())
            .build_cartesian_2d(
                -x_buffer..length + x_buffer,
                -y_buffer..width + y_buffer,
            )
            .map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;

        if config.show_axes {
            chart
                .configure_mesh()
                .x_desc("X (mm)")
                .y_desc("Y (mm)")
                .draw()
                .map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;
        }

        // Draw channels with type-specific colors
        let (boundary_lines, channel_lines) = system.get_lines_by_type();

        // Draw boundary lines first
        chart.draw_series(
            boundary_lines.iter().map(|(p1, p2)| {
                PathElement::new(
                    vec![*p1, *p2],
                    convert_color(&config.boundary_style.color).stroke_width(config.boundary_style.width as u32)
                )
            })
        ).map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;

        // Draw channels by type with different colors
        for (channel_type, lines) in channel_lines {
            let style = config.channel_type_styles.get_style(channel_type);
            chart.draw_series(
                lines.iter().map(|(p1, p2)| {
                    PathElement::new(
                        vec![*p1, *p2],
                        convert_color(&style.color).stroke_width(style.width as u32)
                    )
                })
            ).map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;
        }

        root.present()
            .map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;

        println!("Schematic plot saved to {}", output_path);
        Ok(())
    }
}

/// Plotters-based implementation of geometric drawing
pub struct PlottersDrawer<'a, DB: DrawingBackend> {
    #[allow(dead_code)] // Reserved for future direct drawing operations
    drawing_area: &'a DrawingArea<DB, Shift>,
    chart: Option<&'a mut ChartContext<'a, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>>,
}

impl<'a, DB: DrawingBackend> PlottersDrawer<'a, DB> {
    /// Create a new PlottersDrawer with the given drawing area and optional chart context
    ///
    /// # Arguments
    ///
    /// * `drawing_area` - The plotters drawing area to draw on
    /// * `chart` - Optional chart context for coordinate transformations
    pub fn new(
        drawing_area: &'a DrawingArea<DB, Shift>,
        chart: Option<&'a mut ChartContext<'a, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>>,
    ) -> Self {
        Self { drawing_area, chart }
    }
}

impl<'a, DB: DrawingBackend> GeometricDrawer for PlottersDrawer<'a, DB> {
    fn draw_line(&mut self, from: Point2D, to: Point2D, style: &LineStyle) -> VisualizationResult<()> {
        if let Some(chart) = &mut self.chart {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![from, to],
                convert_color(&style.color).stroke_width(style.width as u32)
            )))
            .map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;
        }
        Ok(())
    }
    
    fn draw_path(&mut self, points: &[Point2D], style: &LineStyle) -> VisualizationResult<()> {
        if points.len() < 2 {
            return Err(VisualizationError::InvalidParameters {
                parameter: "points".to_string(),
                value: format!("{} points", points.len()),
                constraint: "Path must have at least 2 points".to_string(),
            });
        }

        if let Some(chart) = &mut self.chart {
            // Use slice directly instead of cloning to Vec - zero-copy optimization
            chart.draw_series(std::iter::once(PathElement::new(
                points, // Pass slice directly instead of points.to_vec()
                convert_color(&style.color).stroke_width(style.width as u32)
            )))
            .map_err(|e| VisualizationError::rendering_error(&e.to_string()))?;
        }
        Ok(())
    }
    
    fn draw_rectangle(&mut self, top_left: Point2D, bottom_right: Point2D, style: &LineStyle) -> VisualizationResult<()> {
        let points = vec![
            top_left,
            (bottom_right.0, top_left.1),
            bottom_right,
            (top_left.0, bottom_right.1),
            top_left, // Close the rectangle
        ];
        self.draw_path(&points, style)
    }
    
    fn fill_rectangle(&mut self, _top_left: Point2D, _bottom_right: Point2D, _color: &Color) -> VisualizationResult<()> {
        // For now, just return Ok - filled rectangles would require more complex plotters usage
        Ok(())
    }
    
    fn draw_text(&mut self, _position: Point2D, _text: &str, _style: &TextStyle) -> VisualizationResult<()> {
        // For now, just return Ok - text drawing would require more complex plotters usage
        Ok(())
    }
}

/// Plotters-based implementation of the visualization engine
pub struct PlottersVisualizationEngine<'a, DB: DrawingBackend> {
    drawer: PlottersDrawer<'a, DB>,
}

impl<'a, DB: DrawingBackend> PlottersVisualizationEngine<'a, DB> {
    /// Create a new PlottersVisualizationEngine with the given drawer
    ///
    /// # Arguments
    ///
    /// * `drawer` - The PlottersDrawer to use for rendering operations
    pub fn new(drawer: PlottersDrawer<'a, DB>) -> Self {
        Self { drawer }
    }
}

impl<'a, DB: DrawingBackend> VisualizationEngine for PlottersVisualizationEngine<'a, DB> {
    fn visualize_system(&mut self, system: &ChannelSystem, config: &RenderConfig) -> VisualizationResult<()> {
        self.visualize_boundary(system, &config.boundary_style)?;
        self.visualize_channels(system, &config.channel_style)?;
        Ok(())
    }
    
    fn visualize_channels(&mut self, system: &ChannelSystem, style: &LineStyle) -> VisualizationResult<()> {
        let lines = system.get_lines();
        for (p1, p2) in lines {
            self.drawer.draw_line(p1, p2, style)?;
        }
        Ok(())
    }
    
    fn visualize_boundary(&mut self, system: &ChannelSystem, style: &LineStyle) -> VisualizationResult<()> {
        for &(p1, p2) in &system.box_outline {
            self.drawer.draw_line(p1, p2, style)?;
        }
        Ok(())
    }
    
    fn add_axes(&mut self, _system: &ChannelSystem, _config: &RenderConfig) -> VisualizationResult<()> {
        // Axes are handled by the chart configuration in plotters
        Ok(())
    }
    
    fn add_title(&mut self, _title: &str, _style: &TextStyle) -> VisualizationResult<()> {
        // Title is handled by the chart configuration in plotters
        Ok(())
    }
}

/// Convert our Color type to plotters RGBColor
fn convert_color(color: &Color) -> RGBColor {
    RGBColor(color.r, color.g, color.b)
}

/// Convenience function to create a plotters renderer
pub fn create_plotters_renderer() -> PlottersRenderer {
    PlottersRenderer
}

/// Convenience function for backward compatibility
pub fn plot_geometry_with_plotters(
    system: &ChannelSystem,
    output_path: &str,
) -> VisualizationResult<()> {
    let renderer = PlottersRenderer;
    let config = RenderConfig::default();
    renderer.render_system(system, output_path, &config)
}
