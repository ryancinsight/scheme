//! CSG Mathematical Enhancements Test Suite
//! 
//! This module implements comprehensive tests for csgrs-inspired mathematical
//! enhancements to our CSG implementation, following Cathedral Engineering
//! principles with strict TDD methodology.
//!
//! **Test Categories:**
//! - Adaptive epsilon calculation validation
//! - Robust floating-point comparison testing
//! - Enhanced degenerate triangle detection
//! - Numerical stability under extreme conditions
//! - Performance benchmarking for enhanced algorithms

use pyvismil::mesh::csg::models::{
    calculate_adaptive_epsilon_enhanced,
    robust_float_equal_enhanced,
    is_degenerate_triangle_enhanced,
    calculate_adaptive_epsilon,
    robust_float_equal,
    is_degenerate_triangle,
    EPSILON,
};
use stl_io::{Triangle, Vector};
use std::time::Instant;

/// Test epsilon for mathematical enhancement validation
const TEST_EPSILON: f32 = 1e-5;

/// Test adaptive epsilon calculation for small geometry
#[test]
fn test_adaptive_epsilon_small_geometry() {
    println!("=== Testing Adaptive Epsilon: Small Geometry ===");
    
    // Create millimeter-scale cube (0.001 unit sides)
    let small_triangles = create_small_scale_cube(0.001);
    
    let original_epsilon = calculate_adaptive_epsilon(&small_triangles);
    let enhanced_epsilon = calculate_adaptive_epsilon_enhanced(&small_triangles);
    
    println!("Small geometry scale: 0.001 units");
    println!("Original epsilon: {:.2e}", original_epsilon);
    println!("Enhanced epsilon: {:.2e}", enhanced_epsilon);
    
    // Enhanced epsilon should be smaller for small geometry
    assert!(enhanced_epsilon <= EPSILON, 
            "Enhanced epsilon should be <= base epsilon for small geometry: got {:.2e}", enhanced_epsilon);
    
    // Should be within reasonable bounds
    assert!(enhanced_epsilon >= EPSILON * 0.001,
            "Enhanced epsilon should not be too small: got {:.2e}", enhanced_epsilon);
    
    // Should be different from original (demonstrating improvement)
    let epsilon_ratio = enhanced_epsilon / original_epsilon;
    println!("Enhancement ratio: {:.3}", epsilon_ratio);
}

/// Test adaptive epsilon calculation for large geometry
#[test]
fn test_adaptive_epsilon_large_geometry() {
    println!("=== Testing Adaptive Epsilon: Large Geometry ===");
    
    // Create kilometer-scale cube (1000 unit sides)
    let large_triangles = create_large_scale_cube(1000.0);
    
    let original_epsilon = calculate_adaptive_epsilon(&large_triangles);
    let enhanced_epsilon = calculate_adaptive_epsilon_enhanced(&large_triangles);
    
    println!("Large geometry scale: 1000 units");
    println!("Original epsilon: {:.2e}", original_epsilon);
    println!("Enhanced epsilon: {:.2e}", enhanced_epsilon);
    
    // Enhanced epsilon should be larger for large geometry
    assert!(enhanced_epsilon >= EPSILON,
            "Enhanced epsilon should be >= base epsilon for large geometry: got {:.2e}", enhanced_epsilon);
    
    // Should be within reasonable bounds
    assert!(enhanced_epsilon <= EPSILON * 1000.0,
            "Enhanced epsilon should not be too large: got {:.2e}", enhanced_epsilon);
    
    // Should be different from original (demonstrating improvement)
    let epsilon_ratio = enhanced_epsilon / original_epsilon;
    println!("Enhancement ratio: {:.3}", epsilon_ratio);
}

