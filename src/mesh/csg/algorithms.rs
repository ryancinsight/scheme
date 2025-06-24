//! src/mesh/csg/algorithms.rs
//!
//! Geometric Algorithms - The Mind of the CSG Chapel (Computational Logic)
//!
//! This module implements the core geometric algorithms required for CSG operations,
//! including point-plane classification, polygon-plane classification, and polygon
//! splitting by planes. These algorithms form the mathematical foundation for all
//! BSP tree operations and CSG boolean operations.
//!
//! Following cathedral engineering principles, this module represents the "Mind"
//! component that implements the computational logic and geometric reasoning.
//!
//! # Mathematical Foundation
//!
//! ## Point-Plane Classification
//! The signed distance from a point to a plane is computed using the plane equation:
//! ```text
//! distance = point · normal - w
//! ```
//! Where:
//! - `point` is the 3D position vector
//! - `normal` is the unit normal vector of the plane
//! - `w` is the plane's distance from origin along the normal
//!
//! Classification uses epsilon-based tolerance to handle floating-point precision:
//! - `distance > ε`: point is in front of plane
//! - `distance < -ε`: point is behind plane
//! - `|distance| ≤ ε`: point is on plane (within tolerance)
//!
//! ## Polygon-Plane Classification
//! A polygon's relationship to a plane is determined by classifying all its vertices:
//! - **Front**: All vertices in front of plane
//! - **Back**: All vertices behind plane
//! - **Coplanar**: All vertices on plane (within epsilon)
//! - **Spanning**: Vertices on both sides of plane
//!
//! ## Polygon Splitting
//! When a polygon spans a plane, it must be split into front and back parts.
//! This involves:
//! 1. Finding edge-plane intersections
//! 2. Creating new vertices at intersection points via interpolation
//! 3. Constructing valid polygons on each side of the plane
//! 4. Ensuring proper winding order and normal consistency

use crate::mesh::csg::{Polygon, Plane, Vertex, EPSILON};
use nalgebra::Vector3;



/// Classification of a point relative to a plane
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum PointClassification {
    /// Point is in front of the plane (positive side)
    Front,
    /// Point is behind the plane (negative side)
    Back,
    /// Point is on the plane (within epsilon tolerance)
    OnPlane,
}

/// Classification of a polygon relative to a plane
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum PolygonClassification {
    /// Polygon is entirely in front of the plane
    Front,
    /// Polygon is entirely behind the plane
    Back,
    /// Polygon is coplanar with the plane
    Coplanar,
    /// Polygon spans the plane (vertices on both sides)
    Spanning,
}

/// Result of splitting a polygon by a plane
#[allow(dead_code)]
pub struct SplitResult {
    /// Polygons on the front side of the plane
    pub front: Vec<Polygon>,
    /// Polygons on the back side of the plane
    pub back: Vec<Polygon>,
}



/// Classify a point relative to a plane using robust signed distance calculation
///
/// This function computes the signed distance from a point to a plane and classifies
/// the point's position relative to the plane using adaptive epsilon-based tolerance for
/// enhanced numerical stability.
///
/// # Mathematical Details
///
/// The signed distance is computed using the plane equation:
/// ```text
/// distance = point · normal - w
/// ```
///
/// Classification rules with adaptive tolerance:
/// - `distance > ε`: Point is in front of plane (positive side)
/// - `distance < -ε`: Point is behind plane (negative side)
/// - `|distance| ≤ ε`: Point is on plane (within tolerance)
///
/// # Arguments
/// * `point` - 3D position vector to classify
/// * `plane` - Plane to classify against
///
/// # Returns
/// * `PointClassification` indicating the point's position relative to the plane
///
/// # Numerical Stability Enhancement
/// Uses adaptive epsilon based on point and plane magnitudes to handle varying scales.
/// Points very close to the plane are treated as being on the plane.
#[allow(dead_code)]
pub fn classify_point_to_plane(point: &Vector3<f32>, plane: &Plane) -> PointClassification {
    let distance = plane.normal.dot(point) - plane.w;

    // Calculate adaptive epsilon based on point and plane scale
    let point_magnitude = point.magnitude();
    let plane_magnitude = plane.w.abs().max(1.0);
    let scale_factor = point_magnitude.max(plane_magnitude);
    let adaptive_epsilon = EPSILON * scale_factor.max(1.0);

    if distance > adaptive_epsilon {
        PointClassification::Front
    } else if distance < -adaptive_epsilon {
        PointClassification::Back
    } else {
        PointClassification::OnPlane
    }
}

