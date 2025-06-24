//! src/mesh/csg/bsp_tree.rs
//!
//! Binary Space Partitioning Tree - The Mind of the CSG Chapel
//!
//! This module implements the BSP tree data structure that forms the core of the CSG system.
//! The BSP tree recursively partitions 3D space using polygon planes, enabling efficient
//! boolean operations between complex geometric objects.
//!
//! Following cathedral engineering principles, this module represents the "Mind" component
//! that implements the spatial reasoning and geometric logic of the CSG system.
//!
//! # BSP Tree Theory
//!
//! A Binary Space Partitioning tree is a method for recursively subdividing space into
//! convex sets by hyperplanes. In our 3D CSG context:
//!
//! - Each node represents a region of 3D space
//! - Internal nodes have a splitting plane that divides space into front and back regions
//! - Leaf nodes contain polygons that don't need further subdivision
//! - Polygons coplanar with a splitting plane are stored at that node
//! - Polygons entirely in front/back go to the respective child
//! - Spanning polygons (crossing the plane) are split into front and back parts
//!
//! This structure enables efficient CSG operations by providing spatial coherence
//! and allowing algorithms to process only relevant regions of space.

use crate::mesh::csg::{Polygon, Plane, EPSILON};


/// Classification of a polygon relative to a plane
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Classify a polygon relative to a plane using robust geometric tests
///
/// This function determines the spatial relationship between a polygon and a plane
/// by testing all vertices against the plane equation. The classification uses
/// epsilon-based tolerance to handle floating-point precision issues.
///
/// # Mathematical Foundation
///
/// Point-plane distance: `d = point · normal - w`
/// - `d > ε`: point is in front of plane
/// - `d < -ε`: point is behind plane
/// - `|d| ≤ ε`: point is on plane (within tolerance)
///
/// # Arguments
/// * `polygon` - The polygon to classify
/// * `plane` - The plane to classify against
///
/// # Returns
/// * `PolygonClassification::Front` - All vertices in front of plane
/// * `PolygonClassification::Back` - All vertices behind plane
/// * `PolygonClassification::Coplanar` - All vertices on plane (within epsilon)
/// * `PolygonClassification::Spanning` - Vertices on both sides of plane
///
/// # Edge Cases
/// * Vertices exactly on plane are treated as neutral (don't affect classification)
/// * Empty polygons are treated as coplanar
/// * Degenerate polygons (< 3 vertices) are handled gracefully
fn classify_polygon_to_plane(polygon: &Polygon, plane: &Plane) -> PolygonClassification {
    if polygon.vertices.len() < 3 {
        return PolygonClassification::Coplanar;
    }

    let mut front_count = 0;
    let mut back_count = 0;
    let mut _on_plane_count = 0; // Track for debugging/validation

    for vertex in &polygon.vertices {
        let distance = plane.normal.dot(&vertex.pos) - plane.w;

        if distance > EPSILON {
            front_count += 1;
        } else if distance < -EPSILON {
            back_count += 1;
        } else {
            _on_plane_count += 1;
        }
    }

    // Classification logic: spanning takes precedence over single-sided
    if front_count > 0 && back_count > 0 {
        PolygonClassification::Spanning
    } else if front_count > 0 {
        PolygonClassification::Front
    } else if back_count > 0 {
        PolygonClassification::Back
    } else {
        // All vertices are on the plane (within epsilon tolerance)
        PolygonClassification::Coplanar
    }
}

/// A node in the Binary Space Partitioning tree
/// 
/// Each node represents a region of 3D space partitioned by a plane. Polygons are
/// stored at nodes where they are coplanar with the partitioning plane. Child nodes
/// represent the regions in front of and behind the partitioning plane.
/// 
/// # BSP Tree Theory
/// - Leaf nodes: no partitioning plane, store polygons directly
/// - Internal nodes: have a partitioning plane and up to two children
/// - Polygons coplanar with the partitioning plane are stored at the node
/// - Polygons in front of the plane go to the front child
/// - Polygons behind the plane go to the back child
/// - Spanning polygons (crossing the plane) are split into front and back parts
#[derive(Clone, Debug)]
pub struct CsgNode {
    /// Polygons stored at this node (coplanar with splitting plane)
    pub polygons: Vec<Polygon>,
    /// Child node for polygons in front of splitting plane
    pub front: Option<Box<CsgNode>>,
    /// Child node for polygons behind splitting plane  
    pub back: Option<Box<CsgNode>>,
    /// Splitting plane (None for leaf nodes)
    pub plane: Option<Plane>,
}