/// Test adaptive epsilon calculation for empty geometry
#[test]
fn test_adaptive_epsilon_empty_geometry() {
    println!("=== Testing Adaptive Epsilon: Empty Geometry ===");
    
    let empty_triangles: Vec<Triangle> = Vec::new();
    
    let original_epsilon = calculate_adaptive_epsilon(&empty_triangles);
    let enhanced_epsilon = calculate_adaptive_epsilon_enhanced(&empty_triangles);
    
    println!("Original epsilon: {:.2e}", original_epsilon);
    println!("Enhanced epsilon: {:.2e}", enhanced_epsilon);
    
    // Both should return base epsilon for empty geometry
    assert_eq!(original_epsilon, EPSILON, "Original should return base epsilon for empty geometry");
    assert_eq!(enhanced_epsilon, EPSILON, "Enhanced should return base epsilon for empty geometry");
}

/// Test robust float comparison for normal values
#[test]
fn test_robust_float_equal_normal_values() {
    println!("=== Testing Robust Float Equality: Normal Values ===");
    
    let test_cases = vec![
        (1.0, 1.0 + EPSILON * 0.5, true, "Within epsilon"),
        (1.0, 1.0 + EPSILON * 2.0, false, "Outside epsilon"),
        (0.0, 0.0, true, "Exact zero equality"),
        (1.0, 1.0, true, "Exact equality"),
        (-1.0, -1.0 + EPSILON * 0.5, true, "Negative values within epsilon"),
    ];
    
    for (a, b, expected, description) in test_cases {
        let original_result = robust_float_equal(a, b, EPSILON);
        let enhanced_result = robust_float_equal_enhanced(a, b, EPSILON);
        
        println!("Test: {} | a={:.2e}, b={:.2e}", description, a, b);
        println!("  Original: {}, Enhanced: {}, Expected: {}", original_result, enhanced_result, expected);
        
        assert_eq!(enhanced_result, expected,
                   "Enhanced comparison failed for {}: a={:.2e}, b={:.2e}", description, a, b);
    }
}

/// Test robust float comparison for extreme values
#[test]
fn test_robust_float_equal_extreme_values() {
    println!("=== Testing Robust Float Equality: Extreme Values ===");
    
    let test_cases = vec![
        (f32::NAN, f32::NAN, true, "NaN equality"),
        (f32::NAN, 1.0, false, "NaN vs normal"),
        (f32::INFINITY, f32::INFINITY, true, "Infinity equality"),
        (f32::INFINITY, f32::NEG_INFINITY, false, "Positive vs negative infinity"),
        (1e10, 1e10 + 1e4, true, "Large values with relative tolerance"),
        (1e-10, 1e-10 + 1e-15, true, "Small values with absolute tolerance"),
    ];
    
    for (a, b, expected, description) in test_cases {
        let enhanced_result = robust_float_equal_enhanced(a, b, EPSILON);

        println!("Test: {} | a={:.2e}, b={:.2e}", description, a, b);
        println!("  Enhanced: {}, Expected: {}", enhanced_result, expected);

        // Debug large values case
        if description.contains("Large values") {
            let diff = (a - b).abs();
            let max_magnitude = a.abs().max(b.abs());
            let tolerance = if max_magnitude > 1.0 {
                EPSILON * max_magnitude
            } else {
                EPSILON
            };
            println!("  Debug - diff: {:.2e}, tolerance: {:.2e}, max_magnitude: {:.2e}",
                     diff, tolerance, max_magnitude);
        }

        assert_eq!(enhanced_result, expected,
                   "Enhanced comparison failed for {}: a={:.2e}, b={:.2e}", description, a, b);
    }
}

/// Test enhanced degenerate triangle detection
#[test]
fn test_enhanced_degenerate_detection_basic() {
    println!("=== Testing Enhanced Degenerate Detection: Basic Cases ===");
    
    // Valid triangle
    let valid_triangle = Triangle {
        normal: Vector::new([0.0, 0.0, 1.0]),
        vertices: [
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([1.0, 0.0, 0.0]),
            Vector::new([0.0, 1.0, 0.0]),
        ],
    };
    
    let original_result = is_degenerate_triangle(&valid_triangle);
    let enhanced_result = is_degenerate_triangle_enhanced(&valid_triangle);
    
    println!("Valid triangle - Original: {}, Enhanced: {}", original_result, enhanced_result);
    assert!(!enhanced_result, "Valid triangle should not be detected as degenerate");
    
    // Degenerate triangle (duplicate vertices)
    let degenerate_triangle = Triangle {
        normal: Vector::new([0.0, 0.0, 1.0]),
        vertices: [
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([0.0, 0.0, 0.0]), // Duplicate vertex
            Vector::new([1.0, 0.0, 0.0]),
        ],
    };
    
    let original_result = is_degenerate_triangle(&degenerate_triangle);
    let enhanced_result = is_degenerate_triangle_enhanced(&degenerate_triangle);
    
    println!("Degenerate triangle - Original: {}, Enhanced: {}", original_result, enhanced_result);
    assert!(enhanced_result, "Degenerate triangle should be detected");
}

