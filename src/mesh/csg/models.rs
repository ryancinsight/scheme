//! src/mesh/csg/models.rs
//! 
//! Core Data Structures - The Skeleton of the CSG Chapel
//! 
//! This module defines the fundamental geometric data structures used throughout
//! the CSG system. Following cathedral engineering principles, these structures
//! represent the physical matter and spatial relationships of the CSG domain.

use nalgebra::Vector3;
use std::sync::Arc;

/// Volume conservation tracking for CSG operations
#[derive(Debug, Clone)]
pub struct VolumeTracker {
    pub initial_volume: f32,
    pub current_volume: f32,
    pub operation_history: Vec<String>,
    pub conservation_violations: Vec<String>,
}

impl VolumeTracker {
    /// Create a new volume tracker with initial volume
    pub fn new(initial_volume: f32) -> Self {
        Self {
            initial_volume,
            current_volume: initial_volume,
            operation_history: Vec::new(),
            conservation_violations: Vec::new(),
        }
    }

    /// Record a volume change and check for conservation violations
    pub fn record_operation(&mut self, operation: &str, new_volume: f32) {
        let old_volume = self.current_volume;
        self.current_volume = new_volume;

        let operation_desc = format!("{}: {:.6} → {:.6}", operation, old_volume, new_volume);
        self.operation_history.push(operation_desc);

        // Check for conservation violations
        match operation {
            "subtract" => {
                if new_volume > old_volume + EPSILON {
                    let violation = format!("Subtraction increased volume: {:.6} → {:.6}", old_volume, new_volume);
                    self.conservation_violations.push(violation);
                }
            }
            "union" => {
                // Union should not decrease volume significantly (allowing for numerical precision)
                if new_volume < old_volume - EPSILON {
                    let violation = format!("Union decreased volume: {:.6} → {:.6}", old_volume, new_volume);
                    self.conservation_violations.push(violation);
                }
            }
            "intersection" => {
                if new_volume > old_volume + EPSILON {
                    let violation = format!("Intersection increased volume: {:.6} → {:.6}", old_volume, new_volume);
                    self.conservation_violations.push(violation);
                }
            }
            _ => {}
        }
    }

    /// Check if there are any conservation violations
    pub fn has_violations(&self) -> bool {
        !self.conservation_violations.is_empty()
    }

    /// Get a report of all conservation violations
    pub fn get_violation_report(&self) -> String {
        if self.conservation_violations.is_empty() {
            "No volume conservation violations detected.".to_string()
        } else {
            format!("Volume conservation violations:\n{}", self.conservation_violations.join("\n"))
        }
    }
}

/// Epsilon value for floating-point comparisons in geometric operations
/// Critical mathematical constant for numerical stability in CSG operations
pub const EPSILON: f32 = 1e-5;

/// Calculate adaptive epsilon based on geometry scale for improved numerical stability
///
/// This function computes a context-aware tolerance value based on the bounding box
/// dimensions of the input geometry, providing better numerical stability for both
/// very small and very large geometries.
///
/// # Mathematical Foundation
///
/// The adaptive epsilon is calculated as:
/// ```text
/// adaptive_epsilon = max(EPSILON, scale_factor * EPSILON)
/// where scale_factor = max(bounding_box_dimensions) / reference_scale
/// ```
///
/// # Arguments
/// * `triangles` - Triangle mesh to analyze for scale
///
/// # Returns
/// * Adaptive epsilon value scaled to geometry size
///
/// # Examples
/// ```
/// // Small geometry (millimeter scale) gets smaller epsilon
/// let small_epsilon = calculate_adaptive_epsilon(&small_mesh); // ~1e-8
///
/// // Large geometry (kilometer scale) gets larger epsilon
/// let large_epsilon = calculate_adaptive_epsilon(&large_mesh); // ~1e-2
/// ```
pub fn calculate_adaptive_epsilon(triangles: &[stl_io::Triangle]) -> f32 {
    if triangles.is_empty() {
        return EPSILON;
    }

    // Calculate bounding box
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

    // Calculate maximum dimension
    let max_dimension = (max_x - min_x).max(max_y - min_y).max(max_z - min_z);

    // Reference scale (1.0 unit)
    let reference_scale = 1.0;

    // Scale factor with bounds to prevent extreme values
    let scale_factor = (max_dimension / reference_scale).max(0.001).min(1000.0);

    // Adaptive epsilon with reasonable bounds
    let adaptive_epsilon = EPSILON * scale_factor;
    adaptive_epsilon.max(EPSILON * 0.001).min(EPSILON * 1000.0)
}

/// Robust floating-point equality comparison with adaptive tolerance
///
/// This function provides numerically stable floating-point comparison that
/// handles edge cases like zero values and maintains relative precision.
///
/// # Arguments
/// * `a` - First value to compare
/// * `b` - Second value to compare
/// * `epsilon` - Tolerance for comparison
///
/// # Returns
/// * `true` if values are equal within tolerance, `false` otherwise
///
/// # Mathematical Foundation
/// Uses both absolute and relative tolerance:
/// ```text
/// equal = |a - b| <= max(epsilon, epsilon * max(|a|, |b|))
/// ```
pub fn robust_float_equal(a: f32, b: f32, epsilon: f32) -> bool {
    let diff = (a - b).abs();

    // Handle exact equality (including both zero)
    if diff == 0.0 {
        return true;
    }

    // Use relative tolerance for larger values, absolute for smaller
    let max_magnitude = a.abs().max(b.abs());
    let tolerance = if max_magnitude > 1.0 {
        epsilon * max_magnitude
    } else {
        epsilon
    };

    diff <= tolerance
}

/// Robust floating-point comparison for signed distance calculations
///
/// Specialized comparison for point-plane distance calculations that handles
/// numerical precision issues near plane boundaries.
///
/// # Arguments
/// * `distance` - Signed distance value
/// * `epsilon` - Tolerance for comparison
///
/// # Returns
/// * Classification result as integer: 1 = Front, -1 = Back, 0 = OnPlane
pub fn classify_distance_robust(distance: f32, epsilon: f32) -> i32 {
    if distance > epsilon {
        1  // Front
    } else if distance < -epsilon {
        -1  // Back
    } else {
        0  // OnPlane
    }
}

/// Detect degenerate triangles that should be filtered from CSG operations
///
/// This function identifies triangles that have geometric issues that can cause
/// numerical instability or incorrect results in CSG operations.
///
/// # Degenerate Cases Detected
/// - Zero area triangles (all vertices identical or collinear)
/// - Triangles with duplicate vertices
/// - Triangles with edge lengths below numerical threshold
/// - Triangles with invalid normals (zero length or NaN)
///
/// # Arguments
/// * `triangle` - Triangle to analyze for degeneracy
///
/// # Returns
/// * `true` if triangle is degenerate and should be filtered, `false` otherwise
pub fn is_degenerate_triangle(triangle: &stl_io::Triangle) -> bool {
    let v1 = &triangle.vertices[0];
    let v2 = &triangle.vertices[1];
    let v3 = &triangle.vertices[2];

    // Check for duplicate vertices
    if vertices_equal(v1, v2, EPSILON) || vertices_equal(v2, v3, EPSILON) || vertices_equal(v1, v3, EPSILON) {
        return true;
    }

    // Calculate edge vectors
    let edge1 = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let edge2 = [v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]];

    // Check for very small edges
    let edge1_length_sq = edge1[0] * edge1[0] + edge1[1] * edge1[1] + edge1[2] * edge1[2];
    let edge2_length_sq = edge2[0] * edge2[0] + edge2[1] * edge2[1] + edge2[2] * edge2[2];

    if edge1_length_sq < EPSILON * EPSILON || edge2_length_sq < EPSILON * EPSILON {
        return true;
    }

    // Calculate cross product for area check
    let cross = [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0],
    ];

    // Check for zero area (collinear vertices)
    let cross_magnitude_sq = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
    if cross_magnitude_sq < EPSILON * EPSILON {
        return true;
    }

    // Check for invalid normal
    let normal = &triangle.normal;
    let normal_magnitude_sq = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
    if normal_magnitude_sq < EPSILON * EPSILON || normal[0].is_nan() || normal[1].is_nan() || normal[2].is_nan() {
        return true;
    }

    false
}