impl CsgNode {
    /// Construct BSP tree from polygon list using recursive space partitioning
    ///
    /// This method builds a BSP tree by recursively partitioning space using polygon planes.
    /// The algorithm follows these steps:
    ///
    /// 1. **Base cases**: Empty list → leaf node, Single polygon → leaf node
    /// 2. **Splitting plane selection**: Use first polygon's plane as partitioner
    /// 3. **Polygon classification**: Classify remaining polygons against splitting plane
    /// 4. **Recursive subdivision**: Build front/back subtrees from classified polygons
    /// 5. **Tree assembly**: Create node with coplanar polygons and child subtrees
    ///
    /// # Algorithm Complexity
    /// - Time: O(n²) in worst case (unbalanced tree), O(n log n) average case
    /// - Space: O(n) for tree structure plus polygon storage
    ///
    /// # Splitting Strategy
    /// Currently uses first polygon's plane as splitter. Future optimizations could:
    /// - Choose plane that minimizes polygon splits
    /// - Balance front/back polygon counts
    /// - Prefer axis-aligned planes for better numerical stability
    ///
    /// # Arguments
    /// * `polygons` - List of polygons to build tree from
    ///
    /// # Returns
    /// * Root node of constructed BSP tree
    ///
    /// # Panics
    /// * Never panics - handles all input gracefully including empty lists
    pub fn new(polygons: Vec<Polygon>) -> Self {
        if polygons.is_empty() {
            // Empty list creates leaf node
            return Self {
                polygons: Vec::new(),
                front: None,
                back: None,
                plane: None,
            };
        }

        if polygons.len() == 1 {
            // Single polygon creates leaf node
            return Self {
                polygons,
                front: None,
                back: None,
                plane: None,
            };
        }

        // Multiple polygons: use first polygon's plane as splitter
        let splitting_plane = polygons[0].plane.clone();
        let mut coplanar = vec![polygons[0].clone()];
        let mut front_polygons = Vec::new();
        let mut back_polygons = Vec::new();

        // Classify remaining polygons against the splitting plane
        for polygon in polygons.iter().skip(1) {
            let classification = classify_polygon_to_plane(polygon, &splitting_plane);
            match classification {
                PolygonClassification::Coplanar => {
                    coplanar.push(polygon.clone());
                }
                PolygonClassification::Front => {
                    front_polygons.push(polygon.clone());
                }
                PolygonClassification::Back => {
                    back_polygons.push(polygon.clone());
                }
                PolygonClassification::Spanning => {
                    // Split spanning polygon using the plane's split_polygon method
                    let mut front_parts = Vec::new();
                    let mut back_parts = Vec::new();
                    let mut coplanar_front_parts = Vec::new();
                    let mut coplanar_back_parts = Vec::new();

                    splitting_plane.split_polygon(
                        polygon,
                        &mut coplanar_front_parts,
                        &mut coplanar_back_parts,
                        &mut front_parts,
                        &mut back_parts,
                    );

                    // Add split parts to appropriate collections
                    front_polygons.extend(front_parts);
                    back_polygons.extend(back_parts);
                    coplanar.extend(coplanar_front_parts);
                    coplanar.extend(coplanar_back_parts);
                }
            }
        }

        // Validation: ensure no polygons are lost during classification
        // Note: total_classified may be greater than input due to polygon splitting
        let total_classified = coplanar.len() + front_polygons.len() + back_polygons.len();
        debug_assert!(total_classified >= polygons.len(),
                     "BSP tree construction lost polygons: input {}, classified {}",
                     polygons.len(), total_classified);

        // Create child nodes if needed
        let front = if front_polygons.is_empty() {
            None
        } else {
            Some(Box::new(CsgNode::new(front_polygons)))
        };

        let back = if back_polygons.is_empty() {
            None
        } else {
            Some(Box::new(CsgNode::new(back_polygons)))
        };

        Self {
            polygons: coplanar,
            front,
            back,
            plane: Some(splitting_plane),
        }
    }