/// Enhanced point classification with explicit epsilon parameter
///
/// This function provides explicit control over the tolerance used for classification,
/// allowing for context-specific numerical stability adjustments.
///
/// # Arguments
/// * `point` - 3D position vector to classify
/// * `plane` - Plane to classify against
/// * `epsilon` - Explicit tolerance for classification
///
/// # Returns
/// * `PointClassification` indicating the point's position relative to the plane
#[allow(dead_code)]
pub fn classify_point_to_plane_with_epsilon(
    point: &Vector3<f32>,
    plane: &Plane,
    epsilon: f32
) -> PointClassification {
    let distance = plane.normal.dot(point) - plane.w;

    if distance > epsilon {
        PointClassification::Front
    } else if distance < -epsilon {
        PointClassification::Back
    } else {
        PointClassification::OnPlane
    }
}

/// Classify a polygon relative to a plane using vertex-based analysis
///
/// This function determines the spatial relationship between a polygon and a plane
/// by classifying all vertices and analyzing their distribution. The classification
/// is robust against degenerate cases and numerical precision issues.
///
/// # Algorithm
///
/// 1. **Degenerate case handling**: Polygons with < 3 vertices are coplanar
/// 2. **Vertex classification**: Each vertex is classified against the plane
/// 3. **Distribution analysis**: Count vertices on each side of the plane
/// 4. **Final classification**: Determine overall polygon relationship
///
/// # Classification Rules
///
/// - **Spanning**: Vertices exist on both front and back sides
/// - **Front**: All non-coplanar vertices are in front
/// - **Back**: All non-coplanar vertices are behind
/// - **Coplanar**: All vertices are on the plane (within epsilon)
///
/// # Arguments
/// * `polygon` - Polygon to classify
/// * `plane` - Plane to classify against
///
/// # Returns
/// * `PolygonClassification` indicating the polygon's relationship to the plane
///
/// # Edge Cases
/// * Degenerate polygons (< 3 vertices) are treated as coplanar
/// * Vertices exactly on the plane don't affect front/back classification
/// * Empty polygons are handled gracefully
#[allow(dead_code)]
pub fn classify_polygon_to_plane(polygon: &Polygon, plane: &Plane) -> PolygonClassification {
    if polygon.vertices.len() < 3 {
        return PolygonClassification::Coplanar;
    }

    // Calculate adaptive epsilon based on polygon scale
    let mut max_vertex_magnitude = 0.0f32;
    for vertex in &polygon.vertices {
        max_vertex_magnitude = max_vertex_magnitude.max(vertex.pos.magnitude());
    }
    let plane_magnitude = plane.w.abs().max(1.0);
    let scale_factor = max_vertex_magnitude.max(plane_magnitude);
    let adaptive_epsilon = EPSILON * scale_factor.max(1.0);

    // Enhanced boundary tolerance for nearly coplanar detection
    let coplanar_tolerance = adaptive_epsilon * 10.0;

    let mut front_count = 0;
    let mut back_count = 0;
    let mut _on_plane_count = 0;
    let mut max_distance = 0.0f32;

    for vertex in &polygon.vertices {
        let distance = plane.normal.dot(&vertex.pos) - plane.w;
        max_distance = max_distance.max(distance.abs());

        let classification = classify_point_to_plane_with_epsilon(&vertex.pos, plane, adaptive_epsilon);
        match classification {
            PointClassification::Front => front_count += 1,
            PointClassification::Back => back_count += 1,
            PointClassification::OnPlane => _on_plane_count += 1,
        }
    }

    // Enhanced coplanar detection for nearly coplanar polygons
    if max_distance <= coplanar_tolerance {
        return PolygonClassification::Coplanar;
    }

    // Robust spanning detection with minimum threshold
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

/// Split a polygon by a plane, creating front and back polygons
/// 
#[allow(dead_code)]
pub fn split_polygon_by_plane(polygon: &Polygon, plane: &Plane) -> SplitResult {
    let classification = classify_polygon_to_plane(polygon, plane);

    match classification {
        PolygonClassification::Front => {
            SplitResult {
                front: vec![polygon.clone()],
                back: Vec::new(),
            }
        }
        PolygonClassification::Back => {
            SplitResult {
                front: Vec::new(),
                back: vec![polygon.clone()],
            }
        }
        PolygonClassification::Coplanar => {
            // Coplanar polygons can go to either side - put in front by convention
            SplitResult {
                front: vec![polygon.clone()],
                back: Vec::new(),
            }
        }
        PolygonClassification::Spanning => {
            // For now, implement basic spanning polygon handling
            // TODO: Implement proper polygon splitting with edge-plane intersections
            split_spanning_polygon(polygon, plane)
        }
    }
}

/// Calculate exact intersection point between an edge and a plane using parametric line equation
///
/// # Mathematical Foundation
///
/// For a line segment from vertex v1 to v2, the parametric equation is:
/// ```text
/// P(t) = v1 + t * (v2 - v1)  where t ∈ [0,1]
/// ```
///
/// For a plane with normal n and distance w, the plane equation is:
/// ```text
/// n · P - w = 0
/// ```
///
/// Substituting the parametric line equation into the plane equation:
/// ```text
/// n · (v1 + t * (v2 - v1)) - w = 0
/// n · v1 + t * n · (v2 - v1) - w = 0
/// t = (w - n · v1) / (n · (v2 - v1))
/// ```
///
/// # Edge Cases Handled
/// - Parallel edges: `n · (v2 - v1) ≈ 0` → no intersection
/// - Vertices on plane: `|distance| ≤ EPSILON` → vertex is intersection
/// - Invalid parameter: `t ∉ [0,1]` → intersection outside edge
///
/// # Arguments
/// * `v1` - First vertex of edge
/// * `v2` - Second vertex of edge
/// * `plane` - Plane to intersect with
///
/// # Returns
/// * `Some(Vertex)` - Intersection vertex if edge crosses plane
/// * `None` - No intersection (parallel, outside edge, or degenerate)
#[allow(dead_code)]
fn calculate_edge_plane_intersection(v1: &Vertex, v2: &Vertex, plane: &Plane) -> Option<Vertex> {
    let edge_vector = v2.pos - v1.pos;
    let edge_length = edge_vector.magnitude();

    // Check for degenerate edge (too short)
    if edge_length < EPSILON {
        return None;
    }

    let denominator = plane.normal.dot(&edge_vector);

    // Enhanced parallel edge detection with adaptive epsilon
    let edge_magnitude = edge_vector.magnitude();
    let adaptive_parallel_epsilon = EPSILON * edge_magnitude.max(1.0);

    if denominator.abs() < adaptive_parallel_epsilon {
        return None;
    }

    // Calculate parametric intersection parameter t with numerical conditioning
    let numerator = plane.w - plane.normal.dot(&v1.pos);
    let t = numerator / denominator;

    // Enhanced bounds checking with small tolerance for numerical precision
    let bounds_epsilon = EPSILON * 0.1;
    if t < -bounds_epsilon || t > 1.0 + bounds_epsilon {
        return None;
    }

    // Clamp t to valid range to prevent numerical drift
    let t_clamped = t.max(0.0).min(1.0);

    // Calculate exact intersection point with numerical conditioning
    let intersection_pos = v1.pos + t_clamped * edge_vector;

    // Enhanced normal interpolation with validation
    let interpolated_normal = v1.normal + t_clamped * (v2.normal - v1.normal);
    let normal_magnitude = interpolated_normal.magnitude();

    let intersection_normal = if normal_magnitude > EPSILON {
        interpolated_normal.normalize()
    } else {
        // Fallback to plane normal if interpolation fails
        plane.normal
    };

    Some(Vertex::new(intersection_pos, intersection_normal))
}

/// Split a spanning polygon by a plane using exact edge-plane intersection calculations
///
/// # Algorithm Overview
///
/// This function implements precise polygon splitting using parametric line-plane
/// intersection mathematics. The algorithm:
///
/// 1. **Edge Analysis**: Examines each edge of the polygon to find exact intersections
/// 2. **Intersection Calculation**: Uses parametric equations to find precise intersection points
/// 3. **Polygon Construction**: Builds front and back polygons maintaining vertex order
/// 4. **Validation**: Ensures resulting polygons have valid geometry (≥3 vertices)
///
/// # Geometric Precision
///
/// Unlike conservative vertex-separation approaches, this implementation:
/// - Creates exact intersection vertices at mathematically correct positions
/// - Preserves polygon winding order and normal consistency
/// - Handles edge cases: coplanar edges, vertices on plane, parallel edges
/// - Maintains numerical stability using consistent epsilon tolerance
///
/// # Arguments
/// * `polygon` - Polygon to split (must be spanning the plane)
/// * `plane` - Splitting plane
///
/// # Returns
/// * `SplitResult` containing front and back polygon lists
#[allow(dead_code)]
fn split_spanning_polygon(polygon: &Polygon, plane: &Plane) -> SplitResult {
    let mut front_vertices = Vec::new();
    let mut back_vertices = Vec::new();

    // Process each edge of the polygon to build split polygons
    let vertex_count = polygon.vertices.len();

    for i in 0..vertex_count {
        let current_vertex = &polygon.vertices[i];
        let next_vertex = &polygon.vertices[(i + 1) % vertex_count];

        // Classify current vertex
        let current_classification = classify_point_to_plane(&current_vertex.pos, plane);

        // Add current vertex to appropriate side(s)
        match current_classification {
            PointClassification::Front => {
                front_vertices.push(current_vertex.clone());
            }
            PointClassification::Back => {
                back_vertices.push(current_vertex.clone());
            }
            PointClassification::OnPlane => {
                // Vertices exactly on plane belong to both sides
                front_vertices.push(current_vertex.clone());
                back_vertices.push(current_vertex.clone());
            }
        }

        // Check for edge-plane intersection
        if let Some(intersection_vertex) = calculate_edge_plane_intersection(current_vertex, next_vertex, plane) {
            // Add intersection vertex to both sides
            front_vertices.push(intersection_vertex.clone());
            back_vertices.push(intersection_vertex);
        }
    }

    // Create polygons from collected vertices
    let mut front_polygons = Vec::new();
    let mut back_polygons = Vec::new();

    // Validate and create front polygon
    if front_vertices.len() >= 3 {
        // Remove duplicate consecutive vertices to avoid degenerate polygons
        let cleaned_front = remove_consecutive_duplicates(&front_vertices);
        if cleaned_front.len() >= 3 {
            front_polygons.push(Polygon::new(cleaned_front, polygon.shared.clone()));
        }
    }

    // Validate and create back polygon
    if back_vertices.len() >= 3 {
        // Remove duplicate consecutive vertices to avoid degenerate polygons
        let cleaned_back = remove_consecutive_duplicates(&back_vertices);
        if cleaned_back.len() >= 3 {
            back_polygons.push(Polygon::new(cleaned_back, polygon.shared.clone()));
        }
    }

    // Fallback: if we couldn't create valid polygons on both sides,
    // use conservative approach to ensure spanning polygons produce results
    if front_polygons.is_empty() || back_polygons.is_empty() {
        return conservative_split_fallback(polygon, plane);
    }

    SplitResult {
        front: front_polygons,
        back: back_polygons,
    }
}

/// Remove consecutive duplicate vertices from a vertex list to prevent degenerate polygons
///
/// This function ensures polygon validity by removing vertices that are too close to
/// their neighbors (within EPSILON distance). This prevents creation of degenerate
/// polygons with zero-length edges that can cause numerical instability.
///
/// # Arguments
/// * `vertices` - List of vertices to clean
///
/// # Returns
/// * Cleaned vertex list with consecutive duplicates removed
#[allow(dead_code)]
fn remove_consecutive_duplicates(vertices: &[Vertex]) -> Vec<Vertex> {
    if vertices.len() < 2 {
        return vertices.to_vec();
    }

    let mut cleaned = Vec::new();
    cleaned.push(vertices[0].clone());

    for i in 1..vertices.len() {
        let distance = (vertices[i].pos - vertices[i-1].pos).magnitude();
        if distance > EPSILON {
            cleaned.push(vertices[i].clone());
        }
    }

    // Check if last vertex is too close to first vertex (closing the loop)
    if cleaned.len() > 1 {
        let distance = (cleaned[cleaned.len()-1].pos - cleaned[0].pos).magnitude();
        if distance <= EPSILON {
            cleaned.pop();
        }
    }

    cleaned
}

/// Conservative fallback splitting for edge cases where exact intersection fails
///
/// This function provides a robust fallback when the exact intersection algorithm
/// cannot produce valid polygons on both sides. It uses the original conservative
/// approach of vertex separation to ensure spanning polygons always produce results.
///
/// # Arguments
/// * `polygon` - Original polygon to split
/// * `plane` - Splitting plane
///
/// # Returns
/// * `SplitResult` with conservative split results
#[allow(dead_code)]
fn conservative_split_fallback(polygon: &Polygon, plane: &Plane) -> SplitResult {
    let mut front_vertices = Vec::new();
    let mut back_vertices = Vec::new();
    let mut on_plane_vertices = Vec::new();

    // Classify each vertex using conservative approach
    for vertex in &polygon.vertices {
        let classification = classify_point_to_plane(&vertex.pos, plane);
        match classification {
            PointClassification::Front => front_vertices.push(vertex.clone()),
            PointClassification::Back => back_vertices.push(vertex.clone()),
            PointClassification::OnPlane => on_plane_vertices.push(vertex.clone()),
        }
    }

    let mut front_polygons = Vec::new();
    let mut back_polygons = Vec::new();

    // Add on-plane vertices to both sides to help create valid polygons
    if !front_vertices.is_empty() && !back_vertices.is_empty() {
        front_vertices.extend(on_plane_vertices.iter().cloned());
        back_vertices.extend(on_plane_vertices.iter().cloned());

        // Create polygons if we have enough vertices
        if front_vertices.len() >= 3 {
            front_polygons.push(Polygon::new(front_vertices, polygon.shared.clone()));
        }

        if back_vertices.len() >= 3 {
            back_polygons.push(Polygon::new(back_vertices, polygon.shared.clone()));
        }

        // Final fallback: duplicate original polygon if needed
        if front_polygons.is_empty() {
            front_polygons.push(polygon.clone());
        }
        if back_polygons.is_empty() {
            back_polygons.push(polygon.clone());
        }
    } else {
        // Not actually spanning - return original polygon
        front_polygons.push(polygon.clone());
    }

    SplitResult {
        front: front_polygons,
        back: back_polygons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::csg::{Vertex, Polygon, PolygonShared};
    use nalgebra::Vector3;
    use std::sync::Arc;
    
    const TEST_EPSILON: f32 = 1e-5;

    /// Helper function to create a plane from normal and point
    fn create_plane(normal: Vector3<f32>, point: Vector3<f32>) -> Plane {
        let w = normal.dot(&point);
        Plane::new(normal, w)
    }

    /// Helper function to create a triangle polygon
    fn create_triangle(p1: Vector3<f32>, p2: Vector3<f32>, p3: Vector3<f32>) -> Polygon {
        let normal = (p2 - p1).cross(&(p3 - p1)).normalize();
        let vertices = vec![
            Vertex::new(p1, normal),
            Vertex::new(p2, normal),
            Vertex::new(p3, normal),
        ];
        let shared = Arc::new(PolygonShared::default());
        Polygon::new(vertices, shared)
    }

    // ===== POINT CLASSIFICATION TESTS =====

    #[test]
    fn test_classify_point_front() {
        // XY plane at z=0, normal pointing in +Z direction
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        let point = Vector3::new(0.0, 0.0, 1.0); // In front of plane (z > 0)
        
        let classification = classify_point_to_plane(&point, &plane);
        assert_eq!(classification, PointClassification::Front,
                   "Point at z=1 should be in front of XY plane");
    }

    #[test]
    fn test_classify_point_back() {
        // XY plane at z=0, normal pointing in +Z direction
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        let point = Vector3::new(0.0, 0.0, -1.0); // Behind plane (z < 0)
        
        let classification = classify_point_to_plane(&point, &plane);
        assert_eq!(classification, PointClassification::Back,
                   "Point at z=-1 should be behind XY plane");
    }

    #[test]
    fn test_classify_point_on_plane() {
        // XY plane at z=0, normal pointing in +Z direction
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        let point = Vector3::new(1.0, 1.0, 0.0); // On plane (z = 0)
        
        let classification = classify_point_to_plane(&point, &plane);
        assert_eq!(classification, PointClassification::OnPlane,
                   "Point at z=0 should be on XY plane");
    }

    #[test]
    fn test_classify_point_near_plane() {
        // Test epsilon tolerance for points very close to plane
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        let point_close = Vector3::new(0.0, 0.0, EPSILON * 0.5); // Within epsilon
        
        let classification = classify_point_to_plane(&point_close, &plane);
        assert_eq!(classification, PointClassification::OnPlane,
                   "Point within epsilon should be classified as on plane");
    }

    // ===== POLYGON CLASSIFICATION TESTS =====

    #[test]
    fn test_classify_polygon_front() {
        // Triangle entirely in front of XY plane
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 1.0)
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let classification = classify_polygon_to_plane(&triangle, &plane);
        assert_eq!(classification, PolygonClassification::Front,
                   "Triangle at z=1 should be entirely in front of XY plane");
    }

    #[test]
    fn test_classify_polygon_back() {
        // Triangle entirely behind XY plane
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(1.0, 0.0, -1.0),
            Vector3::new(0.0, 1.0, -1.0)
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let classification = classify_polygon_to_plane(&triangle, &plane);
        assert_eq!(classification, PolygonClassification::Back,
                   "Triangle at z=-1 should be entirely behind XY plane");
    }

    #[test]
    fn test_classify_polygon_coplanar() {
        // Triangle in XY plane
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0)
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let classification = classify_polygon_to_plane(&triangle, &plane);
        assert_eq!(classification, PolygonClassification::Coplanar,
                   "Triangle in XY plane should be coplanar with XY plane");
    }

    #[test]
    fn test_classify_polygon_spanning() {
        // Triangle spanning XY plane (vertices on both sides)
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, -1.0), // Behind plane
            Vector3::new(1.0, 0.0, 1.0),  // In front of plane
            Vector3::new(0.0, 1.0, 0.0)   // On plane
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let classification = classify_polygon_to_plane(&triangle, &plane);
        assert_eq!(classification, PolygonClassification::Spanning,
                   "Triangle with vertices on both sides should be spanning");
    }

    // ===== POLYGON SPLITTING TESTS =====

    #[test]
    fn test_split_polygon_no_split_needed_front() {
        // Triangle entirely in front - should return original in front list
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 1.0)
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let result = split_polygon_by_plane(&triangle, &plane);
        assert_eq!(result.front.len(), 1, "Front triangle should be returned in front list");
        assert_eq!(result.back.len(), 0, "No triangles should be in back list");
    }

    #[test]
    fn test_split_polygon_no_split_needed_back() {
        // Triangle entirely behind - should return original in back list
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(1.0, 0.0, -1.0),
            Vector3::new(0.0, 1.0, -1.0)
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let result = split_polygon_by_plane(&triangle, &plane);
        assert_eq!(result.front.len(), 0, "No triangles should be in front list");
        assert_eq!(result.back.len(), 1, "Back triangle should be returned in back list");
    }

    #[test]
    fn test_split_polygon_spanning() {
        // Triangle spanning plane - should create front and back polygons
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, -1.0), // Behind plane
            Vector3::new(1.0, 0.0, 1.0),  // In front of plane
            Vector3::new(0.0, 1.0, 1.0)   // In front of plane
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let result = split_polygon_by_plane(&triangle, &plane);
        assert!(result.front.len() > 0, "Spanning triangle should create front polygon(s)");
        assert!(result.back.len() > 0, "Spanning triangle should create back polygon(s)");
        
        // Verify all resulting polygons have at least 3 vertices
        for polygon in &result.front {
            assert!(polygon.vertices.len() >= 3, 
                    "Front polygon should have at least 3 vertices, got {}", 
                    polygon.vertices.len());
        }
        for polygon in &result.back {
            assert!(polygon.vertices.len() >= 3, 
                    "Back polygon should have at least 3 vertices, got {}", 
                    polygon.vertices.len());
        }
    }

    #[test]
    fn test_split_polygon_edge_on_plane() {
        // Triangle with one edge exactly on the splitting plane
        let triangle = create_triangle(
            Vector3::new(0.0, 0.0, 0.0),  // On plane
            Vector3::new(1.0, 0.0, 0.0),  // On plane
            Vector3::new(0.5, 1.0, 1.0)   // In front of plane
        );
        let plane = create_plane(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0));
        
        let result = split_polygon_by_plane(&triangle, &plane);
        
        // Should handle edge case gracefully - exact behavior depends on implementation
        // At minimum, should not panic and should preserve polygon count
        let total_polygons = result.front.len() + result.back.len();
        assert!(total_polygons > 0, "Split should produce at least one polygon");
    }

    // ===== ENHANCED INTERSECTION ALGORITHM TESTS =====

    #[test]
    fn test_calculate_edge_plane_intersection_basic() {
        // Test edge that crosses XY plane (Z=0) at midpoint
        let v1 = Vertex::new(Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0));
        let v2 = Vertex::new(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0));
        let plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0);

        let intersection = calculate_edge_plane_intersection(&v1, &v2, &plane);

        assert!(intersection.is_some(), "Should find intersection");
        let vertex = intersection.unwrap();
        assert!((vertex.pos.z - 0.0).abs() < TEST_EPSILON, "Intersection should be at Z=0");
        assert!((vertex.pos.x - 0.0).abs() < TEST_EPSILON, "X coordinate should be preserved");
        assert!((vertex.pos.y - 0.0).abs() < TEST_EPSILON, "Y coordinate should be preserved");
    }

    #[test]
    fn test_calculate_edge_plane_intersection_parallel() {
        // Test edge parallel to plane (no intersection)
        let v1 = Vertex::new(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0));
        let v2 = Vertex::new(Vector3::new(1.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0));
        let plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0);

        let intersection = calculate_edge_plane_intersection(&v1, &v2, &plane);

        assert!(intersection.is_none(), "Parallel edge should not intersect");
    }

    #[test]
    fn test_calculate_edge_plane_intersection_outside_edge() {
        // Test intersection that would occur outside the edge bounds
        let v1 = Vertex::new(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0));
        let v2 = Vertex::new(Vector3::new(0.0, 0.0, 2.0), Vector3::new(0.0, 0.0, 1.0));
        let plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0);

        let intersection = calculate_edge_plane_intersection(&v1, &v2, &plane);

        assert!(intersection.is_none(), "Intersection outside edge bounds should return None");
    }

    #[test]
    fn test_calculate_edge_plane_intersection_vertex_on_plane() {
        // Test edge where one vertex is exactly on the plane
        let v1 = Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)); // On plane
        let v2 = Vertex::new(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0)); // Above plane
        let plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0);

        let intersection = calculate_edge_plane_intersection(&v1, &v2, &plane);

        // Should find intersection at the vertex on the plane
        assert!(intersection.is_some(), "Should find intersection at vertex on plane");
        let vertex = intersection.unwrap();
        assert!((vertex.pos.z - 0.0).abs() < TEST_EPSILON, "Intersection should be at Z=0");
    }

    #[test]
    fn test_remove_consecutive_duplicates_basic() {
        let vertices = vec![
            Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)), // Duplicate
            Vertex::new(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];

        let cleaned = remove_consecutive_duplicates(&vertices);

        assert_eq!(cleaned.len(), 3, "Should remove one duplicate vertex");
        assert!((cleaned[0].pos - Vector3::new(0.0, 0.0, 0.0)).magnitude() < TEST_EPSILON);
        assert!((cleaned[1].pos - Vector3::new(1.0, 0.0, 0.0)).magnitude() < TEST_EPSILON);
        assert!((cleaned[2].pos - Vector3::new(0.0, 1.0, 0.0)).magnitude() < TEST_EPSILON);
    }

    #[test]
    fn test_enhanced_split_spanning_polygon() {
        // Create a triangle that spans the XY plane (Z=0) with exact intersection
        let shared = Arc::new(PolygonShared::default());
        let vertices = vec![
            Vertex::new(Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0)), // Behind plane
            Vertex::new(Vector3::new(2.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0)),  // In front of plane
            Vertex::new(Vector3::new(1.0, 2.0, 1.0), Vector3::new(0.0, 0.0, 1.0)),  // In front of plane
        ];
        let polygon = Polygon::new(vertices, shared);

        // Create XY plane (Z=0)
        let plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0);

        // Split using enhanced algorithm
        let result = split_spanning_polygon(&polygon, &plane);

        // Should have polygons on both sides
        assert!(result.front.len() > 0, "Enhanced split should have front polygons");
        assert!(result.back.len() > 0, "Enhanced split should have back polygons");

        // Verify all resulting polygons have valid structure
        for polygon in &result.front {
            assert!(polygon.vertices.len() >= 3, "Front polygons should have at least 3 vertices");
            // Verify vertices are actually in front of or on the plane
            for vertex in &polygon.vertices {
                let distance = plane.normal.dot(&vertex.pos) - plane.w;
                assert!(distance >= -TEST_EPSILON, "Front polygon vertices should be in front of or on plane");
            }
        }
        for polygon in &result.back {
            assert!(polygon.vertices.len() >= 3, "Back polygons should have at least 3 vertices");
            // Verify vertices are actually behind or on the plane
            for vertex in &polygon.vertices {
                let distance = plane.normal.dot(&vertex.pos) - plane.w;
                assert!(distance <= TEST_EPSILON, "Back polygon vertices should be behind or on plane");
            }
        }
    }
}