/// Enhanced adaptive epsilon calculation based on csgrs insights
///
/// This function implements scale-aware tolerance calculation that adapts to
/// the geometry size, providing better numerical stability for both very small
/// and very large geometries.
///
/// # Arguments
/// * `triangles` - Triangle mesh to analyze for scale
///
/// # Returns
/// * Adaptive epsilon value scaled to geometry size
///
/// # Mathematical Foundation
/// Based on csgrs approach with our constraints:
/// ```text
/// adaptive_epsilon = EPSILON * scale_factor
/// where scale_factor = max(bounding_box_dimensions) / reference_scale
/// ```
///
/// @ENHANCEMENT(REF: CSGRS-001): Adaptive epsilon for scale-aware tolerance
pub fn calculate_adaptive_epsilon_enhanced(triangles: &[stl_io::Triangle]) -> f32 {
    if triangles.is_empty() {
        return EPSILON;
    }

    // Calculate bounding box with improved precision
    let mut min_bounds = [f32::INFINITY; 3];
    let mut max_bounds = [f32::NEG_INFINITY; 3];

    for triangle in triangles {
        for vertex in &triangle.vertices {
            for i in 0..3 {
                min_bounds[i] = min_bounds[i].min(vertex[i]);
                max_bounds[i] = max_bounds[i].max(vertex[i]);
            }
        }
    }

    // Calculate maximum dimension with robust handling
    let dimensions = [
        max_bounds[0] - min_bounds[0],
        max_bounds[1] - min_bounds[1],
        max_bounds[2] - min_bounds[2],
    ];

    let max_dimension = dimensions.iter().fold(0.0f32, |a, &b| a.max(b));

    // Handle degenerate cases
    if max_dimension <= 0.0 || !max_dimension.is_finite() {
        return EPSILON;
    }

    // Reference scale (1.0 unit) with csgrs-inspired scaling
    let reference_scale = 1.0;
    let scale_factor = max_dimension / reference_scale;

    // Apply bounds to prevent extreme values (csgrs approach)
    let bounded_scale_factor = scale_factor.max(0.001).min(1000.0);

    // Calculate adaptive epsilon with safety bounds
    let adaptive_epsilon = EPSILON * bounded_scale_factor;
    adaptive_epsilon.max(EPSILON * 0.001).min(EPSILON * 1000.0)
}

/// Enhanced robust floating-point equality comparison
///
/// This function implements csgrs-inspired robust comparison that handles
/// both relative and absolute tolerance for improved numerical stability.
///
/// # Arguments
/// * `a` - First value to compare
/// * `b` - Second value to compare
/// * `epsilon` - Base tolerance for comparison
///
/// # Returns
/// * `true` if values are equal within tolerance, `false` otherwise
///
/// # Mathematical Foundation
/// Uses both absolute and relative tolerance (csgrs approach):
/// ```text
/// equal = |a - b| <= max(epsilon, epsilon * max(|a|, |b|))
/// ```
///
/// @ENHANCEMENT(REF: CSGRS-002): Robust floating-point equality
pub fn robust_float_equal_enhanced(a: f32, b: f32, epsilon: f32) -> bool {
    // Handle exact equality first (including both zero)
    if a == b {
        return true;
    }

    // Handle NaN cases - NaN is only equal to NaN
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }

    // Handle infinity cases - infinities are equal if same sign
    if a.is_infinite() || b.is_infinite() {
        return a == b; // This handles +inf == +inf and -inf == -inf
    }

    // For finite values, use enhanced tolerance calculation
    let diff = (a - b).abs();
    let max_magnitude = a.abs().max(b.abs());

    // Use relative tolerance for larger values, absolute for smaller (csgrs approach)
    let tolerance = if max_magnitude > 1.0 {
        epsilon * max_magnitude
    } else {
        epsilon
    };

    diff <= tolerance
}

/// Enhanced degenerate triangle detection with comprehensive validation
///
/// This function implements csgrs-inspired comprehensive degenerate detection
/// that identifies multiple types of geometric issues.
///
/// # Arguments
/// * `triangle` - Triangle to analyze for degeneracy
///
/// # Returns
/// * `true` if triangle is degenerate and should be filtered, `false` otherwise
///
/// # Degenerate Cases Detected (csgrs approach)
/// - Zero area triangles (all vertices identical or collinear)
/// - Triangles with duplicate vertices
/// - Triangles with edge lengths below numerical threshold
/// - Triangles with invalid normals (zero length or NaN)
/// - Triangles with extreme aspect ratios
///
/// @ENHANCEMENT(REF: CSGRS-003): Comprehensive degenerate detection
pub fn is_degenerate_triangle_enhanced(triangle: &stl_io::Triangle) -> bool {
    let v1 = &triangle.vertices[0];
    let v2 = &triangle.vertices[1];
    let v3 = &triangle.vertices[2];

    // Enhanced vertex equality check using robust comparison
    if vertices_equal_enhanced(v1, v2, EPSILON) ||
       vertices_equal_enhanced(v2, v3, EPSILON) ||
       vertices_equal_enhanced(v1, v3, EPSILON) {
        return true;
    }

    // Calculate edge vectors with enhanced precision
    let edge1 = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let edge2 = [v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]];

    // Enhanced edge length validation
    let edge1_length_sq = edge1[0] * edge1[0] + edge1[1] * edge1[1] + edge1[2] * edge1[2];
    let edge2_length_sq = edge2[0] * edge2[0] + edge2[1] * edge2[1] + edge2[2] * edge2[2];

    let min_edge_threshold = EPSILON * EPSILON;
    if edge1_length_sq < min_edge_threshold || edge2_length_sq < min_edge_threshold {
        return true;
    }

    // Calculate cross product for area and normal validation
    let cross = [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0],
    ];

    // Enhanced zero area detection (csgrs approach)
    let cross_magnitude_sq = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
    if cross_magnitude_sq < min_edge_threshold {
        return true;
    }

    // Enhanced normal validation
    let normal = &triangle.normal;
    let normal_magnitude_sq = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];

    // Check for invalid normal (zero length, NaN, or infinity)
    if normal_magnitude_sq < min_edge_threshold ||
       !normal[0].is_finite() || !normal[1].is_finite() || !normal[2].is_finite() {
        return true;
    }

    // Enhanced aspect ratio check (csgrs-inspired)
    let edge3_x = v1[0] - v3[0];
    let edge3_y = v1[1] - v3[1];
    let edge3_z = v1[2] - v3[2];
    let edge3_length_sq = edge3_x * edge3_x + edge3_y * edge3_y + edge3_z * edge3_z;

    let max_edge_sq = edge1_length_sq.max(edge2_length_sq).max(edge3_length_sq);
    let min_edge_sq = edge1_length_sq.min(edge2_length_sq).min(edge3_length_sq);

    // Reject triangles with extreme aspect ratios
    if max_edge_sq > min_edge_sq * 1e6 {
        return true;
    }

    false
}