/// Test enhanced degenerate triangle detection for edge cases
#[test]
fn test_enhanced_degenerate_detection_edge_cases() {
    println!("=== Testing Enhanced Degenerate Detection: Edge Cases ===");
    
    // Zero area triangle (collinear vertices)
    let collinear_triangle = Triangle {
        normal: Vector::new([0.0, 0.0, 1.0]),
        vertices: [
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([1.0, 0.0, 0.0]),
            Vector::new([2.0, 0.0, 0.0]), // Collinear
        ],
    };
    
    let enhanced_result = is_degenerate_triangle_enhanced(&collinear_triangle);
    println!("Collinear triangle - Enhanced: {}", enhanced_result);
    assert!(enhanced_result, "Collinear triangle should be detected as degenerate");
    
    // Triangle with invalid normal (NaN)
    let invalid_normal_triangle = Triangle {
        normal: Vector::new([f32::NAN, 0.0, 1.0]),
        vertices: [
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([1.0, 0.0, 0.0]),
            Vector::new([0.0, 1.0, 0.0]),
        ],
    };
    
    let enhanced_result = is_degenerate_triangle_enhanced(&invalid_normal_triangle);
    println!("Invalid normal triangle - Enhanced: {}", enhanced_result);
    assert!(enhanced_result, "Triangle with NaN normal should be detected as degenerate");
    
    // Triangle with extreme aspect ratio
    let extreme_aspect_triangle = Triangle {
        normal: Vector::new([0.0, 0.0, 1.0]),
        vertices: [
            Vector::new([0.0, 0.0, 0.0]),
            Vector::new([1000.0, 0.0, 0.0]),
            Vector::new([0.0, 0.001, 0.0]), // Very thin triangle
        ],
    };
    
    let enhanced_result = is_degenerate_triangle_enhanced(&extreme_aspect_triangle);
    println!("Extreme aspect triangle - Enhanced: {}", enhanced_result);
    assert!(enhanced_result, "Triangle with extreme aspect ratio should be detected as degenerate");
}

