//! tests/error_tests.rs
//! 
//! Comprehensive tests for the domain-specific error types

use scheme::{
    error::{
        SchemeError, GeometryError, ConfigurationError, VisualizationError, StrategyError,
        SchemeResult, GeometryResult, ConfigurationResult, VisualizationResult, StrategyResult,
    },
    geometry::Point2D,
};
use std::error::Error;

/// Test GeometryError creation and display
#[test]
fn test_geometry_error_creation() {
    let error = GeometryError::invalid_point((f64::NAN, 5.0));
    assert!(error.to_string().contains("Invalid point coordinates"));
    assert!(error.to_string().contains("NaN"));
    
    let error = GeometryError::invalid_box_dimensions(-1.0, 10.0);
    assert!(error.to_string().contains("Invalid box dimensions"));
    assert!(error.to_string().contains("width=-1"));
    
    let error = GeometryError::insufficient_space(100.0, 50.0);
    assert!(error.to_string().contains("Insufficient space"));
    assert!(error.to_string().contains("Required space: 100"));
    assert!(error.to_string().contains("available: 50"));
}

/// Test ConfigurationError creation and display
#[test]
fn test_configuration_error_creation() {
    let error = ConfigurationError::invalid_geometry_config(
        "channel_width", 
        -1.0, 
        "Must be positive"
    );
    assert!(error.to_string().contains("Invalid geometry configuration"));
    assert!(error.to_string().contains("channel_width"));
    assert!(error.to_string().contains("-1"));
    assert!(error.to_string().contains("Must be positive"));
    
    let error = ConfigurationError::invalid_serpentine_config(
        "fill_factor",
        1.5,
        "Must be between 0.1 and 0.95"
    );
    assert!(error.to_string().contains("Invalid serpentine configuration"));
    assert!(error.to_string().contains("fill_factor"));
    assert!(error.to_string().contains("1.5"));
}

/// Test VisualizationError creation and display
#[test]
fn test_visualization_error_creation() {
    let error = VisualizationError::file_error("Permission denied");
    assert!(error.to_string().contains("File I/O error"));
    assert!(error.to_string().contains("Permission denied"));
    
    let error = VisualizationError::invalid_output_path(
        "invalid.xyz",
        "Unsupported format"
    );
    assert!(error.to_string().contains("Invalid output path"));
    assert!(error.to_string().contains("invalid.xyz"));
    assert!(error.to_string().contains("Unsupported format"));
    
    let error = VisualizationError::EmptyChannelSystem;
    assert!(error.to_string().contains("Cannot visualize empty channel system"));
}

/// Test StrategyError creation and display
#[test]
fn test_strategy_error_creation() {
    let error = StrategyError::strategy_creation_failed(
        "CustomStrategy",
        "Invalid parameters"
    );
    assert!(error.to_string().contains("Failed to create strategy"));
    assert!(error.to_string().contains("CustomStrategy"));
    assert!(error.to_string().contains("Invalid parameters"));
    
    let error = StrategyError::execution_failed(
        (0.0, 0.0),
        (10.0, 10.0),
        "Path generation failed"
    );
    assert!(error.to_string().contains("Strategy execution failed"));
    assert!(error.to_string().contains("from (0, 0) to (10, 10)"));
    assert!(error.to_string().contains("Path generation failed"));
}

/// Test SchemeError conversion from other error types
#[test]
fn test_scheme_error_conversion() {
    let geometry_error = GeometryError::invalid_point((f64::INFINITY, 0.0));
    let scheme_error: SchemeError = geometry_error.into();
    assert!(scheme_error.to_string().contains("Geometry error"));
    
    let config_error = ConfigurationError::invalid_geometry_config(
        "test_field",
        999.0,
        "Too large"
    );
    let scheme_error: SchemeError = config_error.into();
    assert!(scheme_error.to_string().contains("Configuration error"));
    
    let viz_error = VisualizationError::EmptyChannelSystem;
    let scheme_error: SchemeError = viz_error.into();
    assert!(scheme_error.to_string().contains("Visualization error"));
}

/// Test result type aliases work correctly
#[test]
fn test_result_type_aliases() {
    fn geometry_operation() -> GeometryResult<i32> {
        Ok(42)
    }
    
    fn config_operation() -> ConfigurationResult<String> {
        Err(ConfigurationError::invalid_geometry_config(
            "test",
            0.0,
            "Invalid"
        ))
    }
    
    fn visualization_operation() -> VisualizationResult<()> {
        Err(VisualizationError::EmptyChannelSystem)
    }
    
    fn strategy_operation() -> StrategyResult<bool> {
        Ok(true)
    }
    
    fn scheme_operation() -> SchemeResult<f64> {
        Err(SchemeError::Geometry(GeometryError::invalid_point((0.0, 0.0))))
    }
    
    assert_eq!(geometry_operation().unwrap(), 42);
    assert!(config_operation().is_err());
    assert!(visualization_operation().is_err());
    assert_eq!(strategy_operation().unwrap(), true);
    assert!(scheme_operation().is_err());
}

/// Test error chaining and source information
#[test]
fn test_error_chaining() {
    let geometry_error = GeometryError::invalid_point((f64::NAN, 0.0));
    let scheme_error = SchemeError::Geometry(geometry_error);
    
    // Test that the error chain is preserved
    assert!(scheme_error.source().is_some());
    
    // Test error display includes nested information
    let error_string = scheme_error.to_string();
    assert!(error_string.contains("Geometry error"));
    assert!(error_string.contains("Invalid point coordinates"));
}

/// Test error helper functions
#[test]
fn test_error_helpers() {
    let point: Point2D = (f64::INFINITY, 5.0);
    let error = GeometryError::invalid_point(point);
    assert!(error.to_string().contains("inf"));
    
    let error = GeometryError::insufficient_space(200.0, 100.0);
    assert!(error.to_string().contains("200"));
    assert!(error.to_string().contains("100"));
    
    let error = VisualizationError::rendering_error("Backend failed");
    assert!(error.to_string().contains("Rendering backend error"));
    assert!(error.to_string().contains("Backend failed"));
}

/// Test that errors implement standard traits
#[test]
fn test_error_traits() {
    let error = GeometryError::invalid_point((0.0, 0.0));
    
    // Test Debug
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("InvalidPoint"));
    
    // Test that error messages are consistent
    let error_msg = error.to_string();
    assert!(!error_msg.is_empty());
    
    // Test Send and Sync (compile-time check)
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GeometryError>();
    assert_send_sync::<ConfigurationError>();
    assert_send_sync::<VisualizationError>();
    assert_send_sync::<StrategyError>();
    assert_send_sync::<SchemeError>();
}

/// Test error context preservation
#[test]
fn test_error_context() {
    let error = GeometryError::NodeCreationFailed {
        x: 10.0,
        y: 20.0,
        reason: "Overlapping with existing node".to_string(),
    };
    
    let error_msg = error.to_string();
    assert!(error_msg.contains("Failed to create node"));
    assert!(error_msg.contains("(10, 20)"));
    assert!(error_msg.contains("Overlapping with existing node"));
    
    let error = GeometryError::ChannelCreationFailed {
        from_id: 1,
        to_id: 5,
        reason: "Invalid path".to_string(),
    };
    
    let error_msg = error.to_string();
    assert!(error_msg.contains("Failed to create channel"));
    assert!(error_msg.contains("from node 1 to node 5"));
    assert!(error_msg.contains("Invalid path"));
}