/// Enhanced vertex equality check using robust comparison
fn vertices_equal_enhanced(v1: &stl_io::Vector<f32>, v2: &stl_io::Vector<f32>, epsilon: f32) -> bool {
    robust_float_equal_enhanced(v1[0], v2[0], epsilon) &&
    robust_float_equal_enhanced(v1[1], v2[1], epsilon) &&
    robust_float_equal_enhanced(v1[2], v2[2], epsilon)
}

/// Enhanced vertex interpolation with clamped parametric interpolation
///
/// This function implements csgrs-inspired vertex interpolation that provides
/// improved numerical stability through parameter clamping and robust vertex
/// comparison using enhanced mathematical functions from Phase 1.
///
/// # Arguments
/// * `v1` - First vertex for interpolation
/// * `v2` - Second vertex for interpolation
/// * `t` - Interpolation parameter (will be clamped to [0.0, 1.0])
///
/// # Returns
/// * Interpolated vertex position
///
/// # Mathematical Foundation
/// Uses clamped parametric interpolation (csgrs approach):
/// ```text
/// t_clamped = clamp(t, 0.0, 1.0)
/// result = v1 + t_clamped * (v2 - v1)
/// ```
///
/// # Numerical Stability Improvements
/// - Parameter clamping prevents extrapolation beyond vertex bounds
/// - Uses robust floating-point comparison for vertex validation
/// - Handles edge cases (t=0.0, t=1.0) explicitly for exact results
///
/// @ENHANCEMENT(REF: CSGRS-004): Clamped parametric interpolation
pub fn interpolate_vertex_enhanced(
    v1: &stl_io::Vector<f32>,
    v2: &stl_io::Vector<f32>,
    t: f32
) -> stl_io::Vector<f32> {
    // Handle edge cases explicitly for exact results
    if robust_float_equal_enhanced(t, 0.0, EPSILON) {
        return *v1;
    }
    if robust_float_equal_enhanced(t, 1.0, EPSILON) {
        return *v2;
    }

    // Clamp parameter to valid range [0.0, 1.0] (csgrs approach)
    let t_clamped = t.max(0.0).min(1.0);

    // Enhanced parametric interpolation with improved precision
    stl_io::Vector::new([
        v1[0] + t_clamped * (v2[0] - v1[0]),
        v1[1] + t_clamped * (v2[1] - v1[1]),
        v1[2] + t_clamped * (v2[2] - v1[2]),
    ])
}

/// Enhanced polygon classification with robust geometric predicates
///
/// This function implements csgrs-inspired polygon classification that provides
/// improved numerical stability through adaptive epsilon calculation and robust
/// geometric predicates using enhanced mathematical functions from Phase 1.
///
/// # Arguments
/// * `polygon` - Polygon to classify against the plane
/// * `plane` - Plane to classify the polygon against
///
/// # Returns
/// * Enhanced polygon classification with improved boundary handling
///
/// # Mathematical Foundation
/// Uses adaptive epsilon and robust geometric predicates (csgrs approach):
/// ```text
/// distance = plane.normal.dot(vertex.pos) - plane.w
/// classification = robust_comparison(distance, adaptive_epsilon)
/// ```
///
/// # Numerical Stability Improvements
/// - Adaptive epsilon based on polygon scale for better precision
/// - Robust floating-point comparison for boundary detection
/// - Enhanced handling of near-coplanar cases
/// - Improved spanning detection with minimum threshold
///
/// @ENHANCEMENT(REF: CSGRS-005): Robust geometric predicates
pub fn classify_polygon_enhanced(polygon: &Polygon, plane: &Plane) -> crate::mesh::csg::bsp_tree::PolygonClassification {
    use crate::mesh::csg::bsp_tree::PolygonClassification;

    if polygon.vertices.len() < 3 {
        return PolygonClassification::Coplanar;
    }

    // Performance optimization: use base epsilon for simple cases, adaptive for complex
    let use_adaptive = polygon.vertices.len() > 6; // Only use adaptive for complex polygons
    let epsilon = if use_adaptive {
        let polygon_triangles = polygon_to_triangles(polygon);
        calculate_adaptive_epsilon_enhanced(&polygon_triangles)
    } else {
        EPSILON
    };

    let mut front_count = 0;
    let mut back_count = 0;
    let mut _on_plane_count = 0;

    // Enhanced vertex classification with robust predicates
    for vertex in &polygon.vertices {
        let distance = plane.normal.dot(&vertex.pos) - plane.w;

        // Use robust floating-point comparison for boundary detection
        if robust_float_equal_enhanced(distance, 0.0, epsilon) {
            _on_plane_count += 1;
        } else if distance > epsilon {
            front_count += 1;
        } else if distance < -epsilon {
            back_count += 1;
        }
        // Note: vertices exactly on plane (within epsilon) don't affect classification
    }

    // Enhanced classification logic with improved spanning detection
    if front_count > 0 && back_count > 0 {
        PolygonClassification::Spanning
    } else if front_count > 0 {
        PolygonClassification::Front
    } else if back_count > 0 {
        PolygonClassification::Back
    } else {
        // All vertices are on the plane (within tolerance)
        PolygonClassification::Coplanar
    }
}

/// Helper function to convert polygon to triangles for adaptive epsilon calculation
fn polygon_to_triangles(polygon: &Polygon) -> Vec<stl_io::Triangle> {
    if polygon.vertices.len() < 3 {
        return Vec::new();
    }

    // Simple triangulation: fan from first vertex
    let mut triangles = Vec::new();
    let first_vertex = &polygon.vertices[0];

    for i in 1..polygon.vertices.len() - 1 {
        let second_vertex = &polygon.vertices[i];
        let third_vertex = &polygon.vertices[i + 1];

        // Calculate triangle normal (simplified)
        let edge1 = third_vertex.pos - first_vertex.pos;
        let edge2 = second_vertex.pos - first_vertex.pos;
        let normal = edge1.cross(&edge2).normalize();

        let triangle = stl_io::Triangle {
            normal: stl_io::Vector::new([normal.x, normal.y, normal.z]),
            vertices: [
                stl_io::Vector::new([first_vertex.pos.x, first_vertex.pos.y, first_vertex.pos.z]),
                stl_io::Vector::new([second_vertex.pos.x, second_vertex.pos.y, second_vertex.pos.z]),
                stl_io::Vector::new([third_vertex.pos.x, third_vertex.pos.y, third_vertex.pos.z]),
            ],
        };

        triangles.push(triangle);
    }

    triangles
}

