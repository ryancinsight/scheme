//! tests/visualization_tests.rs
//! 
//! Comprehensive tests for the visualization abstraction traits

use scheme::{
    geometry::{ChannelSystem, ChannelType, Node, Channel},
    visualizations::{
        traits::{
            SchematicRenderer, RenderConfig, OutputFormat, Color, LineStyle, TextStyle,
        },
        PlottersRenderer, create_plotters_renderer,
    },
    error::{VisualizationError, VisualizationResult},
};

/// Mock renderer for testing the SchematicRenderer trait
struct MockRenderer {
    pub render_called: std::cell::RefCell<bool>,
    pub last_output_path: std::cell::RefCell<String>,
}

impl MockRenderer {
    fn new() -> Self {
        Self {
            render_called: std::cell::RefCell::new(false),
            last_output_path: std::cell::RefCell::new(String::new()),
        }
    }
}

impl SchematicRenderer for MockRenderer {
    fn render_system(
        &self,
        _system: &ChannelSystem,
        output_path: &str,
        _config: &RenderConfig,
    ) -> VisualizationResult<()> {
        *self.render_called.borrow_mut() = true;
        *self.last_output_path.borrow_mut() = output_path.to_string();
        Ok(())
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::PNG, OutputFormat::SVG]
    }
}

/// Create a simple test channel system
fn create_test_system() -> ChannelSystem {
    let nodes = vec![
        Node { id: 0, point: (0.0, 5.0), metadata: None },
        Node { id: 1, point: (10.0, 5.0), metadata: None },
    ];

    let channels = vec![
        Channel {
            id: 0,
            from_node: 0,
            to_node: 1,
            width: 1.0,
            height: 0.5,
            channel_type: ChannelType::Straight,
            metadata: None,
        }
    ];
    
    let box_outline = vec![
        ((0.0, 0.0), (20.0, 0.0)),
        ((20.0, 0.0), (20.0, 10.0)),
        ((20.0, 10.0), (0.0, 10.0)),
        ((0.0, 10.0), (0.0, 0.0)),
    ];
    
    ChannelSystem {
        box_dims: (20.0, 10.0),
        nodes,
        channels,
        box_outline,
    }
}

/// Test RenderConfig default values
#[test]
fn test_render_config_default() {
    let config = RenderConfig::default();
    
    assert_eq!(config.width, 1024);
    assert_eq!(config.height, 768);
    assert_eq!(config.title, "Channel Schematic");
    assert!(config.show_axes);
    assert!(!config.show_grid);
    assert_eq!(config.margin_fraction, 0.05);
    
    // Test color values
    assert_eq!(config.background_color.r, 255);
    assert_eq!(config.background_color.g, 255);
    assert_eq!(config.background_color.b, 255);
    assert_eq!(config.background_color.a, 255);
}

/// Test Color creation and constants
#[test]
fn test_color_creation() {
    let red = Color::rgb(255, 0, 0);
    assert_eq!(red.r, 255);
    assert_eq!(red.g, 0);
    assert_eq!(red.b, 0);
    assert_eq!(red.a, 255);
    
    let transparent_blue = Color::rgba(0, 0, 255, 128);
    assert_eq!(transparent_blue.r, 0);
    assert_eq!(transparent_blue.g, 0);
    assert_eq!(transparent_blue.b, 255);
    assert_eq!(transparent_blue.a, 128);
    
    // Test constants
    assert_eq!(Color::WHITE.r, 255);
    assert_eq!(Color::WHITE.g, 255);
    assert_eq!(Color::WHITE.b, 255);
    
    assert_eq!(Color::BLACK.r, 0);
    assert_eq!(Color::BLACK.g, 0);
    assert_eq!(Color::BLACK.b, 0);
}

/// Test LineStyle creation
#[test]
fn test_line_style_creation() {
    let solid_line = LineStyle::solid(Color::BLACK, 2.0);
    assert_eq!(solid_line.width, 2.0);
    assert!(solid_line.dash_pattern.is_none());
    
    let dashed_line = LineStyle::dashed(Color::RED, 1.0, vec![5.0, 3.0]);
    assert_eq!(dashed_line.width, 1.0);
    assert!(dashed_line.dash_pattern.is_some());
    assert_eq!(dashed_line.dash_pattern.unwrap(), vec![5.0, 3.0]);
}

/// Test TextStyle creation
#[test]
fn test_text_style_creation() {
    let style = TextStyle::new(Color::BLUE, 14.0, "Arial");
    assert_eq!(style.color, Color::BLUE);
    assert_eq!(style.font_size, 14.0);
    assert_eq!(style.font_family, "Arial");
}

/// Test OutputFormat extensions
#[test]
fn test_output_format_extensions() {
    assert_eq!(OutputFormat::PNG.extension(), "png");
    assert_eq!(OutputFormat::SVG.extension(), "svg");
    assert_eq!(OutputFormat::PDF.extension(), "pdf");
    assert_eq!(OutputFormat::JPEG.extension(), "jpg");
}