    /// Check if this node is a leaf (has no children)
    ///
    /// # Returns
    /// * `true` if node has no front or back children
    ///
    #[allow(dead_code)]
    pub fn is_leaf(&self) -> bool {
        self.front.is_none() && self.back.is_none()
    }

    /// Insert a single polygon into the appropriate subtree based on plane classification
    ///
    /// This method traverses the BSP tree to find the correct location for a polygon
    /// based on its spatial relationship to the splitting planes. The algorithm:
    ///
    /// 1. **Leaf nodes**: Add polygon directly to the node's polygon list
    /// 2. **Internal nodes**: Classify polygon against splitting plane
    ///    - Coplanar → store at current node
    ///    - Front → insert into front child (create if needed)
    ///    - Back → insert into back child (create if needed)
    ///    - Spanning → store at current node (conservative approach)
    ///
    /// # Tree Modification
    /// This method may create new child nodes if they don't exist, potentially
    /// converting leaf nodes into internal nodes.
    ///
    /// # Arguments
    /// * `polygon` - Polygon to insert into tree
    ///
    /// # Performance
    /// - Time: O(log n) average case, O(n) worst case (unbalanced tree)
    /// - Space: O(1) for insertion, may allocate new child nodes
    #[allow(dead_code)]
    pub fn insert_polygon(&mut self, polygon: Polygon) {
        if let Some(ref plane) = self.plane {
            let classification = classify_polygon_to_plane(&polygon, plane);
            match classification {
                PolygonClassification::Coplanar => {
                    self.polygons.push(polygon);
                }
                PolygonClassification::Front => {
                    if self.front.is_none() {
                        self.front = Some(Box::new(CsgNode::new(vec![])));
                    }
                    self.front.as_mut().unwrap().insert_polygon(polygon);
                }
                PolygonClassification::Back => {
                    if self.back.is_none() {
                        self.back = Some(Box::new(CsgNode::new(vec![])));
                    }
                    self.back.as_mut().unwrap().insert_polygon(polygon);
                }
                PolygonClassification::Spanning => {
                    // Split spanning polygon and insert parts into appropriate subtrees
                    let mut front_parts = Vec::new();
                    let mut back_parts = Vec::new();
                    let mut coplanar_front_parts = Vec::new();
                    let mut coplanar_back_parts = Vec::new();

                    plane.split_polygon(
                        &polygon,
                        &mut coplanar_front_parts,
                        &mut coplanar_back_parts,
                        &mut front_parts,
                        &mut back_parts,
                    );

                    // Insert front parts
                    for front_part in front_parts {
                        if self.front.is_none() {
                            self.front = Some(Box::new(CsgNode::new(vec![])));
                        }
                        self.front.as_mut().unwrap().insert_polygon(front_part);
                    }

                    // Insert back parts
                    for back_part in back_parts {
                        if self.back.is_none() {
                            self.back = Some(Box::new(CsgNode::new(vec![])));
                        }
                        self.back.as_mut().unwrap().insert_polygon(back_part);
                    }

                    // Insert coplanar parts at current node
                    self.polygons.extend(coplanar_front_parts);
                    self.polygons.extend(coplanar_back_parts);
                }
            }
        } else {
            // Leaf node: just add the polygon
            self.polygons.push(polygon);
        }
    }

    /// Collect all polygons from entire subtree using depth-first traversal
    ///
    /// # Returns
    /// * Vector containing all polygons in the subtree
    ///
    pub fn collect_polygons(&self) -> Vec<Polygon> {
        let mut result = Vec::new();

        // Add polygons from this node
        result.extend(self.polygons.iter().cloned());

        // Add polygons from front child
        if let Some(ref front) = self.front {
            result.extend(front.collect_polygons());
        }

        // Add polygons from back child
        if let Some(ref back) = self.back {
            result.extend(back.collect_polygons());
        }

        result
    }