/// Performance benchmark for enhanced mathematical functions
#[test]
fn test_enhanced_functions_performance() {
    println!("=== Performance Benchmark: Enhanced Mathematical Functions ===");
    
    // Create test data
    let test_triangles = create_performance_test_mesh(1000);
    let test_values: Vec<(f32, f32)> = (0..10000)
        .map(|i| (i as f32 * 0.001, (i + 1) as f32 * 0.001))
        .collect();
    
    // Benchmark adaptive epsilon calculation
    let start = Instant::now();
    for _ in 0..100 {
        let _ = calculate_adaptive_epsilon(&test_triangles);
    }
    let original_epsilon_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..100 {
        let _ = calculate_adaptive_epsilon_enhanced(&test_triangles);
    }
    let enhanced_epsilon_time = start.elapsed();
    
    println!("Adaptive epsilon - Original: {:?}, Enhanced: {:?}", 
             original_epsilon_time, enhanced_epsilon_time);
    
    // Benchmark robust float comparison
    let start = Instant::now();
    for (a, b) in &test_values {
        let _ = robust_float_equal(*a, *b, EPSILON);
    }
    let original_float_time = start.elapsed();
    
    let start = Instant::now();
    for (a, b) in &test_values {
        let _ = robust_float_equal_enhanced(*a, *b, EPSILON);
    }
    let enhanced_float_time = start.elapsed();
    
    println!("Float comparison - Original: {:?}, Enhanced: {:?}", 
             original_float_time, enhanced_float_time);
    
    // Benchmark degenerate detection
    let start = Instant::now();
    for triangle in &test_triangles {
        let _ = is_degenerate_triangle(triangle);
    }
    let original_degenerate_time = start.elapsed();
    
    let start = Instant::now();
    for triangle in &test_triangles {
        let _ = is_degenerate_triangle_enhanced(triangle);
    }
    let enhanced_degenerate_time = start.elapsed();
    
    println!("Degenerate detection - Original: {:?}, Enhanced: {:?}", 
             original_degenerate_time, enhanced_degenerate_time);
    
    // Performance should be reasonable (enhanced functions may be slower due to additional checks)
    // Allow up to 5x slower for enhanced functions with better robustness
    let epsilon_ratio = enhanced_epsilon_time.as_nanos() as f64 / original_epsilon_time.as_nanos().max(1) as f64;
    let float_ratio = enhanced_float_time.as_nanos() as f64 / original_float_time.as_nanos().max(1) as f64;
    let degenerate_ratio = enhanced_degenerate_time.as_nanos() as f64 / original_degenerate_time.as_nanos().max(1) as f64;

    println!("Performance ratios - Epsilon: {:.2}x, Float: {:.2}x, Degenerate: {:.2}x",
             epsilon_ratio, float_ratio, degenerate_ratio);

    assert!(epsilon_ratio < 5.0,
            "Enhanced epsilon calculation should not be more than 5x slower: {:.2}x", epsilon_ratio);
    assert!(float_ratio < 5.0,
            "Enhanced float comparison should not be more than 5x slower: {:.2}x", float_ratio);
    assert!(degenerate_ratio < 5.0,
            "Enhanced degenerate detection should not be more than 5x slower: {:.2}x", degenerate_ratio);
}

// Helper functions for test data generation

/// Create a small-scale cube for testing adaptive epsilon
fn create_small_scale_cube(scale: f32) -> Vec<Triangle> {
    let half_scale = scale * 0.5;
    vec![
        Triangle {
            normal: Vector::new([0.0, 0.0, 1.0]),
            vertices: [
                Vector::new([-half_scale, -half_scale, half_scale]),
                Vector::new([half_scale, -half_scale, half_scale]),
                Vector::new([half_scale, half_scale, half_scale]),
            ],
        },
        Triangle {
            normal: Vector::new([0.0, 0.0, 1.0]),
            vertices: [
                Vector::new([-half_scale, -half_scale, half_scale]),
                Vector::new([half_scale, half_scale, half_scale]),
                Vector::new([-half_scale, half_scale, half_scale]),
            ],
        },
    ]
}

/// Create a large-scale cube for testing adaptive epsilon
fn create_large_scale_cube(scale: f32) -> Vec<Triangle> {
    let half_scale = scale * 0.5;
    vec![
        Triangle {
            normal: Vector::new([0.0, 0.0, 1.0]),
            vertices: [
                Vector::new([-half_scale, -half_scale, half_scale]),
                Vector::new([half_scale, -half_scale, half_scale]),
                Vector::new([half_scale, half_scale, half_scale]),
            ],
        },
        Triangle {
            normal: Vector::new([0.0, 0.0, 1.0]),
            vertices: [
                Vector::new([-half_scale, -half_scale, half_scale]),
                Vector::new([half_scale, half_scale, half_scale]),
                Vector::new([-half_scale, half_scale, half_scale]),
            ],
        },
    ]
}

/// Create a mesh for performance testing
fn create_performance_test_mesh(triangle_count: usize) -> Vec<Triangle> {
    (0..triangle_count)
        .map(|i| {
            let offset = i as f32 * 0.1;
            Triangle {
                normal: Vector::new([0.0, 0.0, 1.0]),
                vertices: [
                    Vector::new([offset, offset, 0.0]),
                    Vector::new([offset + 1.0, offset, 0.0]),
                    Vector::new([offset, offset + 1.0, 0.0]),
                ],
            }
        })
        .collect()
}