/// Enhanced BSP polygon splitting with performance optimizations
///
/// This function implements csgrs-inspired polygon splitting that provides
/// improved performance through memory pre-allocation, enhanced interpolation
/// integration, and robust polygon construction using enhanced mathematical
/// functions from Phase 1 and Phase 2 Priorities 1-2.
///
/// # Arguments
/// * `plane` - Plane to split the polygon against
/// * `polygon` - Polygon to split
/// * `front` - Vector to store polygons in front of the plane
/// * `back` - Vector to store polygons behind the plane
///
/// # Mathematical Foundation
/// Uses enhanced classification and interpolation (csgrs approach):
/// ```text
/// 1. Classify polygon using enhanced geometric predicates
/// 2. For spanning polygons, split edges using enhanced interpolation
/// 3. Construct result polygons with optimized memory allocation
/// ```
///
/// # Performance Optimizations
/// - Pre-allocate result vectors based on polygon complexity
/// - Use enhanced interpolation for numerical stability
/// - Optimize polygon construction to reduce memory allocations
/// - Integrate with adaptive epsilon for scale-aware precision
///
/// @ENHANCEMENT(REF: CSGRS-006): Performance-optimized BSP operations
pub fn split_polygon_enhanced(
    plane: &Plane,
    polygon: &Polygon,
    front: &mut Vec<Polygon>,
    back: &mut Vec<Polygon>
) {
    use crate::mesh::csg::bsp_tree::PolygonClassification;

    // Handle degenerate polygons early
    if polygon.vertices.len() < 3 {
        return;
    }

    // Use enhanced classification from Priority 2
    let classification = classify_polygon_enhanced(polygon, plane);

    match classification {
        PolygonClassification::Front => {
            // Polygon entirely in front - add to front list
            front.push(polygon.clone());
        },
        PolygonClassification::Back => {
            // Polygon entirely behind - add to back list
            back.push(polygon.clone());
        },
        PolygonClassification::Coplanar => {
            // Polygon on plane - add to front list (standard BSP convention)
            front.push(polygon.clone());
        },
        PolygonClassification::Spanning => {
            // Polygon spans plane - requires splitting
            split_spanning_polygon_enhanced(plane, polygon, front, back);
        }
    }
}

/// Enhanced spanning polygon splitting with optimized vertex interpolation
///
/// This helper function handles the complex case of polygons that span the
/// splitting plane, using enhanced interpolation and robust polygon construction.
///
/// @ENHANCEMENT(REF: CSGRS-006): Performance-optimized spanning polygon splitting
fn split_spanning_polygon_enhanced(
    plane: &Plane,
    polygon: &Polygon,
    front: &mut Vec<Polygon>,
    back: &mut Vec<Polygon>
) {
    // Performance optimization: use base epsilon for simple polygons, adaptive for complex
    let use_adaptive = polygon.vertices.len() > 6;
    let epsilon = if use_adaptive {
        let polygon_triangles = polygon_to_triangles(polygon);
        calculate_adaptive_epsilon_enhanced(&polygon_triangles)
    } else {
        EPSILON
    };

    // Pre-allocate vectors with estimated capacity (performance optimization)
    let estimated_vertices = polygon.vertices.len() + 2;
    let mut front_vertices = Vec::with_capacity(estimated_vertices);
    let mut back_vertices = Vec::with_capacity(estimated_vertices);

    // Cache vertex distances to avoid recalculation (performance optimization)
    let mut vertex_distances: Vec<f32> = Vec::with_capacity(polygon.vertices.len());
    for vertex in &polygon.vertices {
        vertex_distances.push(plane.normal.dot(&vertex.pos) - plane.w);
    }

    // Process each edge of the polygon
    for i in 0..polygon.vertices.len() {
        let current_vertex = &polygon.vertices[i];
        let next_index = (i + 1) % polygon.vertices.len();
        let next_vertex = &polygon.vertices[next_index];

        let current_distance = vertex_distances[i];
        let next_distance = vertex_distances[next_index];

        // Optimized vertex classification (reduced function calls)
        let current_front = current_distance > epsilon;
        let current_back = current_distance < -epsilon;
        let current_on_plane = current_distance.abs() <= epsilon;
        let next_front = next_distance > epsilon;
        let next_back = next_distance < -epsilon;

        // Add current vertex to appropriate lists
        if current_front || current_on_plane {
            front_vertices.push(current_vertex.clone());
        }
        if current_back || current_on_plane {
            back_vertices.push(current_vertex.clone());
        }

        // Check if edge crosses the plane (optimized condition)
        if (current_front && next_back) || (current_back && next_front) {
            // Edge crosses plane - compute intersection
            let total_distance = (current_distance - next_distance).abs();
            if total_distance > epsilon {
                let t = current_distance.abs() / total_distance;

                // Optimized interpolation: direct calculation for simple cases
                let intersection_pos = if use_adaptive {
                    // Use enhanced interpolation for complex polygons
                    interpolate_vertex_enhanced(
                        &stl_io::Vector::new([current_vertex.pos.x, current_vertex.pos.y, current_vertex.pos.z]),
                        &stl_io::Vector::new([next_vertex.pos.x, next_vertex.pos.y, next_vertex.pos.z]),
                        t
                    )
                } else {
                    // Direct interpolation for simple polygons (performance optimization)
                    let t_clamped = t.max(0.0).min(1.0);
                    stl_io::Vector::new([
                        current_vertex.pos.x + t_clamped * (next_vertex.pos.x - current_vertex.pos.x),
                        current_vertex.pos.y + t_clamped * (next_vertex.pos.y - current_vertex.pos.y),
                        current_vertex.pos.z + t_clamped * (next_vertex.pos.z - current_vertex.pos.z),
                    ])
                };

                // Optimized normal interpolation
                let intersection_normal = if use_adaptive {
                    interpolate_vertex_enhanced(
                        &stl_io::Vector::new([current_vertex.normal.x, current_vertex.normal.y, current_vertex.normal.z]),
                        &stl_io::Vector::new([next_vertex.normal.x, next_vertex.normal.y, next_vertex.normal.z]),
                        t
                    )
                } else {
                    let t_clamped = t.max(0.0).min(1.0);
                    stl_io::Vector::new([
                        current_vertex.normal.x + t_clamped * (next_vertex.normal.x - current_vertex.normal.x),
                        current_vertex.normal.y + t_clamped * (next_vertex.normal.y - current_vertex.normal.y),
                        current_vertex.normal.z + t_clamped * (next_vertex.normal.z - current_vertex.normal.z),
                    ])
                };

                let intersection_vertex = Vertex::new(
                    nalgebra::Vector3::new(intersection_pos[0], intersection_pos[1], intersection_pos[2]),
                    nalgebra::Vector3::new(intersection_normal[0], intersection_normal[1], intersection_normal[2])
                );

                // Add intersection vertex to both front and back
                front_vertices.push(intersection_vertex.clone());
                back_vertices.push(intersection_vertex);
            }
        }
    }

    // Create result polygons if they have enough vertices
    if front_vertices.len() >= 3 {
        let shared = std::sync::Arc::new(PolygonShared::default());
        front.push(Polygon::new(front_vertices, shared));
    }

    if back_vertices.len() >= 3 {
        let shared = std::sync::Arc::new(PolygonShared::default());
        back.push(Polygon::new(back_vertices, shared));
    }
}

/// Check if two vertices are equal within tolerance (original implementation)
fn vertices_equal(v1: &stl_io::Vector<f32>, v2: &stl_io::Vector<f32>, epsilon: f32) -> bool {
    robust_float_equal(v1[0], v2[0], epsilon) &&
    robust_float_equal(v1[1], v2[1], epsilon) &&
    robust_float_equal(v1[2], v2[2], epsilon)
}