    /// Calculate the total volume of all polygons in this BSP tree
    ///
    /// This method computes the volume by summing the volume contributions
    /// of all polygons in the tree using the divergence theorem.
    ///
    /// # Returns
    /// * Total signed volume of the mesh represented by this BSP tree
    ///
    pub fn calculate_volume(&self) -> f32 {
        let mut total_volume = 0.0;

        // Add volume contributions from polygons at this node
        for polygon in &self.polygons {
            total_volume += polygon.volume_contribution();
        }

        // Recursively add volume from children
        if let Some(ref front) = self.front {
            total_volume += front.calculate_volume();
        }
        if let Some(ref back) = self.back {
            total_volume += back.calculate_volume();
        }

        total_volume.abs() // Return absolute value to handle orientation
    }

    /// Calculate the total surface area of all polygons in this BSP tree
    ///
    /// # Returns
    /// * Total surface area of all polygons in the tree
    ///
    pub fn calculate_surface_area(&self) -> f32 {
        let mut total_area = 0.0;

        // Add area contributions from polygons at this node
        for polygon in &self.polygons {
            total_area += polygon.area();
        }

        // Recursively add area from children
        if let Some(ref front) = self.front {
            total_area += front.calculate_surface_area();
        }
        if let Some(ref back) = self.back {
            total_area += back.calculate_surface_area();
        }

        total_area
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::csg::{Vertex, Polygon, PolygonShared, Plane};
    use nalgebra::Vector3;
    use std::sync::Arc;
    
    const TEST_EPSILON: f32 = 1e-5;

    /// Helper function to create a valid triangle polygon for testing
    fn create_test_triangle(
        p1: Vector3<f32>, 
        p2: Vector3<f32>, 
        p3: Vector3<f32>,
        normal: Vector3<f32>
    ) -> Polygon {
        let vertices = vec![
            Vertex::new(p1, normal),
            Vertex::new(p2, normal),
            Vertex::new(p3, normal),
        ];
        let shared = Arc::new(PolygonShared::default());
        Polygon::new(vertices, shared)
    }

    /// Helper function to create a triangle in the XY plane at z=0
    fn create_xy_triangle(x_offset: f32, y_offset: f32) -> Polygon {
        create_test_triangle(
            Vector3::new(x_offset, y_offset, 0.0),
            Vector3::new(x_offset + 1.0, y_offset, 0.0),
            Vector3::new(x_offset, y_offset + 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0)
        )
    }

    /// Helper function to create a triangle in the XZ plane at y=0
    fn create_xz_triangle(x_offset: f32, z_offset: f32) -> Polygon {
        create_test_triangle(
            Vector3::new(x_offset, 0.0, z_offset),
            Vector3::new(x_offset + 1.0, 0.0, z_offset),
            Vector3::new(x_offset, 0.0, z_offset + 1.0),
            Vector3::new(0.0, 1.0, 0.0)
        )
    }

    #[test]
    fn test_csg_node_new_empty() {
        let node = CsgNode::new(vec![]);
        
        // Empty list should create leaf node with no plane
        assert!(node.is_leaf(), "Empty polygon list should create leaf node");
        assert!(node.plane.is_none(), "Leaf node should have no splitting plane");
        assert_eq!(node.polygons.len(), 0, "Empty node should have no polygons");
        assert!(node.front.is_none(), "Leaf node should have no front child");
        assert!(node.back.is_none(), "Leaf node should have no back child");
    }

    #[test]
    fn test_csg_node_new_single_polygon() {
        let triangle = create_xy_triangle(0.0, 0.0);
        let node = CsgNode::new(vec![triangle.clone()]);
        
        // Single polygon should create leaf node storing that polygon
        assert!(node.is_leaf(), "Single polygon should create leaf node");
        assert_eq!(node.polygons.len(), 1, "Node should store the single polygon");
        
        // Verify the stored polygon matches the input
        let stored_polygon = &node.polygons[0];
        assert_eq!(stored_polygon.vertices.len(), triangle.vertices.len(), 
                   "Stored polygon should have same vertex count");
        
        for (i, vertex) in stored_polygon.vertices.iter().enumerate() {
            assert!((vertex.pos - triangle.vertices[i].pos).magnitude() < TEST_EPSILON,
                    "Vertex {} position should match: expected {:?}, got {:?}", 
                    i, triangle.vertices[i].pos, vertex.pos);
        }
    }

    #[test]
    fn test_csg_node_new_multiple_coplanar() {
        // Create multiple triangles in the same XY plane (z=0)
        let triangle1 = create_xy_triangle(0.0, 0.0);
        let triangle2 = create_xy_triangle(2.0, 0.0);
        let triangle3 = create_xy_triangle(0.0, 2.0);
        
        let polygons = vec![triangle1, triangle2, triangle3];
        let node = CsgNode::new(polygons.clone());
        
        // Multiple coplanar polygons should be stored at root without subdivision
        assert_eq!(node.polygons.len(), 3, "All coplanar polygons should be stored at root");
        
        // Since all polygons are coplanar, there should be no subdivision
        // (This test may need adjustment based on actual implementation strategy)
        // For now, we test that all polygons are preserved somewhere in the tree
        let collected = node.collect_polygons();
        assert_eq!(collected.len(), 3, "All polygons should be preserved in tree");
    }

    #[test]
    fn test_csg_node_new_spanning_polygons() {
        // Create polygons on different sides of the first polygon's plane
        let xy_triangle = create_xy_triangle(0.0, 0.0);  // In XY plane (z=0)
        let front_triangle = create_test_triangle(
            Vector3::new(0.0, 0.0, 1.0),  // In front of XY plane (z > 0)
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0)
        );
        let back_triangle = create_test_triangle(
            Vector3::new(0.0, 0.0, -1.0),  // Behind XY plane (z < 0)
            Vector3::new(1.0, 0.0, -1.0),
            Vector3::new(0.0, 1.0, -1.0),
            Vector3::new(0.0, 0.0, 1.0)
        );
        
        let polygons = vec![xy_triangle, front_triangle, back_triangle];
        let node = CsgNode::new(polygons);
        
        // Should create front/back children for polygons on different sides
        // The exact structure depends on implementation, but we test basic properties
        let collected = node.collect_polygons();
        assert_eq!(collected.len(), 3, "All polygons should be preserved in tree");
        
        // At least one of front or back should exist if polygons are on different sides
        let has_children = node.front.is_some() || node.back.is_some();
        assert!(has_children, "Tree should have children when polygons span different sides of splitting plane");
    }