/// Test SchematicRenderer trait with mock implementation
#[test]
fn test_schematic_renderer_trait() {
    let renderer = MockRenderer::new();
    let system = create_test_system();
    let config = RenderConfig::default();
    
    // Test successful rendering
    let result = renderer.render_system(&system, "test.png", &config);
    assert!(result.is_ok());
    assert!(*renderer.render_called.borrow());
    assert_eq!(*renderer.last_output_path.borrow(), "test.png");
    
    // Test supported formats
    let formats = renderer.supported_formats();
    assert!(formats.contains(&OutputFormat::PNG));
    assert!(formats.contains(&OutputFormat::SVG));
}

/// Test output path validation
#[test]
fn test_output_path_validation() {
    let renderer = MockRenderer::new();
    
    // Test valid paths
    assert!(renderer.validate_output_path("test.png").is_ok());
    assert!(renderer.validate_output_path("output.svg").is_ok());
    assert!(renderer.validate_output_path("PATH/TO/FILE.PNG").is_ok()); // Case insensitive
    
    // Test invalid paths
    assert!(renderer.validate_output_path("test.pdf").is_err());
    assert!(renderer.validate_output_path("test.xyz").is_err());
    assert!(renderer.validate_output_path("test").is_err());
    
    // Test error message contains supported formats
    let error = renderer.validate_output_path("test.pdf").unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains(".png"));
    assert!(error_msg.contains(".svg"));
}

/// Test PlottersRenderer creation and basic functionality
#[test]
fn test_plotters_renderer_creation() {
    let renderer = create_plotters_renderer();
    let formats = renderer.supported_formats();
    
    // PlottersRenderer should support PNG, JPEG, and SVG
    assert!(formats.contains(&OutputFormat::PNG));
    assert!(formats.contains(&OutputFormat::JPEG));
    assert!(formats.contains(&OutputFormat::SVG));

    // Test path validation
    assert!(renderer.validate_output_path("test.png").is_ok());
    assert!(renderer.validate_output_path("test.jpg").is_ok());
    assert!(renderer.validate_output_path("test.svg").is_ok());
    assert!(renderer.validate_output_path("test.pdf").is_err()); // PDF not supported yet
}

/// Test empty channel system handling
#[test]
fn test_empty_channel_system_handling() {
    let renderer = PlottersRenderer;
    let empty_system = ChannelSystem {
        box_dims: (10.0, 10.0),
        nodes: vec![],
        channels: vec![],
        box_outline: vec![],
    };
    let config = RenderConfig::default();
    
    let result = renderer.render_system(&empty_system, "test.png", &config);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        VisualizationError::EmptyChannelSystem => {}, // Expected
        other => panic!("Expected EmptyChannelSystem error, got: {:?}", other),
    }
}

/// Test configuration customization
#[test]
fn test_configuration_customization() {
    let mut config = RenderConfig::default();
    config.width = 800;
    config.height = 600;
    config.title = "Custom Title".to_string();
    config.show_grid = true;
    config.background_color = Color::rgb(240, 240, 240);
    
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert_eq!(config.title, "Custom Title");
    assert!(config.show_grid);
    assert_eq!(config.background_color.r, 240);
}

/// Test that visualization traits are object-safe
#[test]
fn test_trait_object_safety() {
    let renderer = MockRenderer::new();
    let system = create_test_system();
    let config = RenderConfig::default();
    
    // Test that we can use the trait as a trait object
    let renderer_ref: &dyn SchematicRenderer = &renderer;
    let result = renderer_ref.render_system(&system, "test.png", &config);
    assert!(result.is_ok());
    
    // Test that we can box the trait
    let boxed_renderer: Box<dyn SchematicRenderer> = Box::new(MockRenderer::new());
    let result = boxed_renderer.render_system(&system, "test.png", &config);
    assert!(result.is_ok());
}

/// Test error propagation in visualization
#[test]
fn test_error_propagation() {
    struct FailingRenderer;
    
    impl SchematicRenderer for FailingRenderer {
        fn render_system(
            &self,
            _system: &ChannelSystem,
            _output_path: &str,
            _config: &RenderConfig,
        ) -> VisualizationResult<()> {
            Err(VisualizationError::rendering_error("Simulated failure"))
        }
        
        fn supported_formats(&self) -> Vec<OutputFormat> {
            vec![OutputFormat::PNG]
        }
    }
    
    let renderer = FailingRenderer;
    let system = create_test_system();
    let config = RenderConfig::default();
    
    let result = renderer.render_system(&system, "test.png", &config);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Simulated failure"));
}

/// Test color equality and cloning
#[test]
fn test_color_traits() {
    let color1 = Color::rgb(100, 150, 200);
    let color2 = Color::rgb(100, 150, 200);
    let color3 = Color::rgb(100, 150, 201);
    
    assert_eq!(color1, color2);
    assert_ne!(color1, color3);
    
    let cloned_color = color1.clone();
    assert_eq!(color1, cloned_color);
}