/// Filter degenerate triangles from a mesh
///
/// This function removes triangles that are identified as degenerate to improve
/// numerical stability and prevent CSG operation failures.
///
/// # Arguments
/// * `triangles` - Input triangle mesh
///
/// # Returns
/// * Filtered mesh with degenerate triangles removed
pub fn filter_degenerate_triangles(triangles: &[stl_io::Triangle]) -> Vec<stl_io::Triangle> {
    triangles.iter()
        .filter(|triangle| !is_degenerate_triangle(triangle))
        .cloned()
        .collect()
}

/// Validate triangle mesh for CSG operations
///
/// This function performs comprehensive validation of a triangle mesh to ensure
/// it's suitable for CSG operations, reporting any issues found.
///
/// # Arguments
/// * `triangles` - Triangle mesh to validate
///
/// # Returns
/// * `Result` with validation summary or error details
pub fn validate_mesh_for_csg(triangles: &[stl_io::Triangle]) -> Result<MeshValidationReport, String> {
    let mut report = MeshValidationReport::new();

    if triangles.is_empty() {
        return Err("Empty mesh cannot be used for CSG operations".to_string());
    }

    for (i, triangle) in triangles.iter().enumerate() {
        if is_degenerate_triangle(triangle) {
            report.degenerate_triangles.push(i);
        }
    }

    report.total_triangles = triangles.len();
    report.valid_triangles = triangles.len() - report.degenerate_triangles.len();

    Ok(report)
}

/// Mesh validation report for CSG operations
#[derive(Debug, Clone)]
pub struct MeshValidationReport {
    pub total_triangles: usize,
    pub valid_triangles: usize,
    pub degenerate_triangles: Vec<usize>,
}

impl MeshValidationReport {
    pub fn new() -> Self {
        Self {
            total_triangles: 0,
            valid_triangles: 0,
            degenerate_triangles: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.degenerate_triangles.is_empty()
    }

    pub fn degenerate_ratio(&self) -> f32 {
        if self.total_triangles == 0 {
            0.0
        } else {
            self.degenerate_triangles.len() as f32 / self.total_triangles as f32
        }
    }
}

/// A vertex in 3D space with position and normal vector
/// 
/// The Vertex represents a point in space along with its surface normal,
/// providing the fundamental building block for all geometric operations.
#[derive(Clone, Debug, PartialEq)]
pub struct Vertex {
    /// 3D position of the vertex
    pub pos: Vector3<f32>,
    /// Surface normal at this vertex
    pub normal: Vector3<f32>,
}

impl Vertex {
    /// Create a new vertex with the given position and normal
    /// 
    /// # Arguments
    /// * `pos` - 3D position vector
    /// * `normal` - Surface normal vector (should be normalized)
    pub fn new(pos: Vector3<f32>, normal: Vector3<f32>) -> Self {
        Self { pos, normal }
    }

    /// Flip the vertex normal (invert direction)
    /// 
    /// This operation is used when reversing polygon winding order
    /// during CSG operations.
    pub fn flip(&mut self) {
        self.normal = -self.normal;
    }

    /// Linear interpolation between this vertex and another
    /// 
    /// Creates a new vertex at parameter t between self (t=0) and other (t=1).
    /// Both position and normal are interpolated and the normal is normalized.
    /// 
    /// # Arguments
    /// * `other` - The target vertex to interpolate towards
    /// * `t` - Interpolation parameter [0.0, 1.0]
    /// 
    /// # Returns
    /// * New interpolated vertex
    /// 
    pub fn interpolate(&self, other: &Vertex, t: f32) -> Vertex {
        let interpolated_pos = self.pos.lerp(&other.pos, t);
        let interpolated_normal = self.normal.lerp(&other.normal, t);

        // Handle edge case where interpolated normal has zero length
        let normal_magnitude = interpolated_normal.magnitude();
        let final_normal = if normal_magnitude < EPSILON {
            // Fallback to one of the original normals if interpolation results in zero vector
            if t < 0.5 { self.normal } else { other.normal }
        } else {
            interpolated_normal / normal_magnitude // Manual normalization for better control
        };

        Vertex {
            pos: interpolated_pos,
            normal: final_normal,
        }
    }
}

/// A plane in 3D space defined by normal vector and distance
/// 
/// The plane equation is: normal · point = w
/// Points with normal · point > w are in front of the plane,
/// points with normal · point < w are behind the plane.
#[derive(Clone, Debug)]
pub struct Plane {
    /// Unit normal vector of the plane
    pub normal: Vector3<f32>,
    /// Distance from origin along normal (normal · point_on_plane)
    pub w: f32,
}

impl Plane {
    /// Create a new plane with the given normal and distance
    /// 
    /// # Arguments
    /// * `normal` - Unit normal vector of the plane
    /// * `w` - Distance from origin along normal
    pub fn new(normal: Vector3<f32>, w: f32) -> Self {
        Self { normal, w }
    }

    /// Create a plane from three points
    /// 
    /// Constructs a plane passing through the three given points.
    /// The normal is computed using the cross product of edge vectors.
    /// 
    /// # Arguments
    /// * `a` - First point on the plane
    /// * `b` - Second point on the plane  
    /// * `c` - Third point on the plane
    /// 
    /// # Returns
    /// * New plane passing through the three points
    /// 
    pub fn from_points(a: &Vector3<f32>, b: &Vector3<f32>, c: &Vector3<f32>) -> Self {
        let edge1 = b - a;
        let edge2 = c - a;
        let cross = edge1.cross(&edge2);

        // Check for degenerate case (coplanar points or zero-area triangle)
        let cross_magnitude = cross.magnitude();
        if cross_magnitude < EPSILON {
            // Fallback to a default plane (XY plane) for degenerate cases
            // In a production system, this might return a Result instead
            return Self {
                normal: Vector3::new(0.0, 0.0, 1.0),
                w: 0.0,
            };
        }

        let normal = cross / cross_magnitude; // Manual normalization for better control
        let w = normal.dot(a);
        Self { normal, w }
    }

    /// Flip the plane (invert normal and distance)
    ///
    /// This operation reverses the plane's orientation, swapping
    /// front and back sides.
    pub fn flip(&mut self) {
        self.normal = -self.normal;
        self.w = -self.w;
    }