    #[test]
    fn test_csg_node_is_leaf() {
        // Test leaf detection for various node configurations
        let empty_node = CsgNode::new(vec![]);
        assert!(empty_node.is_leaf(), "Empty node should be leaf");
        
        let single_polygon_node = CsgNode::new(vec![create_xy_triangle(0.0, 0.0)]);
        assert!(single_polygon_node.is_leaf(), "Single polygon node should be leaf");
        
        // Test node with children (will need to be updated when implementation is complete)
        let mut node_with_children = CsgNode::new(vec![]);
        node_with_children.front = Some(Box::new(CsgNode::new(vec![])));
        assert!(!node_with_children.is_leaf(), "Node with front child should not be leaf");
        
        let mut node_with_back = CsgNode::new(vec![]);
        node_with_back.back = Some(Box::new(CsgNode::new(vec![])));
        assert!(!node_with_back.is_leaf(), "Node with back child should not be leaf");
    }

    #[test]
    fn test_csg_node_insert_polygon() {
        // Create initial tree with one polygon
        let initial_triangle = create_xy_triangle(0.0, 0.0);
        let mut node = CsgNode::new(vec![initial_triangle]);
        
        // Insert a polygon that should go to front (z > 0)
        let front_triangle = create_test_triangle(
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0)
        );
        
        node.insert_polygon(front_triangle);
        
