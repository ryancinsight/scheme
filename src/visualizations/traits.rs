//! visualizations/traits.rs - Visualization Abstraction Traits
//!
//! This module defines traits for visualization that abstract away the specific
//! plotting library implementation. This follows the Dependency Inversion Principle
//! by allowing the visualization logic to depend on abstractions rather than
//! concrete implementations.

use crate::geometry::{ChannelSystem, Point2D};
use crate::error::VisualizationResult;

/// Trait for rendering 2D microfluidic schematics
/// 
/// This trait abstracts the rendering backend, allowing different
/// plotting libraries to be used without changing the core visualization logic.
pub trait SchematicRenderer {
    /// Render a complete channel system to the specified output
    /// 
    /// # Arguments
    /// * `system` - The channel system to render
    /// * `output_path` - Path where the rendered output should be saved
    /// * `config` - Configuration for the rendering
    /// 
    /// # Returns
    /// Result indicating success or failure of the rendering operation
    fn render_system(
        &self,
        system: &ChannelSystem,
        output_path: &str,
        config: &RenderConfig,
    ) -> VisualizationResult<()>;
    
    /// Get the supported output formats for this renderer
    fn supported_formats(&self) -> Vec<OutputFormat>;
    
    /// Validate that the output path has a supported format
    fn validate_output_path(&self, path: &str) -> VisualizationResult<()> {
        let formats = self.supported_formats();
        let path_lower = path.to_lowercase();

        for format in &formats {
            if path_lower.ends_with(&format.extension()) {
                return Ok(());
            }
        }

        let supported_extensions: Vec<String> = formats
            .iter()
            .map(|f| format!(".{}", f.extension()))
            .collect();
        
        Err(crate::error::VisualizationError::invalid_output_path(
            path,
            &format!("Unsupported format. Supported formats: {}", supported_extensions.join(", "))
        ))
    }
}

/// Trait for drawing basic geometric primitives
/// 
/// This trait provides a low-level interface for drawing operations
/// that can be implemented by different rendering backends.
pub trait GeometricDrawer {
    /// Draw a line between two points
    fn draw_line(&mut self, from: Point2D, to: Point2D, style: &LineStyle) -> VisualizationResult<()>;
    
    /// Draw a series of connected line segments
    fn draw_path(&mut self, points: &[Point2D], style: &LineStyle) -> VisualizationResult<()>;
    
    /// Draw a rectangle outline
    fn draw_rectangle(&mut self, top_left: Point2D, bottom_right: Point2D, style: &LineStyle) -> VisualizationResult<()>;
    
    /// Fill a rectangle
    fn fill_rectangle(&mut self, top_left: Point2D, bottom_right: Point2D, color: &Color) -> VisualizationResult<()>;
    
    /// Draw text at a specific position
    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> VisualizationResult<()>;
}

/// Configuration for rendering operations
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Width of the output image in pixels
    pub width: u32,
    /// Height of the output image in pixels
    pub height: u32,
    /// Background color
    pub background_color: Color,
    /// Title to display on the schematic
    pub title: String,
    /// Whether to show coordinate axes
    pub show_axes: bool,
    /// Whether to show grid lines
    pub show_grid: bool,
    /// Margin around the content as a fraction of the total size
    pub margin_fraction: f64,
    /// Style for channel lines
    pub channel_style: LineStyle,
    /// Style for boundary box lines
    pub boundary_style: LineStyle,
    /// Style for axis labels
    pub axis_label_style: TextStyle,
    /// Style for the title
    pub title_style: TextStyle,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            background_color: Color::rgb(255, 255, 255), // White
            title: "Channel Schematic".to_string(),
            show_axes: true,
            show_grid: false,
            margin_fraction: 0.05,
            channel_style: LineStyle {
                color: Color::rgb(0, 0, 0), // Black
                width: 1.0,
                dash_pattern: None,
            },
            boundary_style: LineStyle {
                color: Color::rgb(0, 0, 0), // Black
                width: 2.0,
                dash_pattern: None,
            },
            axis_label_style: TextStyle {
                color: Color::rgb(0, 0, 0), // Black
                font_size: 12.0,
                font_family: "sans-serif".to_string(),
            },
            title_style: TextStyle {
                color: Color::rgb(0, 0, 0), // Black
                font_size: 24.0,
                font_family: "sans-serif".to_string(),
            },
        }
    }
}

/// Supported output formats
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    PNG,
    SVG,
    PDF,
    JPEG,
}

impl OutputFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::PNG => "png",
            OutputFormat::SVG => "svg",
            OutputFormat::PDF => "pdf",
            OutputFormat::JPEG => "jpg",
        }
    }
}

/// Color representation
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    
    /// Create a new color with RGB values
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    /// Create a new color with RGBA values
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Style for drawing lines
#[derive(Debug, Clone)]
pub struct LineStyle {
    pub color: Color,
    pub width: f64,
    pub dash_pattern: Option<Vec<f64>>,
}

impl LineStyle {
    /// Create a solid line style
    pub fn solid(color: Color, width: f64) -> Self {
        Self {
            color,
            width,
            dash_pattern: None,
        }
    }
    
    /// Create a dashed line style
    pub fn dashed(color: Color, width: f64, dash_pattern: Vec<f64>) -> Self {
        Self {
            color,
            width,
            dash_pattern: Some(dash_pattern),
        }
    }
}

/// Style for drawing text
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f64,
    pub font_family: String,
}

impl TextStyle {
    /// Create a new text style
    pub fn new(color: Color, font_size: f64, font_family: &str) -> Self {
        Self {
            color,
            font_size,
            font_family: font_family.to_string(),
        }
    }
}

/// High-level visualization operations
/// 
/// This trait provides higher-level operations for visualizing
/// microfluidic systems, built on top of the basic drawing primitives.
pub trait VisualizationEngine {
    /// Visualize a complete channel system
    fn visualize_system(&mut self, system: &ChannelSystem, config: &RenderConfig) -> VisualizationResult<()>;
    
    /// Visualize just the channels without the boundary box
    fn visualize_channels(&mut self, system: &ChannelSystem, style: &LineStyle) -> VisualizationResult<()>;
    
    /// Visualize the boundary box
    fn visualize_boundary(&mut self, system: &ChannelSystem, style: &LineStyle) -> VisualizationResult<()>;
    
    /// Add coordinate axes to the visualization
    fn add_axes(&mut self, system: &ChannelSystem, config: &RenderConfig) -> VisualizationResult<()>;
    
    /// Add a title to the visualization
    fn add_title(&mut self, title: &str, style: &TextStyle) -> VisualizationResult<()>;
}