    /// Split a polygon by this plane into front, back, and coplanar parts
    ///
    /// This method implements the core polygon splitting algorithm used in BSP tree
    /// construction and CSG operations. It classifies each vertex of the polygon
    /// against the plane and handles spanning polygons by creating intersection vertices.
    ///
    /// # Algorithm
    /// 1. **Vertex Classification**: Each vertex is classified as Front, Back, or Coplanar
    /// 2. **Polygon Classification**: Based on vertex classifications
    /// 3. **Intersection Calculation**: For spanning polygons, compute edge-plane intersections
    /// 4. **Polygon Construction**: Build new polygons on each side of the plane
    ///
    /// # Arguments
    /// * `polygon` - The polygon to split
    /// * `co_planar_front` - Output vector for coplanar polygons facing same direction as plane
    /// * `co_planar_back` - Output vector for coplanar polygons facing opposite direction
    /// * `front` - Output vector for polygons entirely in front of plane
    /// * `back` - Output vector for polygons entirely behind plane
    ///
    /// # Mathematical Precision
    /// Uses parametric line-plane intersection: `t = (plane.w - plane.normal.dot(v1)) / plane.normal.dot(v2 - v1)`
    /// with EPSILON = 1e-5 for numerical stability.
    pub fn split_polygon(
        &self,
        polygon: &Polygon,
        co_planar_front: &mut Vec<Polygon>,
        co_planar_back: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        #[derive(PartialEq, Eq)]
        enum PointType {
            Coplanar,
            Front,
            Back,
        }

        let mut polygon_type = 0;
        let mut point_types = Vec::new();

        // Classify each vertex against the plane
        for vertex in &polygon.vertices {
            let signed_distance = self.normal.dot(&vertex.pos) - self.w;
            let point_type = if signed_distance < -EPSILON {
                PointType::Back
            } else if signed_distance > EPSILON {
                PointType::Front
            } else {
                PointType::Coplanar
            };

            // Build polygon classification bitmask
            polygon_type |= match point_type {
                PointType::Coplanar => 0,
                PointType::Front => 1,
                PointType::Back => 2,
            };
            point_types.push(point_type);
        }

        // Handle polygon based on classification
        match polygon_type {
            0 => {
                // All vertices coplanar - determine orientation
                if self.normal.dot(&polygon.plane.normal) > 0.0 {
                    co_planar_front.push(polygon.clone());
                } else {
                    co_planar_back.push(polygon.clone());
                }
            }
            1 => {
                // All vertices in front
                front.push(polygon.clone());
            }
            2 => {
                // All vertices behind
                back.push(polygon.clone());
            }
            3 => {
                // Spanning polygon - split using parametric intersection
                let mut front_vertices = Vec::new();
                let mut back_vertices = Vec::new();

                for i in 0..polygon.vertices.len() {
                    let j = (i + 1) % polygon.vertices.len();
                    let current_type = &point_types[i];
                    let next_type = &point_types[j];
                    let current_vertex = &polygon.vertices[i];
                    let next_vertex = &polygon.vertices[j];

                    // Add current vertex to appropriate side(s)
                    if *current_type != PointType::Back {
                        front_vertices.push(current_vertex.clone());
                    }
                    if *current_type != PointType::Front {
                        back_vertices.push(current_vertex.clone());
                    }

                    // Check for edge-plane intersection
                    if (*current_type == PointType::Front && *next_type == PointType::Back)
                        || (*current_type == PointType::Back && *next_type == PointType::Front)
                    {
                        // Calculate parametric intersection using exact formula
                        let edge_direction = next_vertex.pos - current_vertex.pos;
                        let denominator = self.normal.dot(&edge_direction);

                        if denominator.abs() > EPSILON {
                            let t = (self.w - self.normal.dot(&current_vertex.pos)) / denominator;

                            // Clamp t to [0,1] for numerical stability
                            let t_clamped = t.max(0.0).min(1.0);
                            let intersection_vertex = current_vertex.interpolate(next_vertex, t_clamped);

                            front_vertices.push(intersection_vertex.clone());
                            back_vertices.push(intersection_vertex);
                        }
                    }
                }

                // Create polygons from collected vertices (minimum 3 vertices required)
                if front_vertices.len() >= 3 {
                    front.push(Polygon::new(front_vertices, polygon.shared.clone()));
                }
                if back_vertices.len() >= 3 {
                    back.push(Polygon::new(back_vertices, polygon.shared.clone()));
                }
            }
            _ => {
                // Should not happen with 2-bit classification
            }
        }
    }
}

/// Shared properties for polygons (material, texture, etc.)
///
/// Using Arc for efficient sharing of properties across multiple polygons.
/// Currently empty but designed for future extension with material properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolygonShared {
    // Future: material_id, texture_coords, color, etc.
}

/// A polygon in 3D space with vertices, shared properties, and plane
/// 
/// The Polygon represents a planar face with an ordered list of vertices.
/// All vertices must be coplanar and the polygon must be convex for
/// proper CSG operations.
#[derive(Clone, Debug)]
pub struct Polygon {
    /// Ordered vertices defining the polygon boundary
    pub vertices: Vec<Vertex>,
    /// Shared properties (material, texture, etc.)
    pub shared: Arc<PolygonShared>,
    /// Plane containing this polygon
    pub plane: Plane,
}

impl Polygon {
    /// Create a new polygon from vertices and shared properties
    ///
    /// The plane is automatically computed from the first three vertices using
    /// the cross product of edge vectors. The polygon must be convex and all
    /// vertices should be coplanar for proper CSG operations.
    ///
    /// # Arguments
    /// * `vertices` - Ordered list of vertices (minimum 3, should be coplanar)
    /// * `shared` - Shared properties for this polygon
    ///
    /// # Returns
    /// * New polygon with computed plane
    ///
    /// # Panics
    /// * If fewer than 3 vertices are provided
    ///
    /// # Mathematical Notes
    /// * Plane normal is computed as (v1-v0) × (v2-v0) normalized
    /// * Plane distance w is computed as normal · v0
    /// * Degenerate triangles (zero area) are handled gracefully
    pub fn new(vertices: Vec<Vertex>, shared: Arc<PolygonShared>) -> Self {
        if vertices.len() < 3 {
            panic!("Polygon requires at least 3 vertices, got {}", vertices.len());
        }

        // Compute plane from first three vertices
        let plane = Plane::from_points(
            &vertices[0].pos,
            &vertices[1].pos,
            &vertices[2].pos,
        );

        // Validate that the computed plane normal has reasonable magnitude
        // (This catches degenerate cases where from_points returns a fallback plane)
        let normal_magnitude = plane.normal.magnitude();
        if (normal_magnitude - 1.0).abs() > EPSILON {
            // This should rarely happen due to the fallback in from_points,
            // but provides an additional safety check
            eprintln!("Warning: Polygon plane normal magnitude is {}, expected ~1.0", normal_magnitude);
        }

        Self { vertices, shared, plane }
    }