        // Verify polygon was inserted (exact location depends on implementation)
        let collected = node.collect_polygons();
        assert_eq!(collected.len(), 2, "Tree should contain both original and inserted polygon");
    }

    #[test]
    fn test_csg_node_collect_polygons() {
        // Test collection preserves all polygons and maintains count
        let triangle1 = create_xy_triangle(0.0, 0.0);
        let triangle2 = create_xy_triangle(1.0, 0.0);
        let triangle3 = create_xy_triangle(0.0, 1.0);

        let polygons = vec![triangle1, triangle2, triangle3];
        let original_count = polygons.len();
        let node = CsgNode::new(polygons);

        let collected = node.collect_polygons();
        assert_eq!(collected.len(), original_count,
                   "Collection should preserve all polygons: expected {}, got {}",
                   original_count, collected.len());

        // Verify that each collected polygon has valid structure
        for (i, polygon) in collected.iter().enumerate() {
            assert!(polygon.vertices.len() >= 3,
                    "Collected polygon {} should have at least 3 vertices, got {}",
                    i, polygon.vertices.len());

            // Verify normal is normalized
            let normal_magnitude = polygon.plane.normal.magnitude();
            assert!((normal_magnitude - 1.0).abs() < TEST_EPSILON,
                    "Collected polygon {} should have normalized plane normal: got magnitude {}",
                    i, normal_magnitude);
        }
    }

    #[test]
    fn test_polygon_splitting_spanning_case() {
        // RED: Test that spanning polygons are properly split instead of treated as coplanar

        // Create a reference polygon in the XY plane (z=0) to establish the splitting plane
        let reference_polygon = create_xy_triangle(0.0, 0.0);

        // Create a spanning polygon that crosses the z=0 plane
        // Triangle with vertices at z=-1, z=0, z=1 (spans the plane)
        let spanning_polygon = create_test_triangle(
            Vector3::new(2.0, 0.0, -1.0),  // Behind plane (z < 0)
            Vector3::new(3.0, 0.0, 0.0),   // On plane (z = 0)
            Vector3::new(2.0, 1.0, 1.0),   // In front of plane (z > 0)
            Vector3::new(0.0, 0.0, 1.0)    // Normal pointing up
        );

        // Create BSP tree with reference polygon first (establishes splitting plane)
        // then the spanning polygon (should be split)
        let node = CsgNode::new(vec![reference_polygon.clone(), spanning_polygon.clone()]);

        // CRITICAL TEST: The spanning polygon should be split, not stored as coplanar
        let collected = node.collect_polygons();

        // Expected behavior after implementation:
        // - 1 reference polygon (coplanar)
        // - 2+ split parts from spanning polygon
        // Total: 3+ polygons
        assert!(collected.len() >= 3,
                "Should have reference polygon plus split parts of spanning polygon, got {} polygons",
                collected.len());

        // Verify that we have front and/or back children (indicating splitting occurred)
        let has_subdivision = node.front.is_some() || node.back.is_some();
        assert!(has_subdivision,
                "BSP tree should have subdivisions when spanning polygons are properly split");
    }

    #[test]
    fn test_polygon_splitting_edge_cases() {
        // RED: Test edge cases for polygon splitting

        // Test case 1: Polygon with vertex exactly on plane
        let _plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0);
        let polygon_on_plane = create_test_triangle(
            Vector3::new(0.0, 0.0, 0.0),   // Exactly on plane
            Vector3::new(1.0, 0.0, -1.0),  // Behind plane
            Vector3::new(0.0, 1.0, 1.0),   // In front of plane
            Vector3::new(0.0, 0.0, 1.0)
        );

        let node = CsgNode::new(vec![polygon_on_plane]);
        let collected = node.collect_polygons();

        // Should handle vertex-on-plane case correctly
        assert!(!collected.is_empty(), "Should preserve polygon even with vertex on plane");

        // Test case 2: Very small spanning polygon (numerical precision test)
        let small_spanning = create_test_triangle(
            Vector3::new(0.0, 0.0, -1e-6),  // Just behind plane
            Vector3::new(1.0, 0.0, 0.0),    // On plane
            Vector3::new(0.0, 1.0, 1e-6),   // Just in front of plane
            Vector3::new(0.0, 0.0, 1.0)
        );

        let node2 = CsgNode::new(vec![small_spanning]);
        let collected2 = node2.collect_polygons();

        // Should handle numerical precision correctly with EPSILON = 1e-5
        assert!(!collected2.is_empty(), "Should handle small spanning polygons correctly");
    }
}