    /// Flip the polygon (reverse winding order and normals)
    ///
    /// This operation reverses the polygon's orientation by:
    /// 1. Reversing the vertex order
    /// 2. Flipping all vertex normals
    /// 3. Flipping the plane normal
    ///
    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.flip();
        }
        self.plane.flip();
    }

    /// Calculate the volume contribution of this polygon using the divergence theorem
    ///
    /// This method computes the signed volume contribution of the polygon to the overall
    /// mesh volume using the divergence theorem. The sign depends on the polygon orientation.
    ///
    /// # Mathematical Formula
    /// For a triangle with vertices v1, v2, v3:
    /// volume_contribution = (centroid · normal) / 6
    /// where centroid = (v1 + v2 + v3) / 3 and normal = (v2-v1) × (v3-v1)
    ///
    /// # Returns
    /// * Signed volume contribution (positive for outward-facing polygons)
    pub fn volume_contribution(&self) -> f32 {
        if self.vertices.len() < 3 {
            return 0.0;
        }

        // For triangular polygons, use the standard formula
        if self.vertices.len() == 3 {
            let v1 = &self.vertices[0].pos;
            let v2 = &self.vertices[1].pos;
            let v3 = &self.vertices[2].pos;

            // Calculate triangle centroid
            let centroid = (v1 + v2 + v3) / 3.0;

            // Calculate triangle normal using cross product
            let edge1 = v2 - v1;
            let edge2 = v3 - v1;
            let normal = edge1.cross(&edge2);

            // Volume contribution using divergence theorem
            centroid.dot(&normal) / 6.0
        } else {
            // For polygons with more than 3 vertices, triangulate and sum contributions
            let mut total_volume = 0.0;
            for i in 1..self.vertices.len() - 1 {
                let v1 = &self.vertices[0].pos;
                let v2 = &self.vertices[i].pos;
                let v3 = &self.vertices[i + 1].pos;

                let centroid = (v1 + v2 + v3) / 3.0;
                let edge1 = v2 - v1;
                let edge2 = v3 - v1;
                let normal = edge1.cross(&edge2);

                total_volume += centroid.dot(&normal) / 6.0;
            }
            total_volume
        }
    }

    /// Calculate the area of this polygon
    ///
    /// # Returns
    /// * Area of the polygon (always positive)
    pub fn area(&self) -> f32 {
        if self.vertices.len() < 3 {
            return 0.0;
        }

        if self.vertices.len() == 3 {
            let v1 = &self.vertices[0].pos;
            let v2 = &self.vertices[1].pos;
            let v3 = &self.vertices[2].pos;

            let edge1 = v2 - v1;
            let edge2 = v3 - v1;
            edge1.cross(&edge2).magnitude() / 2.0
        } else {
            // For polygons with more than 3 vertices, triangulate and sum areas
            let mut total_area = 0.0;
            for i in 1..self.vertices.len() - 1 {
                let v1 = &self.vertices[0].pos;
                let v2 = &self.vertices[i].pos;
                let v3 = &self.vertices[i + 1].pos;

                let edge1 = v2 - v1;
                let edge2 = v3 - v1;
                total_area += edge1.cross(&edge2).magnitude() / 2.0;
            }
            total_area
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;
    use std::sync::Arc;

    const TEST_EPSILON: f32 = 1e-5;

    // ===== VERTEX TESTS =====

    #[test]
    fn test_vertex_interpolate_midpoint() {
        let v1 = Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let v2 = Vertex::new(Vector3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

        let result = v1.interpolate(&v2, 0.5);

        // Expected position: midpoint (1.0, 0.0, 0.0)
        let expected_pos = Vector3::new(1.0, 0.0, 0.0);
        assert!((result.pos - expected_pos).magnitude() < TEST_EPSILON,
                "Position interpolation failed: expected {:?}, got {:?}", expected_pos, result.pos);

        // Expected normal: normalized lerp of (1,0,0) and (0,1,0) = normalized (0.5,0.5,0) = (0.707..., 0.707..., 0.0)
        let expected_normal_magnitude = 1.0;
        assert!((result.normal.magnitude() - expected_normal_magnitude).abs() < TEST_EPSILON,
                "Normal should be normalized: expected magnitude 1.0, got {}", result.normal.magnitude());

        let expected_normal_x = 0.7071067811865476; // sqrt(2)/2
        let expected_normal_y = 0.7071067811865476; // sqrt(2)/2
        assert!((result.normal.x - expected_normal_x).abs() < TEST_EPSILON,
                "Normal X component failed: expected {}, got {}", expected_normal_x, result.normal.x);
        assert!((result.normal.y - expected_normal_y).abs() < TEST_EPSILON,
                "Normal Y component failed: expected {}, got {}", expected_normal_y, result.normal.y);
        assert!(result.normal.z.abs() < TEST_EPSILON,
                "Normal Z component should be 0: got {}", result.normal.z);
    }

    #[test]
    fn test_vertex_interpolate_endpoints() {
        let v1 = Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let v2 = Vertex::new(Vector3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

        // Test t=0.0 (should return v1)
        let result_0 = v1.interpolate(&v2, 0.0);
        assert!((result_0.pos - v1.pos).magnitude() < TEST_EPSILON,
                "t=0.0 position failed: expected {:?}, got {:?}", v1.pos, result_0.pos);
        assert!((result_0.normal - v1.normal).magnitude() < TEST_EPSILON,
                "t=0.0 normal failed: expected {:?}, got {:?}", v1.normal, result_0.normal);

        // Test t=1.0 (should return v2)
        let result_1 = v1.interpolate(&v2, 1.0);
        assert!((result_1.pos - v2.pos).magnitude() < TEST_EPSILON,
                "t=1.0 position failed: expected {:?}, got {:?}", v2.pos, result_1.pos);
        assert!((result_1.normal - v2.normal).magnitude() < TEST_EPSILON,
                "t=1.0 normal failed: expected {:?}, got {:?}", v2.normal, result_1.normal);
    }

    #[test]
    fn test_vertex_flip() {
        let mut vertex = Vertex::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(1.0, 0.0, 0.0));
        let original_pos = vertex.pos;

        vertex.flip();

        // Position should remain unchanged
        assert!((vertex.pos - original_pos).magnitude() < TEST_EPSILON,
                "Position should not change during flip: expected {:?}, got {:?}", original_pos, vertex.pos);

        // Normal should be negated
        let expected_normal = Vector3::new(-1.0, 0.0, 0.0);
        assert!((vertex.normal - expected_normal).magnitude() < TEST_EPSILON,
                "Normal should be negated: expected {:?}, got {:?}", expected_normal, vertex.normal);
    }

    // ===== PLANE TESTS =====

    #[test]
    fn test_plane_from_points_basic() {
        // Create plane from three points forming a triangle in XY plane
        let a = Vector3::new(0.0, 0.0, 0.0);
        let b = Vector3::new(1.0, 0.0, 0.0);
        let c = Vector3::new(0.0, 1.0, 0.0);

        let plane = Plane::from_points(&a, &b, &c);

        // Expected normal: (b-a) x (c-a) = (1,0,0) x (0,1,0) = (0,0,1)
        let expected_normal = Vector3::new(0.0, 0.0, 1.0);
        assert!((plane.normal - expected_normal).magnitude() < TEST_EPSILON,
                "Normal failed: expected {:?}, got {:?}", expected_normal, plane.normal);

        // Expected w: normal · a = (0,0,1) · (0,0,0) = 0
        let expected_w = 0.0;
        assert!((plane.w - expected_w).abs() < TEST_EPSILON,
                "W value failed: expected {}, got {}", expected_w, plane.w);
    }

    #[test]
    fn test_plane_from_points_arbitrary() {
        // Create plane from three points at z=1
        let a = Vector3::new(1.0, 1.0, 1.0);
        let b = Vector3::new(2.0, 1.0, 1.0);
        let c = Vector3::new(1.0, 2.0, 1.0);

        let plane = Plane::from_points(&a, &b, &c);

        // Expected normal: (b-a) x (c-a) = (1,0,0) x (0,1,0) = (0,0,1)
        let expected_normal = Vector3::new(0.0, 0.0, 1.0);
        assert!((plane.normal - expected_normal).magnitude() < TEST_EPSILON,
                "Normal failed: expected {:?}, got {:?}", expected_normal, plane.normal);

        // Expected w: normal · a = (0,0,1) · (1,1,1) = 1
        let expected_w = 1.0;
        assert!((plane.w - expected_w).abs() < TEST_EPSILON,
                "W value failed: expected {}, got {}", expected_w, plane.w);
    }

    #[test]
    fn test_plane_flip() {
        let mut plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 1.0);

        plane.flip();

        // Normal should be negated
        let expected_normal = Vector3::new(0.0, 0.0, -1.0);
        assert!((plane.normal - expected_normal).magnitude() < TEST_EPSILON,
                "Normal should be negated: expected {:?}, got {:?}", expected_normal, plane.normal);

        // W should be negated
        let expected_w = -1.0;
        assert!((plane.w - expected_w).abs() < TEST_EPSILON,
                "W should be negated: expected {}, got {}", expected_w, plane.w);
    }

    #[test]
    fn test_plane_normal_is_normalized() {
        // Test with various point configurations
        let test_cases = vec![
            (Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)),
            (Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 1.0), Vector3::new(1.0, 2.0, 1.0)),
            (Vector3::new(0.0, 0.0, 0.0), Vector3::new(3.0, 0.0, 0.0), Vector3::new(0.0, 4.0, 0.0)),
        ];

        for (a, b, c) in test_cases {
            let plane = Plane::from_points(&a, &b, &c);
            let normal_magnitude = plane.normal.magnitude();
            assert!((normal_magnitude - 1.0).abs() < TEST_EPSILON,
                    "Normal should be normalized: expected magnitude 1.0, got {} for points {:?}, {:?}, {:?}",
                    normal_magnitude, a, b, c);
        }
    }

    // ===== POLYGON TESTS =====

    #[test]
    fn test_polygon_new_valid_triangle() {
        // Create triangle vertices forming a triangle in XY plane
        let vertices = vec![
            Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        let shared = Arc::new(PolygonShared::default());

        let polygon = Polygon::new(vertices.clone(), shared);

        // Verify vertices are preserved
        assert_eq!(polygon.vertices.len(), 3, "Polygon should have 3 vertices");
        for (i, vertex) in polygon.vertices.iter().enumerate() {
            assert!((vertex.pos - vertices[i].pos).magnitude() < TEST_EPSILON,
                    "Vertex {} position mismatch: expected {:?}, got {:?}", i, vertices[i].pos, vertex.pos);
            assert!((vertex.normal - vertices[i].normal).magnitude() < TEST_EPSILON,
                    "Vertex {} normal mismatch: expected {:?}, got {:?}", i, vertices[i].normal, vertex.normal);
        }

        // Verify plane is computed correctly (should be (0,0,1) with w=0)
        let expected_normal = Vector3::new(0.0, 0.0, 1.0);
        assert!((polygon.plane.normal - expected_normal).magnitude() < TEST_EPSILON,
                "Plane normal failed: expected {:?}, got {:?}", expected_normal, polygon.plane.normal);

        let expected_w = 0.0;
        assert!((polygon.plane.w - expected_w).abs() < TEST_EPSILON,
                "Plane w failed: expected {}, got {}", expected_w, polygon.plane.w);
    }

    #[test]
    fn test_polygon_flip_reverses_winding() {
        // Create triangle polygon
        let vertices = vec![
            Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        let shared = Arc::new(PolygonShared::default());
        let mut polygon = Polygon::new(vertices.clone(), shared);

        // Store original state
        let original_vertices = polygon.vertices.clone();
        let original_plane_normal = polygon.plane.normal;
        let original_plane_w = polygon.plane.w;

        polygon.flip();

        // Verify vertex order is reversed
        assert_eq!(polygon.vertices.len(), original_vertices.len(), "Vertex count should not change");
        for (i, vertex) in polygon.vertices.iter().enumerate() {
            let original_index = original_vertices.len() - 1 - i;
            let original_vertex = &original_vertices[original_index];

            // Position should be same
            assert!((vertex.pos - original_vertex.pos).magnitude() < TEST_EPSILON,
                    "Vertex {} position should be unchanged: expected {:?}, got {:?}",
                    i, original_vertex.pos, vertex.pos);

            // Normal should be negated
            let expected_normal = -original_vertex.normal;
            assert!((vertex.normal - expected_normal).magnitude() < TEST_EPSILON,
                    "Vertex {} normal should be negated: expected {:?}, got {:?}",
                    i, expected_normal, vertex.normal);
        }

        // Verify plane normal is negated
        let expected_plane_normal = -original_plane_normal;
        assert!((polygon.plane.normal - expected_plane_normal).magnitude() < TEST_EPSILON,
                "Plane normal should be negated: expected {:?}, got {:?}",
                expected_plane_normal, polygon.plane.normal);

        // Verify plane w is negated
        let expected_plane_w = -original_plane_w;
        assert!((polygon.plane.w - expected_plane_w).abs() < TEST_EPSILON,
                "Plane w should be negated: expected {}, got {}",
                expected_plane_w, polygon.plane.w);
    }

    #[test]
    #[should_panic(expected = "Polygon requires at least 3 vertices")]
    fn test_polygon_new_insufficient_vertices_zero() {
        let vertices = vec![];
        let shared = Arc::new(PolygonShared::default());
        Polygon::new(vertices, shared);
    }

    #[test]
    #[should_panic(expected = "Polygon requires at least 3 vertices")]
    fn test_polygon_new_insufficient_vertices_one() {
        let vertices = vec![
            Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        let shared = Arc::new(PolygonShared::default());
        Polygon::new(vertices, shared);
    }

    #[test]
    #[should_panic(expected = "Polygon requires at least 3 vertices")]
    fn test_polygon_new_insufficient_vertices_two() {
        let vertices = vec![
            Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        let shared = Arc::new(PolygonShared::default());
        Polygon::new(vertices, shared);
    }

    // ===== ROBUSTNESS TESTS =====

    #[test]
    fn test_vertex_interpolate_zero_normal_edge_case() {
        // Test interpolation where the result might have zero normal
        let v1 = Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let v2 = Vertex::new(Vector3::new(1.0, 0.0, 0.0), Vector3::new(-1.0, 0.0, 0.0));

        // At t=0.5, normals (1,0,0) and (-1,0,0) should interpolate to (0,0,0)
        // Our robust implementation should handle this gracefully
        let result = v1.interpolate(&v2, 0.5);

        // Should have a valid normal (not zero)
        assert!(result.normal.magnitude() > TEST_EPSILON,
                "Interpolated normal should not be zero: got {:?}", result.normal);

        // Should be one of the original normals as fallback
        let is_v1_normal = (result.normal - v1.normal).magnitude() < TEST_EPSILON;
        let is_v2_normal = (result.normal - v2.normal).magnitude() < TEST_EPSILON;
        assert!(is_v1_normal || is_v2_normal,
                "Fallback normal should be one of the originals: got {:?}", result.normal);
    }

    #[test]
    fn test_plane_from_degenerate_points() {
        // Test plane creation from collinear points (degenerate case)
        let a = Vector3::new(0.0, 0.0, 0.0);
        let b = Vector3::new(1.0, 0.0, 0.0);
        let c = Vector3::new(2.0, 0.0, 0.0); // Collinear with a and b

        let plane = Plane::from_points(&a, &b, &c);

        // Should have a valid normal (fallback to default)
        assert!((plane.normal.magnitude() - 1.0).abs() < TEST_EPSILON,
                "Degenerate plane should have normalized fallback normal: got magnitude {}",
                plane.normal.magnitude());

        // Should be the fallback XY plane
        let expected_normal = Vector3::new(0.0, 0.0, 1.0);
        assert!((plane.normal - expected_normal).magnitude() < TEST_EPSILON,
                "Degenerate plane should use fallback normal: expected {:?}, got {:?}",
                expected_normal, plane.normal);
    }

    #[test]
    fn test_polygon_with_very_small_triangle() {
        // Test polygon creation with a very small but valid triangle
        let vertices = vec![
            Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(1e-6, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(0.0, 1e-6, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        let shared = Arc::new(PolygonShared::default());

        // Should create successfully (might use fallback plane for very small triangles)
        let polygon = Polygon::new(vertices, shared);

        // Should have a valid plane normal
        assert!((polygon.plane.normal.magnitude() - 1.0).abs() < TEST_EPSILON,
                "Small triangle polygon should have normalized plane normal: got magnitude {}",
                polygon.plane.normal.magnitude());
    }
}
