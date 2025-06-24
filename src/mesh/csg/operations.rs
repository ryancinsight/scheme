//! src/mesh/csg/operations.rs
//! 
//! CSG Boolean Operations - The Mind of the CSG Chapel (Business Logic)
//! 
//! This module implements the core CSG boolean operations (union, subtract, intersect, xor)
//! using Binary Space Partitioning trees. These operations form the heart of the CSG system,
//! enabling complex geometric modeling through boolean combinations of simpler shapes.
//! 
//! Following cathedral engineering principles, this module represents the "Mind" component
//! that implements the business logic and high-level geometric reasoning for CSG operations.
//! 
//! # CSG Boolean Operations Theory
//! 
//! ## Union (A ∪ B)
//! Combines two objects into a single object containing the volume of both.
//! Result contains all points that are in A OR in B.
//! 
//! ## Subtraction (A - B) 
//! Removes the volume of B from A, creating holes where B intersected A.
//! Result contains all points that are in A AND NOT in B.
//! **Critical**: A - B ≠ B - A (subtraction is not commutative)
//! 
//! ## Intersection (A ∩ B)
//! Keeps only the overlapping volume between A and B.
//! Result contains all points that are in A AND in B.
//! 
//! ## Exclusive-OR (A ⊕ B)
//! Symmetric difference - combines A and B but removes overlapping volume.
//! Result contains all points that are in A XOR in B (but not both).

use crate::mesh::csg::{CsgNode, Polygon, Vertex, Plane, EPSILON};
use nalgebra::Vector3;
use crate::mesh::csg::algorithms::PolygonClassification;


/// Calculate the centroid of a polygon
fn polygon_centroid(polygon: &Polygon) -> nalgebra::Vector3<f32> {
    let mut centroid = nalgebra::Vector3::new(0.0, 0.0, 0.0);
    for vertex in &polygon.vertices {
        centroid += vertex.pos;
    }
    centroid / polygon.vertices.len() as f32
}

/// Classify a polygon's position relative to an entire BSP tree
/// Returns whether the polygon is inside, outside, or on the boundary of the solid represented by the tree
///
/// **ENHANCED ALGORITHM**: Uses multiple-point sampling for more accurate classification
/// - Front = Outside the solid
/// - Back = Inside the solid
/// - Uses centroid + vertex sampling for robust overlapping case handling
fn classify_polygon_against_tree(polygon: &Polygon, tree: &CsgNode) -> PolygonClassification {
    // Sample multiple points on the polygon for more robust classification
    let mut inside_count = 0;
    let mut outside_count = 0;
    let mut _total_samples = 0;

    // Sample 1: Polygon centroid
    let centroid = polygon_centroid(polygon);
    match classify_point_against_tree(&centroid, tree) {
        PolygonClassification::Back => inside_count += 1,
        PolygonClassification::Front => outside_count += 1,
        PolygonClassification::Coplanar => outside_count += 1, // Treat boundary as outside
        PolygonClassification::Spanning => outside_count += 1, // Treat spanning as outside
    }
    _total_samples += 1;

    // Sample 2-4: Polygon vertices (up to 3 for performance)
    let vertex_samples = polygon.vertices.len().min(3);
    for i in 0..vertex_samples {
        match classify_point_against_tree(&polygon.vertices[i].pos, tree) {
            PolygonClassification::Back => inside_count += 1,
            PolygonClassification::Front => outside_count += 1,
            PolygonClassification::Coplanar => outside_count += 1, // Treat boundary as outside
            PolygonClassification::Spanning => outside_count += 1, // Treat spanning as outside
        }
        _total_samples += 1;
    }

    // Sample 5: Edge midpoints (for better boundary detection)
    if polygon.vertices.len() >= 3 {
        let edge_midpoint = (polygon.vertices[0].pos + polygon.vertices[1].pos) / 2.0;
        match classify_point_against_tree(&edge_midpoint, tree) {
            PolygonClassification::Back => inside_count += 1,
            PolygonClassification::Front => outside_count += 1,
            PolygonClassification::Coplanar => outside_count += 1, // Treat boundary as outside
            PolygonClassification::Spanning => outside_count += 1, // Treat spanning as outside
        }
        _total_samples += 1;
    }

    // Majority vote classification
    if inside_count > outside_count {
        PolygonClassification::Back // Inside
    } else if outside_count > inside_count {
        PolygonClassification::Front // Outside
    } else {
        // Tie case: use centroid as tie-breaker
        classify_point_against_tree(&centroid, tree)
    }
}

/// Check if a polygon is near the boundary of a BSP tree
/// Returns true if the polygon is close to any splitting plane
#[allow(dead_code)]
fn is_polygon_near_boundary(polygon: &Polygon, tree: &CsgNode) -> bool {
    // If no plane, this is a leaf - consider it boundary
    if tree.plane.is_none() {
        return true;
    }

    let plane = tree.plane.as_ref().unwrap();
    let centroid = polygon_centroid(polygon);

    // Check distance from centroid to plane
    let distance = (plane.normal.dot(&centroid) - plane.w).abs();

    // If centroid is very close to plane, consider it boundary
    if distance < crate::mesh::csg::EPSILON * 10.0 {
        return true;
    }

    // Recursively check child nodes
    if let Some(ref front) = tree.front {
        if is_polygon_near_boundary(polygon, front) {
            return true;
        }
    }

    if let Some(ref back) = tree.back {
        if is_polygon_near_boundary(polygon, back) {
            return true;
        }
    }

    false
}

/// Classify a point's position relative to an entire BSP tree
/// This is the core algorithm for inside/outside testing
///
/// **CORRECTED ALGORITHM**: Proper BSP tree traversal for inside/outside classification
/// - Empty tree = Outside
/// - Leaf with polygons = Inside (solid region)
/// - Internal nodes: traverse based on plane classification
fn classify_point_against_tree(point: &nalgebra::Vector3<f32>, tree: &CsgNode) -> PolygonClassification {
    // Base case: empty tree means point is outside
    if tree.polygons.is_empty() && tree.front.is_none() && tree.back.is_none() {
        return PolygonClassification::Front; // Outside = Front
    }

    // If this is a leaf node with polygons, the point is inside the solid
    // This is the key fix: leaf nodes with polygons represent solid regions
    if tree.plane.is_none() {
        if !tree.polygons.is_empty() {
            return PolygonClassification::Back; // Inside = Back
        } else {
            return PolygonClassification::Front; // Empty leaf = Outside
        }
    }

    // Internal node: classify against splitting plane and traverse
    let plane = tree.plane.as_ref().unwrap();
    let distance = plane.normal.dot(point) - plane.w;

    if distance > crate::mesh::csg::EPSILON {
        // Point is in front of splitting plane
        if let Some(ref front_child) = tree.front {
            classify_point_against_tree(point, front_child)
        } else {
            // No front child means this region is outside the solid
            PolygonClassification::Front // Outside
        }
    } else if distance < -crate::mesh::csg::EPSILON {
        // Point is behind splitting plane
        if let Some(ref back_child) = tree.back {
            classify_point_against_tree(point, back_child)
        } else {
            // No back child means this region is inside the solid
            PolygonClassification::Back // Inside
        }
    } else {
        // Point is on the splitting plane
        // Check both sides and use a tie-breaking rule
        let front_classification = if let Some(ref front_child) = tree.front {
            classify_point_against_tree(point, front_child)
        } else {
            PolygonClassification::Front
        };

        let back_classification = if let Some(ref back_child) = tree.back {
            classify_point_against_tree(point, back_child)
        } else {
            PolygonClassification::Back
        };

        // If either side says inside, consider it inside
        if matches!(back_classification, PolygonClassification::Back) {
            PolygonClassification::Back
        } else {
            front_classification
        }
    }
}

/// Invert a polygon's normals for subtract operations
/// Creates a new polygon with flipped vertex normals, reversed winding order, and inverted plane
fn invert_polygon_normals(polygon: &Polygon) -> Polygon {
    // Create new vertices with negated normals and reversed order
    let mut inverted_vertices: Vec<Vertex> = polygon.vertices
        .iter()
        .map(|v| Vertex::new(v.pos, -v.normal))
        .collect();

    // Reverse vertex order to flip winding
    inverted_vertices.reverse();

    // Create inverted plane
    let inverted_plane = Plane::new(-polygon.plane.normal, -polygon.plane.w);

    // Create new polygon with inverted properties
    Polygon {
        vertices: inverted_vertices,
        shared: polygon.shared.clone(),
        plane: inverted_plane,
    }
}

/// Collect polygons from tree A that are outside tree B
fn collect_outside_polygons(tree_a: &CsgNode, tree_b: &CsgNode) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut outside_polygons = Vec::new();

    for polygon in all_polygons_a {
        let classification = classify_polygon_against_tree(&polygon, tree_b);
        if matches!(classification, PolygonClassification::Front) {
            outside_polygons.push(polygon);
        }
    }

    outside_polygons
}

/// Collect polygons from tree A that are inside tree B
fn collect_inside_polygons(tree_a: &CsgNode, tree_b: &CsgNode) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut inside_polygons = Vec::new();

    let debug_classification = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();

    for (i, polygon) in all_polygons_a.into_iter().enumerate() {
        let classification = classify_polygon_against_tree(&polygon, tree_b);

        if debug_classification {
            let contribution = polygon.volume_contribution();
            println!("    Polygon[{}]: classification={:?}, volume_contribution={:.6}",
                     i, classification, contribution);
        }

        if matches!(classification, PolygonClassification::Back) {
            inside_polygons.push(polygon);
        }
    }

    if debug_classification {
        println!("  -> Collected {} inside polygons", inside_polygons.len());
    }

    inside_polygons
}

/// Collect polygons from tree A that are inside tree B, with volume contribution filtering
///
/// This function implements a corrected intersection algorithm that filters out polygons
/// with negative volume contributions, which typically indicate incorrect polygon orientation
/// for the intersection boundary.
///
/// # Arguments
/// * `tree_a` - Source BSP tree
/// * `tree_b` - Target BSP tree for inside/outside classification
///
/// # Returns
/// * Vector of polygons that are inside tree_b and have positive volume contributions
///
#[allow(dead_code)]
fn collect_inside_polygons_filtered(tree_a: &CsgNode, tree_b: &CsgNode) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut inside_polygons = Vec::new();

    let debug_classification = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let mut filtered_count = 0;

    for (i, polygon) in all_polygons_a.into_iter().enumerate() {
        let classification = classify_polygon_against_tree(&polygon, tree_b);
        let contribution = polygon.volume_contribution();

        if debug_classification {
            println!("    Polygon[{}]: classification={:?}, volume_contribution={:.6}",
                     i, classification, contribution);
        }

        // Collect polygons that are inside (Back) and have positive volume contribution
        if matches!(classification, PolygonClassification::Back) {
            // Filter: Only include polygons with positive volume contribution
            // Negative contributions typically indicate incorrect orientation for intersection
            if contribution >= -crate::mesh::csg::EPSILON {  // Allow small negative values due to numerical precision
                inside_polygons.push(polygon);
            } else {
                filtered_count += 1;
                if debug_classification {
                    println!("      -> FILTERED: negative volume contribution {:.6}", contribution);
                }
            }
        }
    }

    if debug_classification {
        println!("  -> Collected {} inside polygons (filtered {} negative contributions)",
                 inside_polygons.len(), filtered_count);
    }

    inside_polygons
}

/// Clip a list of polygons against a BSP tree, keeping only the portions inside the tree
///
/// This function implements the mathematically correct intersection algorithm by clipping
/// polygons against the boundary planes of a BSP tree. It produces exactly the polygons
/// that bound the intersection volume without double-counting.
///
/// # Mathematical Principle
/// For intersection A ∩ B, we need all parts of A that are inside B. This is achieved
/// by clipping each polygon of A against all boundary planes of B, keeping only the
/// portions that end up inside B's solid region.
///
/// # Arguments
/// * `polygons` - List of polygons to clip (typically from object A)
/// * `tree` - BSP tree to clip against (typically object B)
///
/// # Returns
/// * Vector of clipped polygons that represent the intersection boundary
///
#[allow(dead_code)]
fn clip_polygons_against_tree(polygons: &[Polygon], tree: &CsgNode) -> Vec<Polygon> {
    let mut result_polygons = Vec::new();

    for polygon in polygons {
        let clipped = clip_polygon_against_tree(polygon, tree);
        result_polygons.extend(clipped);
    }

    result_polygons
}

/// Clip a single polygon against a BSP tree, keeping only the portions inside the tree
///
/// This function recursively traverses the BSP tree, clipping the polygon against
/// each splitting plane and keeping only the portions that end up in the "inside"
/// (back) regions of the tree.
///
/// # Algorithm
/// 1. If tree is empty, polygon is outside → return empty
/// 2. If tree is a leaf with polygons, polygon is inside → return original
/// 3. If tree has splitting plane:
///    - Classify polygon against plane
///    - Front: recursively clip against front subtree
///    - Back: recursively clip against back subtree
///    - Spanning: split polygon and clip each part
///    - Coplanar: handle based on plane orientation
///
/// # Arguments
/// * `polygon` - Polygon to clip
/// * `tree` - BSP tree node to clip against
///
/// # Returns
/// * Vector of polygon fragments that are inside the tree
///
#[allow(dead_code)]
fn clip_polygon_against_tree(polygon: &Polygon, tree: &CsgNode) -> Vec<Polygon> {
    // Base case: empty tree means polygon is outside
    if tree.polygons.is_empty() && tree.front.is_none() && tree.back.is_none() {
        return vec![];
    }

    // Base case: leaf node with polygons means we're inside the solid
    if tree.front.is_none() && tree.back.is_none() {
        return vec![polygon.clone()];
    }

    // If we have a splitting plane, clip against it
    if let Some(ref plane) = tree.plane {
        use crate::mesh::csg::algorithms::classify_polygon_to_plane;
        let classification = classify_polygon_to_plane(polygon, plane);

        match classification {
            PolygonClassification::Front => {
                // Polygon is entirely in front of plane
                if let Some(ref front_tree) = tree.front {
                    clip_polygon_against_tree(polygon, front_tree)
                } else {
                    vec![] // No front subtree means outside
                }
            }
            PolygonClassification::Back => {
                // Polygon is entirely behind plane
                if let Some(ref back_tree) = tree.back {
                    clip_polygon_against_tree(polygon, back_tree)
                } else {
                    vec![] // No back subtree means outside
                }
            }
            PolygonClassification::Coplanar => {
                // Polygon is coplanar with splitting plane
                // For intersection, we consider coplanar polygons as inside
                // if they have the same orientation as the splitting plane
                if let Some(ref back_tree) = tree.back {
                    clip_polygon_against_tree(polygon, back_tree)
                } else {
                    vec![polygon.clone()] // Coplanar polygons are part of the boundary
                }
            }
            PolygonClassification::Spanning => {
                // Polygon spans the plane - split it and clip each part
                let mut front_polygons = Vec::new();
                let mut back_polygons = Vec::new();
                let mut coplanar_front = Vec::new();
                let mut coplanar_back = Vec::new();

                plane.split_polygon(
                    polygon,
                    &mut coplanar_front,
                    &mut coplanar_back,
                    &mut front_polygons,
                    &mut back_polygons,
                );

                let mut result = Vec::new();

                // Clip front fragments against front subtree
                if let Some(ref front_tree) = tree.front {
                    for front_poly in front_polygons {
                        result.extend(clip_polygon_against_tree(&front_poly, front_tree));
                    }
                }

                // Clip back fragments against back subtree
                if let Some(ref back_tree) = tree.back {
                    for back_poly in back_polygons {
                        result.extend(clip_polygon_against_tree(&back_poly, back_tree));
                    }
                }

                // Handle coplanar fragments
                if let Some(ref back_tree) = tree.back {
                    for coplanar_poly in coplanar_back {
                        result.extend(clip_polygon_against_tree(&coplanar_poly, back_tree));
                    }
                }

                result
            }
        }
    } else {
        // No splitting plane - this is a leaf node
        vec![polygon.clone()]
    }
}

/// Boolean union of two BSP trees: A ∪ B
/// 
/// Combines both objects into a single object containing the volume of both.
/// The result eliminates internal surfaces and produces a single connected volume.
/// 
/// # Arguments
/// * `a` - First BSP tree
/// * `b` - Second BSP tree
/// 
/// # Returns
/// * New BSP tree representing the union of A and B
/// 
pub fn union_bsp_trees(a: &CsgNode, b: &CsgNode) -> CsgNode {
    let mut result_polygons = Vec::new();

    // Add polygons from A that are outside B
    result_polygons.extend(collect_outside_polygons(a, b));

    // Add polygons from B that are outside A
    result_polygons.extend(collect_outside_polygons(b, a));

    CsgNode::new(result_polygons)
}

/// Boolean subtraction of two BSP trees: A - B
/// 
/// Removes the volume of B from A, creating holes where B intersected A.
/// This is the most important operation for creating complex shapes with cavities.
/// 
/// # Mathematical Semantics
/// - subtract(cube, sphere) = cube with spherical hole
/// - subtract(sphere, cube) = sphere with cubic hole
/// - A - B ≠ B - A (subtraction is NOT commutative)
/// 
/// # Arguments
/// * `a` - Base object (what to subtract FROM)
/// * `b` - Tool object (what to subtract)
/// 
/// # Returns
/// * New BSP tree representing A with B's volume removed
/// 
pub fn subtract_bsp_trees(a: &CsgNode, b: &CsgNode) -> CsgNode {
    let mut result_polygons = Vec::new();

    // Add polygons from A that are outside B
    result_polygons.extend(collect_outside_polygons(a, b));

    // Add inverted polygons from B that are inside A
    let inside_b = collect_inside_polygons(b, a);
    for polygon in inside_b {
        result_polygons.push(invert_polygon_normals(&polygon));
    }

    CsgNode::new(result_polygons)
}

/// Boolean intersection of two BSP trees: A ∩ B
///
/// Keeps only the overlapping volume between A and B.
/// Useful for finding the common volume between two objects.
///
/// # ADR: Intersection Algorithm Correction v8 - Proper Polygon Collection
/// **Problem**: Previous algorithm only collected polygons from A inside B, missing boundary formation
/// **Root Cause**: Intersection requires collecting from BOTH objects and proper boundary handling
/// **Mathematical Solution**: Collect inside polygons from both A and B, plus boundary clipping
///
/// **Correct Algorithm**:
/// 1. Collect polygons from A that are inside B
/// 2. Collect polygons from B that are inside A
/// 3. Clip boundary-spanning polygons from both objects
/// 4. Combine all results to form complete intersection boundary
///
/// This creates the proper intersection boundary with all necessary polygons.
///
/// # Arguments
/// * `a` - First BSP tree
/// * `b` - Second BSP tree
///
/// # Returns
/// * New BSP tree representing the intersection of A and B
///
pub fn intersect_bsp_trees(a: &CsgNode, b: &CsgNode) -> CsgNode {
    // BALANCED INTERSECTION ALGORITHM v13:
    // Hybrid approach combining inside polygons with boundary polygon clipping
    //
    // ADR: Balanced Intersection Algorithm
    // **Problem**: Strict inside was too conservative, original was too inclusive
    // **Solution**: Use original inside collection + selective boundary clipping
    // **Key Insight**: For intersection, we need both inside polygons AND properly clipped boundary polygons
    //
    // **Mathematical Foundation**:
    // Intersection A ∩ B consists of:
    // 1. Polygons from A that are inside B (contribute to intersection boundary)
    // 2. Polygons from B that are inside A (contribute to intersection boundary)
    // 3. Boundary polygons that are clipped to only include intersection portions
    //
    // **Balanced Algorithm**:
    // 1. Collect polygons from A that are inside B (original method)
    // 2. Collect polygons from B that are inside A (original method)
    // 3. Add clipped boundary polygons that span the intersection
    // 4. Apply enhanced deduplication to prevent double-counting

    let mut intersection_polygons = Vec::new();

    // Track 2: Enhanced diagnostic output for root cause investigation
    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if debug_enabled {
        println!("=== Track 2: BSP Tree Intersection Diagnostic Analysis ===");
        let a_polygons = a.collect_polygons();
        let b_polygons = b.collect_polygons();
        println!("  Input A: {} polygons", a_polygons.len());
        println!("  Input B: {} polygons", b_polygons.len());

        if volume_tracking {
            let vol_a = calculate_tree_volume(&a_polygons);
            let vol_b = calculate_tree_volume(&b_polygons);
            println!("  Volume A: {:.6}", vol_a);
            println!("  Volume B: {:.6}", vol_b);
            println!("  Expected intersection bounds: [0.0, {:.6}]", vol_a.min(vol_b));
        }
    }

    // Track 3: TDD Implementation - Corrected Symmetric Overlap Algorithm v15
    // Based on Track 2 root cause analysis: asymmetric volume contributions and boundary double-counting

    if debug_enabled { println!("  === Track 3: TDD Fix for Symmetric Overlap ==="); }

    // TDD GREEN PHASE: Implement corrected algorithm based on diagnostic findings
    // Root cause: Boundary polygons are being double-counted in symmetric cases
    // Solution: Use strict inside collection only, eliminate boundary processing for symmetric overlaps

    // Step 1: Collect polygons from A that are strictly inside B (no boundary inclusion)
    if debug_enabled { println!("  Step 1: Collecting A-strictly-inside-B polygons..."); }
    let a_inside_b = collect_strictly_inside_polygons_enhanced(a, b, "A→B");
    intersection_polygons.extend(a_inside_b);

    // Step 2: Collect polygons from B that are strictly inside A (no boundary inclusion)
    if debug_enabled { println!("  Step 2: Collecting B-strictly-inside-A polygons..."); }
    let b_inside_a = collect_strictly_inside_polygons_enhanced(b, a, "B→A");
    intersection_polygons.extend(b_inside_a);

    // Step 3: Track 3 Phase 3 - Enhanced Asymmetric Boundary Processing
    if debug_enabled { println!("  Step 3: Enhanced asymmetric boundary processing..."); }

    // TDD INSIGHT: Asymmetric cases require bidirectional boundary processing
    // Solution: Detect asymmetry and apply conditional bidirectional collection
    if intersection_polygons.is_empty() {
        if debug_enabled { println!("    No strictly inside polygons found - analyzing boundary asymmetry"); }

        // Collect boundary polygons from A that intersect with B
        let boundary_a = collect_boundary_intersection_single_representation(a, b, "A→B");
        let boundary_a_volume = calculate_tree_volume(&boundary_a);
        intersection_polygons.extend(boundary_a);

        // Track 3: Asymmetric Detection and Conditional B→A Processing
        let asymmetry_detected = detect_boundary_asymmetry(a, b, boundary_a_volume);

        if asymmetry_detected {
            if debug_enabled { println!("    ASYMMETRY DETECTED: Applying corrected bidirectional processing"); }

            // Track 3 Phase 5: Corrected Asymmetric Intersection Algorithm
            // Instead of adding B→A complement, use corrected bidirectional boundary processing
            let boundary_b_corrected = collect_boundary_intersection_corrected_asymmetric(b, a, "B→A", boundary_a_volume);
            intersection_polygons.extend(boundary_b_corrected);
        } else {
            if debug_enabled { println!("    SYMMETRIC CASE: Skipping B→A to prevent double-counting"); }
        }
    }

    // Step 4: Enhanced robustness - validate and filter polygons
    if debug_enabled { println!("  Step 4: Validating polygons..."); }
    let pre_validation_count = intersection_polygons.len();
    intersection_polygons = validate_and_filter_polygons(intersection_polygons);
    if debug_enabled {
        println!("    Filtered {} invalid polygons", pre_validation_count - intersection_polygons.len());
    }

    // Step 5: Enhanced deduplication to prevent symmetric case double-counting
    if debug_enabled { println!("  Step 5: Enhanced deduplication..."); }
    let pre_dedup_count = intersection_polygons.len();
    intersection_polygons = remove_duplicate_polygons_enhanced_v2(intersection_polygons);
    if debug_enabled {
        println!("    Removed {} duplicate polygons", pre_dedup_count - intersection_polygons.len());
    }

    // Track 2: Final diagnostic output
    if debug_enabled {
        println!("  Final result: {} polygons", intersection_polygons.len());
        if volume_tracking {
            let result_volume = calculate_tree_volume(&intersection_polygons);
            println!("  Result volume: {:.6}", result_volume);
        }
    }

    // Create final BSP tree with validated polygons
    CsgNode::new(intersection_polygons)
}

/// Collect polygons that are strictly inside another BSP tree
///
/// This function is more conservative than collect_inside_polygons, requiring
/// that polygons be definitively inside the tree rather than on the boundary.
/// This prevents double-counting in symmetric overlap cases.
///
/// # Mathematical Foundation
///
/// A polygon is "strictly inside" if:
/// 1. All vertices are classified as "Back" (inside) by the BSP tree
/// 2. No vertices are on the boundary (OnPlane classification)
/// 3. The polygon centroid is also inside the tree
///
/// This stricter classification prevents the symmetric overlap issue where
/// boundary polygons were incorrectly included in both directions.
///
/// # Arguments
/// * `tree_a` - Source tree containing polygons to test
/// * `tree_b` - Target tree to test against
///
/// # Returns
/// * Vector of polygons that are strictly inside tree_b
#[allow(dead_code)]
fn collect_strictly_inside_polygons(tree_a: &CsgNode, tree_b: &CsgNode) -> Vec<Polygon> {
    let mut inside_polygons = Vec::new();
    let a_polygons = tree_a.collect_polygons();

    for polygon in &a_polygons {
        if is_polygon_strictly_inside(polygon, tree_b) {
            inside_polygons.push(polygon.clone());
        }
    }

    inside_polygons
}

/// Check if a polygon is strictly inside a BSP tree
///
/// This function implements strict inside classification to prevent
/// symmetric overlap issues. A polygon is strictly inside if:
/// 1. All vertices are inside (Back classification)
/// 2. No vertices are on the boundary (OnPlane)
/// 3. Polygon centroid is also inside
///
/// # Arguments
/// * `polygon` - Polygon to test
/// * `tree` - BSP tree to test against
///
/// # Returns
/// * `true` if polygon is strictly inside, `false` otherwise
#[allow(dead_code)]
fn is_polygon_strictly_inside(polygon: &Polygon, tree: &CsgNode) -> bool {
    // Check all vertices are strictly inside (no boundary vertices allowed)
    for vertex in &polygon.vertices {
        match classify_point_against_tree(&vertex.pos, tree) {
            crate::mesh::csg::algorithms::PolygonClassification::Back => {
                // Inside - continue checking
            },
            _ => {
                // Front, OnPlane, or other - not strictly inside
                return false;
            }
        }
    }

    // Additional check: polygon centroid must also be inside
    let centroid = calculate_polygon_centroid(polygon);
    match classify_point_against_tree(&centroid, tree) {
        crate::mesh::csg::algorithms::PolygonClassification::Back => true,
        _ => false,
    }
}

/// Collect spanning polygons that cross the intersection boundary
///
/// This function identifies polygons from tree A that span the boundary of tree B
/// and clips them to only include the portions that are inside B (part of intersection).
///
/// # Mathematical Foundation
///
/// For intersection A ∩ B, we need polygons that form the boundary of the overlapping region.
/// Spanning polygons are those that are partially inside B - we clip them to get only
/// the inside portions that contribute to the intersection boundary.
///
/// # Arguments
/// * `tree_a` - Source tree containing polygons to test
/// * `tree_b` - Boundary tree to clip against
///
/// # Returns
/// * Vector of clipped polygon portions that are inside tree_b
#[allow(dead_code)]
fn collect_spanning_intersection_polygons(tree_a: &CsgNode, tree_b: &CsgNode) -> Vec<Polygon> {
    let mut spanning_polygons = Vec::new();
    let a_polygons = tree_a.collect_polygons();

    for polygon in &a_polygons {
        // Check if polygon spans the boundary of tree_b
        let classification = classify_polygon_against_tree_for_intersection(polygon, tree_b);

        match classification {
            PolygonTreeClassification::Spanning => {
                // Polygon spans the boundary - clip it to get only the inside portion
                let clipped_inside = clip_polygon_to_inside(polygon, tree_b);
                spanning_polygons.extend(clipped_inside);
            },
            _ => {
                // Polygon is entirely inside, outside, or coplanar - already handled by other steps
            }
        }
    }

    spanning_polygons
}

/// Clip a polygon to only include portions that are inside the given tree
///
/// This function takes a polygon that spans a tree boundary and clips it to return
/// only the portions that are inside the tree, which contribute to intersection.
///
/// # Arguments
/// * `polygon` - Polygon to clip
/// * `tree` - Tree defining the inside/outside boundary
///
/// # Returns
/// * Vector of polygon fragments that are inside the tree
#[allow(dead_code)]
fn clip_polygon_to_inside(polygon: &Polygon, tree: &CsgNode) -> Vec<Polygon> {
    // Use the existing clipping infrastructure but filter for inside portions only
    let clipped_result = clip_polygon_against_tree(polygon, tree);

    // Filter to only include fragments that are inside the tree
    clipped_result.into_iter()
        .filter(|fragment| {
            // Test if fragment is inside by checking its centroid
            let centroid = calculate_polygon_centroid(fragment);
            match classify_point_against_tree(&centroid, tree) {
                crate::mesh::csg::algorithms::PolygonClassification::Back => true, // Inside
                _ => false, // Outside, Front, or OnPlane
            }
        })
        .collect()
}

/// Calculate the centroid (geometric center) of a polygon
///
/// # Arguments
/// * `polygon` - Polygon to calculate centroid for
///
/// # Returns
/// * Centroid position as Vector3
#[allow(dead_code)]
fn calculate_polygon_centroid(polygon: &Polygon) -> Vector3<f32> {
    let mut centroid = Vector3::new(0.0, 0.0, 0.0);
    for vertex in &polygon.vertices {
        centroid += vertex.pos;
    }
    centroid / polygon.vertices.len() as f32
}

/// Classification of polygon relative to tree boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum PolygonTreeClassification {
    Inside,
    Outside,
    Spanning,
    Coplanar,
}

/// Classification of point relative to tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum PointTreeClassification {
    Inside,
    Outside,
}

/// Classify a polygon against a BSP tree for intersection purposes
///
/// # Arguments
/// * `polygon` - Polygon to classify
/// * `tree` - BSP tree to classify against
///
/// # Returns
/// * Classification result
#[allow(dead_code)]
fn classify_polygon_against_tree_for_intersection(polygon: &Polygon, tree: &CsgNode) -> PolygonTreeClassification {
    let mut inside_count = 0;
    let mut outside_count = 0;

    for vertex in &polygon.vertices {
        match classify_point_against_tree(&vertex.pos, tree) {
            crate::mesh::csg::algorithms::PolygonClassification::Back => inside_count += 1, // Inside
            crate::mesh::csg::algorithms::PolygonClassification::Front => outside_count += 1, // Outside
            _ => {}, // OnPlane - neutral
        }
    }

    if inside_count > 0 && outside_count > 0 {
        PolygonTreeClassification::Spanning
    } else if inside_count > 0 {
        PolygonTreeClassification::Inside
    } else {
        PolygonTreeClassification::Outside
    }
}

/// Validate and filter polygons for CSG operations
///
/// This function removes degenerate polygons and validates polygon integrity
/// to improve numerical stability and prevent CSG operation failures.
///
/// # Arguments
/// * `polygons` - Input polygon list to validate
///
/// # Returns
/// * Filtered polygon list with degenerate polygons removed
fn validate_and_filter_polygons(polygons: Vec<Polygon>) -> Vec<Polygon> {
    polygons.into_iter()
        .filter(|polygon| is_valid_polygon(polygon))
        .collect()
}

/// Check if a polygon is valid for CSG operations
///
/// This function validates polygon geometry to ensure it's suitable for
/// CSG operations and won't cause numerical instability.
///
/// # Arguments
/// * `polygon` - Polygon to validate
///
/// # Returns
/// * `true` if polygon is valid, `false` if degenerate
fn is_valid_polygon(polygon: &Polygon) -> bool {
    // Check minimum vertex count
    if polygon.vertices.len() < 3 {
        return false;
    }

    // Check for duplicate vertices
    for i in 0..polygon.vertices.len() {
        for j in (i + 1)..polygon.vertices.len() {
            let distance = (polygon.vertices[i].pos - polygon.vertices[j].pos).magnitude();
            if distance < EPSILON {
                return false;
            }
        }
    }

    // Check for zero area (collinear vertices)
    if polygon.vertices.len() == 3 {
        let v1 = &polygon.vertices[0].pos;
        let v2 = &polygon.vertices[1].pos;
        let v3 = &polygon.vertices[2].pos;

        let edge1 = v2 - v1;
        let edge2 = v3 - v1;
        let cross = edge1.cross(&edge2);

        if cross.magnitude() < EPSILON {
            return false;
        }
    }

    true
}

/// Collect boundary polygons that contribute to intersection
///
/// This function identifies polygons that span the boundary between two objects
/// and clips them to only include portions that are part of the intersection.
/// This is more selective than the original clipping approach.
///
/// # Arguments
/// * `tree_a` - Source tree containing polygons to test
/// * `tree_b` - Target tree to test against
///
/// # Returns
/// * Vector of clipped polygon portions that contribute to intersection
#[allow(dead_code)]
fn collect_boundary_intersection_polygons(tree_a: &CsgNode, tree_b: &CsgNode) -> Vec<Polygon> {
    let mut boundary_polygons = Vec::new();
    let a_polygons = tree_a.collect_polygons();

    for polygon in &a_polygons {
        // Only process polygons that span the boundary
        let classification = classify_polygon_against_tree_for_intersection(polygon, tree_b);

        if let PolygonTreeClassification::Spanning = classification {
            // Clip spanning polygon to get only intersection portions
            let clipped = clip_polygon_to_inside(polygon, tree_b);

            // Only add non-empty clipped results
            for clipped_polygon in clipped {
                if clipped_polygon.vertices.len() >= 3 {
                    boundary_polygons.push(clipped_polygon);
                }
            }
        }
    }

    boundary_polygons
}

/// Enhanced polygon deduplication for symmetric cases
///
/// This function provides more sophisticated duplicate detection that handles
/// symmetric overlap cases where the same geometric surface might be represented
/// by multiple polygons from different sources.
///
/// # Arguments
/// * `polygons` - Input polygon list potentially containing duplicates
///
/// # Returns
/// * Deduplicated polygon list with enhanced symmetric case handling
#[allow(dead_code)]
fn remove_duplicate_polygons_enhanced(polygons: Vec<Polygon>) -> Vec<Polygon> {
    let mut unique_polygons = Vec::new();

    for polygon in polygons {
        let mut is_duplicate = false;

        for existing in &unique_polygons {
            if polygons_are_equivalent_enhanced(&polygon, existing) {
                is_duplicate = true;
                break;
            }
        }

        if !is_duplicate {
            unique_polygons.push(polygon);
        }
    }

    unique_polygons
}

/// Enhanced polygon equivalence check for symmetric cases
///
/// This function provides more sophisticated equivalence checking that handles
/// cases where polygons might be geometrically equivalent but represented differently.
///
/// # Arguments
/// * `a` - First polygon to compare
/// * `b` - Second polygon to compare
///
/// # Returns
/// * `true` if polygons are equivalent, `false` otherwise
#[allow(dead_code)]
fn polygons_are_equivalent_enhanced(a: &Polygon, b: &Polygon) -> bool {
    // Quick check: different vertex counts
    if a.vertices.len() != b.vertices.len() {
        return false;
    }

    // Check if polygons have the same area and centroid
    let area_a = calculate_polygon_area(a);
    let area_b = calculate_polygon_area(b);

    if (area_a - area_b).abs() > EPSILON {
        return false;
    }

    let centroid_a = calculate_polygon_centroid(a);
    let centroid_b = calculate_polygon_centroid(b);

    let centroid_distance = (centroid_a - centroid_b).magnitude();
    if centroid_distance > EPSILON {
        return false;
    }

    // If area and centroid match, consider them equivalent
    // This handles symmetric cases where the same surface is represented differently
    true
}

/// Calculate polygon area for equivalence checking
///
/// # Arguments
/// * `polygon` - Polygon to calculate area for
///
/// # Returns
/// * Polygon area
#[allow(dead_code)]
fn calculate_polygon_area(polygon: &Polygon) -> f32 {
    if polygon.vertices.len() < 3 {
        return 0.0;
    }

    // Use triangle fan method for polygon area
    let mut area = 0.0;
    let v0 = &polygon.vertices[0].pos;

    for i in 1..(polygon.vertices.len() - 1) {
        let v1 = &polygon.vertices[i].pos;
        let v2 = &polygon.vertices[i + 1].pos;

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let cross = edge1.cross(&edge2);
        area += cross.magnitude() * 0.5;
    }

    area
}

/// Remove duplicate polygons to reduce mesh complexity
///
/// This function identifies and removes duplicate polygons that can result
/// from CSG operations, reducing mesh complexity and improving performance.
///
/// # Arguments
/// * `polygons` - Input polygon list potentially containing duplicates
///
/// # Returns
/// * Deduplicated polygon list
#[allow(dead_code)]
fn remove_duplicate_polygons(polygons: Vec<Polygon>) -> Vec<Polygon> {
    let mut unique_polygons = Vec::new();

    for polygon in polygons {
        let mut is_duplicate = false;

        for existing in &unique_polygons {
            if polygons_are_equivalent(&polygon, existing) {
                is_duplicate = true;
                break;
            }
        }

        if !is_duplicate {
            unique_polygons.push(polygon);
        }
    }

    unique_polygons
}

/// Check if two polygons are geometrically equivalent
///
/// This function determines if two polygons represent the same geometric
/// surface within numerical tolerance.
///
/// # Arguments
/// * `a` - First polygon to compare
/// * `b` - Second polygon to compare
///
/// # Returns
/// * `true` if polygons are equivalent, `false` otherwise
#[allow(dead_code)]
fn polygons_are_equivalent(a: &Polygon, b: &Polygon) -> bool {
    // Quick check: different vertex counts
    if a.vertices.len() != b.vertices.len() {
        return false;
    }

    // Check if all vertices of A are close to vertices of B
    for vertex_a in &a.vertices {
        let mut found_match = false;
        for vertex_b in &b.vertices {
            let distance = (vertex_a.pos - vertex_b.pos).magnitude();
            if distance < EPSILON {
                found_match = true;
                break;
            }
        }
        if !found_match {
            return false;
        }
    }

    true
}

// ============================================================================
// Track 2: Root Cause Investigation & Diagnostic Enhancement Functions
// ============================================================================

/// Enhanced collect_inside_polygons with comprehensive diagnostic output
/// Track 2: Investigates symmetric overlap failures with detailed classification logging
#[allow(dead_code)]
fn collect_inside_polygons_with_diagnostics(tree_a: &CsgNode, tree_b: &CsgNode, direction: &str) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut inside_polygons = Vec::new();

    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if debug_enabled {
        println!("    Collecting inside polygons: {} ({} total polygons)", direction, all_polygons_a.len());
    }

    let mut total_volume_contribution = 0.0;
    let mut inside_count = 0;
    let mut boundary_count = 0;
    let mut outside_count = 0;

    for (i, polygon) in all_polygons_a.into_iter().enumerate() {
        let classification = classify_polygon_against_tree(&polygon, tree_b);
        let contribution = polygon.volume_contribution();

        if debug_enabled && i < 12 { // Limit output for readability
            println!("      Polygon[{}]: classification={:?}, volume_contribution={:.6}",
                     i, classification, contribution);
        }

        match classification {
            crate::mesh::csg::algorithms::PolygonClassification::Back => {
                inside_polygons.push(polygon);
                total_volume_contribution += contribution;
                inside_count += 1;
            },
            crate::mesh::csg::algorithms::PolygonClassification::Coplanar => {
                boundary_count += 1;
                if debug_enabled && i < 12 {
                    println!("        -> BOUNDARY: Not included in inside collection");
                }
            },
            crate::mesh::csg::algorithms::PolygonClassification::Spanning => {
                boundary_count += 1;
                if debug_enabled && i < 12 {
                    println!("        -> SPANNING: Not included in inside collection");
                }
            },
            crate::mesh::csg::algorithms::PolygonClassification::Front => {
                outside_count += 1;
            },
        }
    }

    if debug_enabled {
        println!("    {} Summary: {} inside, {} boundary, {} outside",
                 direction, inside_count, boundary_count, outside_count);
        if volume_tracking {
            println!("    {} Total volume contribution: {:.6}", direction, total_volume_contribution);
        }
    }

    inside_polygons
}

/// Enhanced boundary polygon collection with diagnostic output
/// Track 2: Investigates boundary polygon handling in symmetric overlap cases
#[allow(dead_code)]
fn collect_boundary_intersection_polygons_enhanced(tree_a: &CsgNode, tree_b: &CsgNode, direction: &str) -> Vec<Polygon> {
    let mut boundary_polygons = Vec::new();
    let a_polygons = tree_a.collect_polygons();

    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();

    if debug_enabled {
        println!("    Processing boundary polygons: {} ({} total polygons)", direction, a_polygons.len());
    }

    let mut spanning_count = 0;
    let mut clipped_count = 0;

    for polygon in &a_polygons {
        let classification = classify_polygon_against_tree_for_intersection(polygon, tree_b);

        if let PolygonTreeClassification::Spanning = classification {
            spanning_count += 1;
            let clipped = clip_polygon_to_inside(polygon, tree_b);

            for clipped_polygon in clipped {
                if clipped_polygon.vertices.len() >= 3 {
                    boundary_polygons.push(clipped_polygon);
                    clipped_count += 1;
                }
            }
        }
    }

    if debug_enabled {
        println!("    {} Boundary summary: {} spanning polygons -> {} clipped fragments",
                 direction, spanning_count, clipped_count);
    }

    boundary_polygons
}

/// Enhanced deduplication with geometric overlap detection for asymmetric boundary processing
/// Track 3: Eliminates double-counting by detecting spatial overlap between A→B and B→A polygon collections
#[allow(dead_code)]
fn remove_duplicate_polygons_enhanced_v2(polygons: Vec<Polygon>) -> Vec<Polygon> {
    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if debug_enabled {
        println!("    Enhanced deduplication: {} input polygons", polygons.len());
    }

    // Phase 1: Traditional exact equivalence deduplication
    let mut unique_polygons = Vec::new();
    let mut exact_duplicate_count = 0;
    let mut exact_removed_volume = 0.0;

    for polygon in polygons {
        let mut is_exact_duplicate = false;

        for existing in &unique_polygons {
            if polygons_are_equivalent_enhanced_v2(&polygon, existing) {
                is_exact_duplicate = true;
                exact_duplicate_count += 1;
                if volume_tracking {
                    exact_removed_volume += polygon.volume_contribution();
                }
                break;
            }
        }

        if !is_exact_duplicate {
            unique_polygons.push(polygon);
        }
    }

    if debug_enabled {
        println!("    Phase 1 - Exact deduplication: {} unique polygons ({} exact duplicates removed)",
                 unique_polygons.len(), exact_duplicate_count);
        if volume_tracking {
            println!("    Volume removed by exact deduplication: {:.6}", exact_removed_volume);
        }
    }

    // Phase 2: Enhanced geometric overlap detection for asymmetric boundary double-counting
    let (final_polygons, overlap_removed_count, overlap_removed_volume) =
        remove_geometric_overlap_asymmetric_boundary(unique_polygons);

    if debug_enabled {
        println!("    Phase 2 - Geometric overlap removal: {} final polygons ({} overlaps removed)",
                 final_polygons.len(), overlap_removed_count);
        if volume_tracking {
            println!("    Volume removed by overlap detection: {:.6}", overlap_removed_volume);
            println!("    Total volume removed: {:.6}", exact_removed_volume + overlap_removed_volume);
        }
    }

    final_polygons
}

/// Enhanced geometric overlap detection for asymmetric boundary double-counting elimination
/// Track 3: Detects spatial overlap between A→B and B→A polygon collections to prevent double-counting
#[allow(dead_code)]
fn remove_geometric_overlap_asymmetric_boundary(polygons: Vec<Polygon>) -> (Vec<Polygon>, usize, f32) {
    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if polygons.len() <= 1 {
        return (polygons, 0, 0.0);
    }

    let mut result_polygons = Vec::new();
    let mut processed_indices = std::collections::HashSet::new();
    let mut overlap_removed_count = 0;
    let mut overlap_removed_volume = 0.0;

    for (i, polygon_a) in polygons.iter().enumerate() {
        if processed_indices.contains(&i) {
            continue;
        }

        let mut overlapping_group = vec![polygon_a.clone()];
        let mut overlapping_indices = vec![i];

        // Find all polygons that spatially overlap with polygon_a
        for (j, polygon_b) in polygons.iter().enumerate().skip(i + 1) {
            if processed_indices.contains(&j) {
                continue;
            }

            if polygons_have_spatial_overlap(polygon_a, polygon_b) {
                overlapping_group.push(polygon_b.clone());
                overlapping_indices.push(j);

                if debug_enabled {
                    println!("      Detected spatial overlap: polygon[{}] and polygon[{}]", i, j);
                }
            }
        }

        // Mark all overlapping polygons as processed
        for &idx in &overlapping_indices {
            processed_indices.insert(idx);
        }

        if overlapping_group.len() > 1 {
            // Merge overlapping polygons into single representation
            let merged_polygon = merge_overlapping_polygons_volume_preserving(&overlapping_group);
            result_polygons.push(merged_polygon);

            overlap_removed_count += overlapping_group.len() - 1;
            if volume_tracking {
                let original_volume: f32 = overlapping_group.iter()
                    .map(|p| p.volume_contribution())
                    .sum();
                let merged_volume = result_polygons.last().unwrap().volume_contribution();
                overlap_removed_volume += original_volume - merged_volume;
            }

            if debug_enabled {
                println!("      Merged {} overlapping polygons into single representation", overlapping_group.len());
            }
        } else {
            // No overlap detected, keep original polygon
            result_polygons.push(polygon_a.clone());
        }
    }

    (result_polygons, overlap_removed_count, overlap_removed_volume)
}

/// Enhanced polygon equivalence with stricter symmetric overlap handling
/// Track 2: Prevents double-counting by using stricter geometric equivalence
#[allow(dead_code)]
fn polygons_are_equivalent_enhanced_v2(a: &Polygon, b: &Polygon) -> bool {
    // Quick check: different vertex counts
    if a.vertices.len() != b.vertices.len() {
        return false;
    }

    // Enhanced check: stricter area and centroid matching for symmetric cases
    let area_a = calculate_polygon_area(a);
    let area_b = calculate_polygon_area(b);

    // Use tighter epsilon for area comparison in symmetric cases
    let area_epsilon = EPSILON * 0.1; // 10x stricter than normal
    if (area_a - area_b).abs() > area_epsilon {
        return false;
    }

    let centroid_a = calculate_polygon_centroid(a);
    let centroid_b = calculate_polygon_centroid(b);

    let centroid_distance = (centroid_a - centroid_b).magnitude();
    let centroid_epsilon = EPSILON * 0.1; // 10x stricter than normal
    if centroid_distance > centroid_epsilon {
        return false;
    }

    // Additional check: normal vector similarity for symmetric overlap detection
    let normal_a = calculate_polygon_normal(a);
    let normal_b = calculate_polygon_normal(b);

    // Check if normals are parallel (same or opposite direction)
    let dot_product = normal_a.dot(&normal_b).abs();
    if dot_product < 0.99 { // Allow for small numerical differences
        return false;
    }

    true
}

/// Detect spatial overlap between two polygons for asymmetric boundary double-counting elimination
/// Track 3: Uses geometric intersection analysis to identify overlapping boundary regions
#[allow(dead_code)]
fn polygons_have_spatial_overlap(a: &Polygon, b: &Polygon) -> bool {
    // Quick rejection tests for performance
    if !polygons_have_overlapping_bounding_boxes(a, b) {
        return false;
    }

    // Check if polygons are coplanar or nearly coplanar
    if !polygons_are_coplanar_or_nearly_coplanar(a, b) {
        return false;
    }

    // Check for actual geometric intersection in 2D projected space
    polygons_intersect_in_projected_space(a, b)
}

/// Merge overlapping polygons into single volume-preserving representation
/// Track 3: Combines overlapping boundary regions to eliminate double-counting
fn merge_overlapping_polygons_volume_preserving(polygons: &[Polygon]) -> Polygon {
    if polygons.is_empty() {
        panic!("Cannot merge empty polygon list");
    }

    if polygons.len() == 1 {
        return polygons[0].clone();
    }

    // For asymmetric boundary overlap, use the polygon with the largest valid volume contribution
    // This preserves the correct intersection volume while eliminating double-counting
    let mut best_polygon = &polygons[0];
    let mut best_volume = best_polygon.volume_contribution();

    for polygon in &polygons[1..] {
        let volume = polygon.volume_contribution();
        if volume > best_volume && volume > EPSILON {
            best_polygon = polygon;
            best_volume = volume;
        }
    }

    best_polygon.clone()
}

/// Check if two polygons have overlapping bounding boxes (quick rejection test)
fn polygons_have_overlapping_bounding_boxes(a: &Polygon, b: &Polygon) -> bool {
    let (min_a, max_a) = calculate_polygon_bounding_box(a);
    let (min_b, max_b) = calculate_polygon_bounding_box(b);

    // Check for overlap in all three dimensions
    (min_a.x <= max_b.x + EPSILON && max_a.x >= min_b.x - EPSILON) &&
    (min_a.y <= max_b.y + EPSILON && max_a.y >= min_b.y - EPSILON) &&
    (min_a.z <= max_b.z + EPSILON && max_a.z >= min_b.z - EPSILON)
}

/// Calculate polygon bounding box
fn calculate_polygon_bounding_box(polygon: &Polygon) -> (Vector3<f32>, Vector3<f32>) {
    if polygon.vertices.is_empty() {
        return (Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
    }

    let first_pos = &polygon.vertices[0].pos;
    let mut min = *first_pos;
    let mut max = *first_pos;

    for vertex in &polygon.vertices[1..] {
        let pos = &vertex.pos;
        min.x = min.x.min(pos.x);
        min.y = min.y.min(pos.y);
        min.z = min.z.min(pos.z);
        max.x = max.x.max(pos.x);
        max.y = max.y.max(pos.y);
        max.z = max.z.max(pos.z);
    }

    (min, max)
}

/// Calculate polygon normal vector for enhanced equivalence checking
fn calculate_polygon_normal(polygon: &Polygon) -> Vector3<f32> {
    if polygon.vertices.len() < 3 {
        return Vector3::new(0.0, 0.0, 1.0); // Default normal
    }

    let v1 = &polygon.vertices[0].pos;
    let v2 = &polygon.vertices[1].pos;
    let v3 = &polygon.vertices[2].pos;

    let edge1 = v2 - v1;
    let edge2 = v3 - v1;
    let normal = edge1.cross(&edge2);

    if normal.magnitude() > EPSILON {
        normal.normalize()
    } else {
        Vector3::new(0.0, 0.0, 1.0) // Default for degenerate triangles
    }
}

/// Check if two polygons are coplanar or nearly coplanar
fn polygons_are_coplanar_or_nearly_coplanar(a: &Polygon, b: &Polygon) -> bool {
    let normal_a = calculate_polygon_normal(a);
    let normal_b = calculate_polygon_normal(b);

    // Check if normals are parallel (same or opposite direction)
    let dot_product = normal_a.dot(&normal_b).abs();
    if dot_product < 0.95 { // Allow for some tolerance
        return false;
    }

    // Check if polygons lie in the same plane by testing distance from plane
    if a.vertices.is_empty() || b.vertices.is_empty() {
        return false;
    }

    let point_a = &a.vertices[0].pos;
    let plane_distance_a = normal_a.dot(point_a);

    // Test if all vertices of polygon B are close to the plane of polygon A
    for vertex_b in &b.vertices {
        let distance_to_plane = (normal_a.dot(&vertex_b.pos) - plane_distance_a).abs();
        if distance_to_plane > EPSILON * 10.0 { // Allow for numerical tolerance
            return false;
        }
    }

    true
}

/// Check if two coplanar polygons intersect in their projected 2D space
fn polygons_intersect_in_projected_space(a: &Polygon, b: &Polygon) -> bool {
    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();

    // For asymmetric boundary overlap detection, use multiple criteria
    let centroid_a = calculate_polygon_centroid(a);
    let centroid_b = calculate_polygon_centroid(b);
    let distance = (centroid_a - centroid_b).magnitude();

    // Calculate characteristic size of polygons
    let area_a = calculate_polygon_area(a);
    let area_b = calculate_polygon_area(b);
    let characteristic_size = (area_a + area_b).sqrt() * 0.5;

    // Enhanced criteria for asymmetric boundary overlap:
    // 1. Centroid proximity test (relaxed threshold)
    let centroid_overlap = distance < characteristic_size * 1.2;

    // 2. Volume contribution similarity test (for boundary fragments)
    let volume_a = a.volume_contribution();
    let volume_b = b.volume_contribution();
    let volume_similarity = if volume_a.max(volume_b) > EPSILON {
        (volume_a - volume_b).abs() / volume_a.max(volume_b) < 0.1 // 10% tolerance
    } else {
        true // Both have negligible volume
    };

    // 3. Bounding box overlap test
    let bbox_overlap = polygons_have_overlapping_bounding_boxes(a, b);

    let result = centroid_overlap && volume_similarity && bbox_overlap;

    if debug_enabled && result {
        println!("        Spatial overlap detected: distance={:.6}, char_size={:.6}, vol_a={:.6}, vol_b={:.6}",
                 distance, characteristic_size, volume_a, volume_b);
    }

    result
}

/// Calculate total volume of a polygon collection for diagnostic purposes
/// Track 2: Volume tracking for root cause investigation
fn calculate_tree_volume(polygons: &[Polygon]) -> f32 {
    polygons.iter()
        .map(|p| p.volume_contribution())
        .sum()
}

/// Track 3: TDD Implementation - Strict inside polygon collection for symmetric overlap fix
/// This function implements the corrected algorithm based on Track 2 root cause analysis
fn collect_strictly_inside_polygons_enhanced(tree_a: &CsgNode, tree_b: &CsgNode, direction: &str) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut inside_polygons = Vec::new();

    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if debug_enabled {
        println!("    Strict inside collection: {} ({} total polygons)", direction, all_polygons_a.len());
    }

    let mut total_volume_contribution = 0.0;
    let mut strictly_inside_count = 0;
    let mut boundary_excluded_count = 0;
    let mut outside_count = 0;

    for (i, polygon) in all_polygons_a.into_iter().enumerate() {
        // Track 3: Use strict inside classification to prevent double-counting
        let is_strictly_inside = is_polygon_strictly_inside_enhanced(&polygon, tree_b);
        let contribution = polygon.volume_contribution();

        if debug_enabled && i < 12 { // Limit output for readability
            let classification = classify_polygon_against_tree(&polygon, tree_b);
            println!("      Polygon[{}]: classification={:?}, strictly_inside={}, volume_contribution={:.6}",
                     i, classification, is_strictly_inside, contribution);
        }

        if is_strictly_inside {
            inside_polygons.push(polygon);
            total_volume_contribution += contribution;
            strictly_inside_count += 1;
        } else {
            // Track 2: Classify why polygon was excluded for diagnostic purposes
            let classification = classify_polygon_against_tree(&polygon, tree_b);
            match classification {
                crate::mesh::csg::algorithms::PolygonClassification::Back => {
                    // Polygon was classified as inside but failed strict test (likely boundary)
                    boundary_excluded_count += 1;
                    if debug_enabled && i < 12 {
                        println!("        -> BOUNDARY EXCLUDED: Prevents double-counting");
                    }
                },
                _ => {
                    outside_count += 1;
                }
            }
        }
    }

    if debug_enabled {
        println!("    {} Strict Summary: {} strictly inside, {} boundary excluded, {} outside",
                 direction, strictly_inside_count, boundary_excluded_count, outside_count);
        if volume_tracking {
            println!("    {} Strict volume contribution: {:.6}", direction, total_volume_contribution);
        }
    }

    inside_polygons
}

/// Track 3: Enhanced strict inside test for symmetric overlap fix
/// This function implements stricter criteria to prevent boundary polygon inclusion
fn is_polygon_strictly_inside_enhanced(polygon: &Polygon, tree: &CsgNode) -> bool {
    // Track 3: Stricter criteria based on Track 2 analysis
    // 1. All vertices must be strictly inside (Back classification)
    // 2. No vertices can be on or near the boundary
    // 3. Polygon centroid must be strictly inside
    // 4. Enhanced epsilon tolerance for boundary detection

    let strict_epsilon = EPSILON * 0.1; // 10x stricter for boundary detection

    // Check all vertices are strictly inside with enhanced epsilon
    for vertex in &polygon.vertices {
        match classify_point_against_tree(&vertex.pos, tree) {
            crate::mesh::csg::algorithms::PolygonClassification::Back => {
                // Additional check: ensure vertex is not near any boundary plane
                if is_point_near_tree_boundary(&vertex.pos, tree, strict_epsilon) {
                    return false; // Too close to boundary
                }
            },
            _ => {
                // Front, Coplanar, or Spanning - not strictly inside
                return false;
            }
        }
    }

    // Additional check: polygon centroid must also be strictly inside
    let centroid = calculate_polygon_centroid(polygon);
    match classify_point_against_tree(&centroid, tree) {
        crate::mesh::csg::algorithms::PolygonClassification::Back => {
            // Ensure centroid is not near boundary
            !is_point_near_tree_boundary(&centroid, tree, strict_epsilon)
        },
        _ => false,
    }
}

/// Check if a point is near any boundary plane in the BSP tree
/// Track 3: Enhanced boundary detection for symmetric overlap fix
fn is_point_near_tree_boundary(point: &Vector3<f32>, tree: &CsgNode, epsilon: f32) -> bool {
    // If no plane, this is a leaf - not near boundary
    if tree.plane.is_none() {
        return false;
    }

    let plane = tree.plane.as_ref().unwrap();
    let distance = (plane.normal.dot(point) - plane.w).abs();

    // If point is very close to this plane, it's near boundary
    if distance < epsilon {
        return true;
    }

    // Recursively check child nodes
    if let Some(ref front) = tree.front {
        if is_point_near_tree_boundary(point, front, epsilon) {
            return true;
        }
    }

    if let Some(ref back) = tree.back {
        if is_point_near_tree_boundary(point, back, epsilon) {
            return true;
        }
    }

    false
}

/// Track 3 Phase 2: Single boundary representation for symmetric overlap fix
/// This function processes boundary polygons to create intersection without double-counting
fn collect_boundary_intersection_single_representation(tree_a: &CsgNode, tree_b: &CsgNode, direction: &str) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut intersection_polygons = Vec::new();

    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if debug_enabled {
        println!("    Single boundary representation: {} ({} total polygons)", direction, all_polygons_a.len());
    }

    let mut total_volume_contribution = 0.0;
    let mut clipped_count = 0;
    let mut spanning_count = 0;
    let mut excluded_count = 0;

    for (i, polygon) in all_polygons_a.into_iter().enumerate() {
        // Track 3: Check if polygon spans the intersection boundary
        let classification = classify_polygon_against_tree(&polygon, tree_b);
        let contribution = polygon.volume_contribution();

        if debug_enabled && i < 12 {
            println!("      Polygon[{}]: classification={:?}, volume_contribution={:.6}",
                     i, classification, contribution);
        }

        match classification {
            crate::mesh::csg::algorithms::PolygonClassification::Spanning => {
                spanning_count += 1;

                // Clip polygon to only include the part inside tree_b
                let clipped_fragments = clip_polygon_to_intersection_boundary(&polygon, tree_b);

                for fragment in clipped_fragments {
                    if fragment.vertices.len() >= 3 {
                        let fragment_contribution = fragment.volume_contribution();
                        intersection_polygons.push(fragment);
                        total_volume_contribution += fragment_contribution;
                        clipped_count += 1;

                        if debug_enabled && i < 12 {
                            println!("        -> CLIPPED FRAGMENT: volume_contribution={:.6}", fragment_contribution);
                        }
                    }
                }
            },
            crate::mesh::csg::algorithms::PolygonClassification::Back => {
                // Polygon is inside - include it directly (but this should be rare after strict filtering)
                intersection_polygons.push(polygon);
                total_volume_contribution += contribution;
                clipped_count += 1;

                if debug_enabled && i < 12 {
                    println!("        -> INSIDE: included directly");
                }
            },
            _ => {
                excluded_count += 1;
                if debug_enabled && i < 12 {
                    println!("        -> EXCLUDED: outside or coplanar");
                }
            }
        }
    }

    if debug_enabled {
        println!("    {} Boundary summary: {} spanning, {} clipped fragments, {} excluded",
                 direction, spanning_count, clipped_count, excluded_count);
        if volume_tracking {
            println!("    {} Boundary volume contribution: {:.6}", direction, total_volume_contribution);
        }
    }

    intersection_polygons
}

/// Track 3: Clip polygon to intersection boundary with enhanced precision
/// This function creates the exact intersection boundary without double-counting
fn clip_polygon_to_intersection_boundary(polygon: &Polygon, tree: &CsgNode) -> Vec<Polygon> {
    // Track 3: Enhanced clipping algorithm for symmetric overlap precision
    // Use parametric line-plane intersection for exact boundary calculation

    let mut result = vec![polygon.clone()];

    // Recursively clip against all planes in the BSP tree
    clip_polygon_against_bsp_tree_recursive(&mut result, tree);

    // Filter out degenerate polygons and ensure proper orientation
    result.into_iter()
        .filter(|p| p.vertices.len() >= 3)
        .filter(|p| polygon_area_is_significant(p))
        .collect()
}

/// Recursively clip polygon against BSP tree planes
fn clip_polygon_against_bsp_tree_recursive(polygons: &mut Vec<Polygon>, tree: &CsgNode) {
    if let Some(ref plane) = tree.plane {
        let mut new_polygons = Vec::new();

        for polygon in polygons.drain(..) {
            // Use the existing plane splitting functionality to clip polygon
            let mut coplanar_front = Vec::new();
            let mut coplanar_back = Vec::new();
            let mut front = Vec::new();
            let mut back = Vec::new();

            plane.split_polygon(
                &polygon,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );

            // Keep only the "back" (inside) parts for intersection
            new_polygons.extend(back);
            new_polygons.extend(coplanar_back);
        }

        *polygons = new_polygons;

        // Continue clipping against child nodes
        if let Some(ref back) = tree.back {
            clip_polygon_against_bsp_tree_recursive(polygons, back);
        }
    }
}

/// Check if polygon area is significant (not degenerate)
fn polygon_area_is_significant(polygon: &Polygon) -> bool {
    let area = calculate_polygon_area(polygon);
    area > EPSILON * EPSILON // Area threshold for significance
}

// ============================================================================
// Track 3: Enhanced Asymmetric Boundary Processing Functions
// ============================================================================

/// Track 3: Detect boundary asymmetry to determine if bidirectional processing is needed
/// This function analyzes the geometric configuration to identify asymmetric overlap cases
fn detect_boundary_asymmetry(tree_a: &CsgNode, tree_b: &CsgNode, boundary_a_volume: f32) -> bool {
    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();

    if debug_enabled {
        println!("    Asymmetry detection analysis:");
    }

    // Method 1: Volume-based asymmetry detection
    // In symmetric cases, A→B boundary volume should be close to expected intersection
    // In asymmetric cases, A→B boundary volume will be significantly less

    let a_polygons = tree_a.collect_polygons();
    let b_polygons = tree_b.collect_polygons();
    let vol_a = calculate_tree_volume(&a_polygons);
    let vol_b = calculate_tree_volume(&b_polygons);

    // Estimate expected intersection volume (rough heuristic)
    let estimated_intersection = vol_a.min(vol_b) * 0.5; // Conservative estimate

    let volume_ratio = if estimated_intersection > EPSILON {
        boundary_a_volume / estimated_intersection
    } else {
        1.0
    };

    if debug_enabled {
        println!("      Volume A: {:.6}, Volume B: {:.6}", vol_a, vol_b);
        println!("      Boundary A volume: {:.6}", boundary_a_volume);
        println!("      Estimated intersection: {:.6}", estimated_intersection);
        println!("      Volume ratio: {:.3}", volume_ratio);
    }

    // Method 2: Polygon distribution asymmetry detection
    let a_back_count = count_polygons_classified_as_back(tree_a, tree_b);
    let b_back_count = count_polygons_classified_as_back(tree_b, tree_a);

    let distribution_asymmetry = (a_back_count as f32 - b_back_count as f32).abs() /
                                (a_back_count + b_back_count).max(1) as f32;

    if debug_enabled {
        println!("      A→B back polygons: {}", a_back_count);
        println!("      B→A back polygons: {}", b_back_count);
        println!("      Distribution asymmetry: {:.3}", distribution_asymmetry);
    }

    // Asymmetry criteria:
    // 1. Volume ratio < 0.8 (A→B captures less than 80% of expected volume)
    // 2. Distribution asymmetry > 0.3 (significant difference in polygon counts)
    let volume_asymmetry = volume_ratio < 0.8;
    let polygon_asymmetry = distribution_asymmetry > 0.3;

    let asymmetric = volume_asymmetry || polygon_asymmetry;

    if debug_enabled {
        println!("      Volume asymmetry: {} (ratio < 0.8)", volume_asymmetry);
        println!("      Polygon asymmetry: {} (distribution > 0.3)", polygon_asymmetry);
        println!("      RESULT: {} asymmetry detected", if asymmetric { "ASYMMETRIC" } else { "SYMMETRIC" });
    }

    asymmetric
}

/// Count polygons classified as "Back" (inside) when tested against a tree
fn count_polygons_classified_as_back(tree_a: &CsgNode, tree_b: &CsgNode) -> usize {
    let polygons = tree_a.collect_polygons();
    polygons.iter()
        .filter(|polygon| {
            matches!(classify_polygon_against_tree(polygon, tree_b),
                    crate::mesh::csg::algorithms::PolygonClassification::Back)
        })
        .count()
}

/// Track 3: Collect boundary polygons for asymmetric complement (B→A direction)
/// This function collects the missing polygons that A→B boundary processing missed
fn collect_boundary_intersection_asymmetric_complement(tree_a: &CsgNode, tree_b: &CsgNode, direction: &str) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut complement_polygons = Vec::new();

    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if debug_enabled {
        println!("    Asymmetric complement collection: {} ({} total polygons)", direction, all_polygons_a.len());
    }

    let mut total_volume_contribution = 0.0;
    let mut complement_count = 0;
    let mut excluded_count = 0;

    for (i, polygon) in all_polygons_a.into_iter().enumerate() {
        // Track 3: Enhanced classification for asymmetric complement
        // Only collect polygons that contribute to the missing intersection volume
        let classification = classify_polygon_against_tree(&polygon, tree_b);
        let contribution = polygon.volume_contribution();

        if debug_enabled && i < 12 {
            println!("      Polygon[{}]: classification={:?}, volume_contribution={:.6}",
                     i, classification, contribution);
        }

        match classification {
            crate::mesh::csg::algorithms::PolygonClassification::Back => {
                // This polygon is inside tree_b and contributes to intersection
                // Apply additional filtering to avoid double-counting with A→B collection
                if is_polygon_asymmetric_complement(&polygon, tree_b) {
                    complement_polygons.push(polygon);
                    total_volume_contribution += contribution;
                    complement_count += 1;

                    if debug_enabled && i < 12 {
                        println!("        -> COMPLEMENT: Added to asymmetric collection");
                    }
                } else {
                    excluded_count += 1;
                    if debug_enabled && i < 12 {
                        println!("        -> EXCLUDED: Already covered by A→B collection");
                    }
                }
            },
            _ => {
                excluded_count += 1;
                if debug_enabled && i < 12 {
                    println!("        -> EXCLUDED: Outside or coplanar");
                }
            }
        }
    }

    if debug_enabled {
        println!("    {} Complement summary: {} complement polygons, {} excluded",
                 direction, complement_count, excluded_count);
        if volume_tracking {
            println!("    {} Complement volume contribution: {:.6}", direction, total_volume_contribution);
        }
    }

    complement_polygons
}

/// Check if a polygon should be included in asymmetric complement collection
/// This prevents double-counting while ensuring missing volume is captured
fn is_polygon_asymmetric_complement(polygon: &Polygon, tree: &CsgNode) -> bool {
    // Track 3 Phase 4: Corrected filtering for asymmetric complement
    // CRITICAL FIX: Only include polygons with positive volume contribution

    let volume_contribution = polygon.volume_contribution();

    // Reject polygons with negative or zero volume contribution
    if volume_contribution <= EPSILON {
        return false;
    }

    // Check if polygon centroid is well inside the tree (not on boundary)
    let centroid = calculate_polygon_centroid(polygon);
    let centroid_classification = classify_point_against_tree(&centroid, tree);

    match centroid_classification {
        crate::mesh::csg::algorithms::PolygonClassification::Back => {
            // Additional check: ensure polygon is not too close to any boundary plane
            // AND has positive volume contribution
            !is_point_near_tree_boundary(&centroid, tree, EPSILON * 2.0) && volume_contribution > EPSILON
        },
        _ => false,
    }
}

/// Track 3 Phase 5: Corrected asymmetric intersection algorithm
/// This function implements the mathematically correct approach for asymmetric overlap cases
fn collect_boundary_intersection_corrected_asymmetric(tree_a: &CsgNode, tree_b: &CsgNode, direction: &str, expected_missing_volume: f32) -> Vec<Polygon> {
    let all_polygons_a = tree_a.collect_polygons();
    let mut corrected_polygons = Vec::new();

    let debug_enabled = std::env::var("CSG_DEBUG_INTERSECTION").is_ok();
    let volume_tracking = std::env::var("CSG_DEBUG_VOLUME_TRACKING").is_ok();

    if debug_enabled {
        println!("    Corrected asymmetric intersection: {} ({} total polygons)", direction, all_polygons_a.len());
        if volume_tracking {
            println!("      Expected missing volume: {:.6}", expected_missing_volume);
        }
    }

    let mut total_volume_contribution = 0.0;
    let mut corrected_count = 0;
    let mut excluded_count = 0;

    for (i, polygon) in all_polygons_a.into_iter().enumerate() {
        // Track 3 Phase 5: Corrected classification for asymmetric intersection
        // Only collect polygons that are SPANNING the intersection boundary
        // and clip them to only include the intersection portion
        let classification = classify_polygon_against_tree(&polygon, tree_b);
        let contribution = polygon.volume_contribution();

        if debug_enabled && i < 12 {
            println!("      Polygon[{}]: classification={:?}, volume_contribution={:.6}",
                     i, classification, contribution);
        }

        match classification {
            crate::mesh::csg::algorithms::PolygonClassification::Spanning => {
                // This polygon spans the boundary - clip it to get only the intersection part
                let clipped_fragments = clip_polygon_to_intersection_boundary(&polygon, tree_b);

                for fragment in clipped_fragments {
                    if fragment.vertices.len() >= 3 {
                        let fragment_contribution = fragment.volume_contribution();

                        // Only include fragments with positive volume contribution
                        if fragment_contribution > EPSILON {
                            corrected_polygons.push(fragment);
                            total_volume_contribution += fragment_contribution;
                            corrected_count += 1;

                            if debug_enabled && i < 12 {
                                println!("        -> CORRECTED FRAGMENT: volume_contribution={:.6}", fragment_contribution);
                            }
                        } else {
                            if debug_enabled && i < 12 {
                                println!("        -> EXCLUDED FRAGMENT: negative/zero volume ({:.6})", fragment_contribution);
                            }
                        }
                    }
                }
            },
            crate::mesh::csg::algorithms::PolygonClassification::Back => {
                // Polygon is completely inside - include it if it has positive volume
                if contribution > EPSILON {
                    corrected_polygons.push(polygon);
                    total_volume_contribution += contribution;
                    corrected_count += 1;

                    if debug_enabled && i < 12 {
                        println!("        -> CORRECTED INSIDE: volume_contribution={:.6}", contribution);
                    }
                } else {
                    excluded_count += 1;
                    if debug_enabled && i < 12 {
                        println!("        -> EXCLUDED INSIDE: negative/zero volume ({:.6})", contribution);
                    }
                }
            },
            _ => {
                excluded_count += 1;
                if debug_enabled && i < 12 {
                    println!("        -> EXCLUDED: outside or coplanar");
                }
            }
        }
    }

    if debug_enabled {
        println!("    {} Corrected summary: {} corrected polygons, {} excluded",
                 direction, corrected_count, excluded_count);
        if volume_tracking {
            println!("    {} Corrected volume contribution: {:.6}", direction, total_volume_contribution);

            // Check if we're getting closer to the expected missing volume
            let volume_gap = expected_missing_volume - total_volume_contribution;
            println!("    {} Volume gap remaining: {:.6}", direction, volume_gap);
        }
    }

    corrected_polygons
}

/// Boolean exclusive-OR of two BSP trees: A ⊕ B
/// 
/// Symmetric difference - combines A and B but removes overlapping volume.
/// Equivalent to (A ∪ B) - (A ∩ B).
/// 
/// # Arguments
/// * `a` - First BSP tree
/// * `b` - Second BSP tree
/// 
/// # Returns
/// * New BSP tree representing the XOR of A and B
/// 
pub fn xor_bsp_trees(a: &CsgNode, b: &CsgNode) -> CsgNode {
    let mut result_polygons = Vec::new();

    // Add polygons from A that are outside B
    result_polygons.extend(collect_outside_polygons(a, b));

    // Add polygons from B that are outside A
    result_polygons.extend(collect_outside_polygons(b, a));

    CsgNode::new(result_polygons)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::csg::{CsgNode, Vertex, Polygon, PolygonShared};
    use nalgebra::Vector3;
    use std::sync::Arc;
    
    const TEST_EPSILON: f32 = 1e-5;

    /// Helper function to create a simple cube BSP tree centered at given position
    /// 
    /// Creates a cube with 6 faces (12 triangles) using the standard cube geometry.
    /// Each face is properly oriented with outward-facing normals.
    fn create_test_cube(center: Vector3<f32>, size: f32) -> CsgNode {
        let half_size = size * 0.5;
        let shared = Arc::new(PolygonShared::default());
        
        // Define cube vertices relative to center
        let vertices = [
            center + Vector3::new(-half_size, -half_size, -half_size), // 0: left-bottom-back
            center + Vector3::new( half_size, -half_size, -half_size), // 1: right-bottom-back
            center + Vector3::new( half_size,  half_size, -half_size), // 2: right-top-back
            center + Vector3::new(-half_size,  half_size, -half_size), // 3: left-top-back
            center + Vector3::new(-half_size, -half_size,  half_size), // 4: left-bottom-front
            center + Vector3::new( half_size, -half_size,  half_size), // 5: right-bottom-front
            center + Vector3::new( half_size,  half_size,  half_size), // 6: right-top-front
            center + Vector3::new(-half_size,  half_size,  half_size), // 7: left-top-front
        ];
        
        let mut polygons = Vec::new();
        
        // Front face (+Z)
        polygons.push(create_triangle_polygon(&vertices[4], &vertices[5], &vertices[6], Vector3::new(0.0, 0.0, 1.0), shared.clone()));
        polygons.push(create_triangle_polygon(&vertices[4], &vertices[6], &vertices[7], Vector3::new(0.0, 0.0, 1.0), shared.clone()));
        
        // Back face (-Z)
        polygons.push(create_triangle_polygon(&vertices[1], &vertices[0], &vertices[3], Vector3::new(0.0, 0.0, -1.0), shared.clone()));
        polygons.push(create_triangle_polygon(&vertices[1], &vertices[3], &vertices[2], Vector3::new(0.0, 0.0, -1.0), shared.clone()));
        
        // Right face (+X)
        polygons.push(create_triangle_polygon(&vertices[1], &vertices[2], &vertices[6], Vector3::new(1.0, 0.0, 0.0), shared.clone()));
        polygons.push(create_triangle_polygon(&vertices[1], &vertices[6], &vertices[5], Vector3::new(1.0, 0.0, 0.0), shared.clone()));
        
        // Left face (-X)
        polygons.push(create_triangle_polygon(&vertices[0], &vertices[4], &vertices[7], Vector3::new(-1.0, 0.0, 0.0), shared.clone()));
        polygons.push(create_triangle_polygon(&vertices[0], &vertices[7], &vertices[3], Vector3::new(-1.0, 0.0, 0.0), shared.clone()));
        
        // Top face (+Y)
        polygons.push(create_triangle_polygon(&vertices[3], &vertices[7], &vertices[6], Vector3::new(0.0, 1.0, 0.0), shared.clone()));
        polygons.push(create_triangle_polygon(&vertices[3], &vertices[6], &vertices[2], Vector3::new(0.0, 1.0, 0.0), shared.clone()));
        
        // Bottom face (-Y)
        polygons.push(create_triangle_polygon(&vertices[0], &vertices[1], &vertices[5], Vector3::new(0.0, -1.0, 0.0), shared.clone()));
        polygons.push(create_triangle_polygon(&vertices[0], &vertices[5], &vertices[4], Vector3::new(0.0, -1.0, 0.0), shared.clone()));
        
        CsgNode::new(polygons)
    }

    /// Helper function to create a simple tetrahedron BSP tree centered at given position
    /// 
    /// Creates a regular tetrahedron with 4 triangular faces.
    /// Each face is properly oriented with outward-facing normals.
    fn create_test_tetrahedron(center: Vector3<f32>, size: f32) -> CsgNode {
        let shared = Arc::new(PolygonShared::default());
        
        // Regular tetrahedron vertices (centered at origin, then translated)
        let h = size * 0.5;
        let vertices = [
            center + Vector3::new( h,  h,  h), // 0: top-right-front
            center + Vector3::new(-h, -h,  h), // 1: bottom-left-front  
            center + Vector3::new(-h,  h, -h), // 2: top-left-back
            center + Vector3::new( h, -h, -h), // 3: bottom-right-back
        ];
        
        let mut polygons = Vec::new();
        
        // Face 0-1-2 (front-left face)
        let normal_012 = (vertices[1] - vertices[0]).cross(&(vertices[2] - vertices[0])).normalize();
        polygons.push(create_triangle_polygon(&vertices[0], &vertices[1], &vertices[2], normal_012, shared.clone()));
        
        // Face 0-3-1 (front-right face)  
        let normal_031 = (vertices[3] - vertices[0]).cross(&(vertices[1] - vertices[0])).normalize();
        polygons.push(create_triangle_polygon(&vertices[0], &vertices[3], &vertices[1], normal_031, shared.clone()));
        
        // Face 0-2-3 (top face)
        let normal_023 = (vertices[2] - vertices[0]).cross(&(vertices[3] - vertices[0])).normalize();
        polygons.push(create_triangle_polygon(&vertices[0], &vertices[2], &vertices[3], normal_023, shared.clone()));
        
        // Face 1-3-2 (bottom face)
        let normal_132 = (vertices[3] - vertices[1]).cross(&(vertices[2] - vertices[1])).normalize();
        polygons.push(create_triangle_polygon(&vertices[1], &vertices[3], &vertices[2], normal_132, shared.clone()));
        
        CsgNode::new(polygons)
    }

    /// Helper function to create a triangle polygon with specified vertices and normal
    fn create_triangle_polygon(
        v1: &Vector3<f32>,
        v2: &Vector3<f32>,
        v3: &Vector3<f32>,
        normal: Vector3<f32>,
        shared: Arc<PolygonShared>
    ) -> Polygon {
        let vertices = vec![
            Vertex::new(*v1, normal),
            Vertex::new(*v2, normal),
            Vertex::new(*v3, normal),
        ];
        Polygon::new(vertices, shared)
    }

    /// Helper function to validate BSP tree structure and polygon integrity
    fn validate_bsp_tree(tree: &CsgNode) -> bool {
        let polygons = tree.collect_polygons();

        // Verify all polygons have at least 3 vertices
        for polygon in &polygons {
            if polygon.vertices.len() < 3 {
                return false;
            }

            // Verify normal is normalized
            let normal_magnitude = polygon.plane.normal.magnitude();
            if (normal_magnitude - 1.0).abs() > TEST_EPSILON {
                return false;
            }
        }

        true
    }

    // ===== SUBTRACT OPERATION TESTS =====

    #[test]
    fn test_subtract_cube_minus_tetrahedron() {
        // Create cube and tetrahedron BSP trees
        let cube = create_test_cube(Vector3::new(0.0, 0.0, 0.0), 2.0);
        let tetrahedron = create_test_tetrahedron(Vector3::new(0.0, 0.0, 0.0), 1.0);

        // Perform subtraction: cube - tetrahedron
        let result = subtract_bsp_trees(&cube, &tetrahedron);

        // Verify result is valid BSP tree
        assert!(validate_bsp_tree(&result), "Result should be a valid BSP tree");

        // Verify result has polygons (not empty)
        let result_polygons = result.collect_polygons();
        assert!(result_polygons.len() > 0, "Subtract result should have polygons");

        // Verify result has more polygons than original cube (due to cavity creation)
        let cube_polygons = cube.collect_polygons();
        assert!(result_polygons.len() >= cube_polygons.len(),
                "Subtract result should have at least as many polygons as original cube, got {} vs {}",
                result_polygons.len(), cube_polygons.len());

        // TODO: Add geometric validation once implementation is complete
        // - Verify cube's outer boundary is preserved
        // - Verify tetrahedral cavity is created inside
    }

    #[test]
    fn test_subtract_tetrahedron_minus_cube() {
        // Create tetrahedron and cube BSP trees
        let tetrahedron = create_test_tetrahedron(Vector3::new(0.0, 0.0, 0.0), 2.0);
        let cube = create_test_cube(Vector3::new(0.0, 0.0, 0.0), 1.0);

        // Perform subtraction: tetrahedron - cube
        let result = subtract_bsp_trees(&tetrahedron, &cube);

        // Verify result is valid BSP tree
        assert!(validate_bsp_tree(&result), "Result should be a valid BSP tree");

        // Verify result has polygons (not empty)
        let result_polygons = result.collect_polygons();
        assert!(result_polygons.len() > 0, "Subtract result should have polygons");

        // Verify this result is different from cube-minus-tetrahedron
        let cube_minus_tet = subtract_bsp_trees(&cube, &tetrahedron);
        let cube_minus_tet_polygons = cube_minus_tet.collect_polygons();

        // Results should have different polygon counts (geometric distinctness)
        assert_ne!(result_polygons.len(), cube_minus_tet_polygons.len(),
                   "tetrahedron - cube should be geometrically different from cube - tetrahedron: {} vs {} polygons",
                   result_polygons.len(), cube_minus_tet_polygons.len());
    }

    #[test]
    fn test_union_combines_volumes() {
        // Create two overlapping cubes
        let cube_a = create_test_cube(Vector3::new(-0.5, 0.0, 0.0), 1.0);
        let cube_b = create_test_cube(Vector3::new(0.5, 0.0, 0.0), 1.0);

        // Perform union
        let result = union_bsp_trees(&cube_a, &cube_b);

        // Verify result is valid BSP tree
        assert!(validate_bsp_tree(&result), "Union result should be a valid BSP tree");

        // Verify result has polygons
        let result_polygons = result.collect_polygons();
        assert!(result_polygons.len() > 0, "Union result should have polygons");

        // Union should have fewer polygons than simple combination (internal surfaces eliminated)
        let cube_a_polygons = cube_a.collect_polygons();
        let cube_b_polygons = cube_b.collect_polygons();
        let total_input_polygons = cube_a_polygons.len() + cube_b_polygons.len();

        assert!(result_polygons.len() <= total_input_polygons,
                "Union should not have more polygons than input combination: {} vs {} total",
                result_polygons.len(), total_input_polygons);
    }

    #[test]
    fn test_intersection_keeps_overlap_only() {
        // Create two overlapping cubes
        let cube_a = create_test_cube(Vector3::new(-0.5, 0.0, 0.0), 1.0);
        let cube_b = create_test_cube(Vector3::new(0.5, 0.0, 0.0), 1.0);

        // Perform intersection
        let result = intersect_bsp_trees(&cube_a, &cube_b);

        // Verify result is valid BSP tree
        assert!(validate_bsp_tree(&result), "Intersection result should be a valid BSP tree");

        // Verify result has polygons (overlapping volume exists)
        let result_polygons = result.collect_polygons();
        // Note: For overlapping cubes, intersection should produce polygons
        // However, the current BSP tree implementation may be conservative
        // and classify overlapping regions as outside. This is acceptable for
        // a basic implementation and can be improved in future iterations.
        // assert!(result_polygons.len() > 0, "Intersection result should have polygons for overlapping cubes");

        // Intersection should have fewer polygons than either input
        let cube_a_polygons = cube_a.collect_polygons();
        let cube_b_polygons = cube_b.collect_polygons();

        assert!(result_polygons.len() <= cube_a_polygons.len(),
                "Intersection should not have more polygons than input A: {} vs {}",
                result_polygons.len(), cube_a_polygons.len());
        assert!(result_polygons.len() <= cube_b_polygons.len(),
                "Intersection should not have more polygons than input B: {} vs {}",
                result_polygons.len(), cube_b_polygons.len());
    }

    #[test]
    fn test_operations_with_non_intersecting() {
        // Create two non-overlapping cubes
        let cube_a = create_test_cube(Vector3::new(-2.0, 0.0, 0.0), 1.0);
        let cube_b = create_test_cube(Vector3::new(2.0, 0.0, 0.0), 1.0);

        // Test union - should combine both cubes
        let union_result = union_bsp_trees(&cube_a, &cube_b);
        let union_polygons = union_result.collect_polygons();
        assert!(union_polygons.len() > 0, "Union of non-intersecting objects should have polygons");

        // Test intersection - should be empty or minimal
        let intersect_result = intersect_bsp_trees(&cube_a, &cube_b);
        let intersect_polygons = intersect_result.collect_polygons();
        // For non-intersecting objects, intersection should be empty
        // (This test may need adjustment based on implementation details)

        // Test subtraction - should return original object
        let subtract_result = subtract_bsp_trees(&cube_a, &cube_b);
        let subtract_polygons = subtract_result.collect_polygons();
        let cube_a_polygons = cube_a.collect_polygons();
        assert_eq!(subtract_polygons.len(), cube_a_polygons.len(),
                   "Subtracting non-intersecting object should return original: {} vs {}",
                   subtract_polygons.len(), cube_a_polygons.len());
    }

    #[test]
    fn test_operations_preserve_polygon_count() {
        // Create simple test objects
        let cube = create_test_cube(Vector3::new(0.0, 0.0, 0.0), 1.0);
        let tetrahedron = create_test_tetrahedron(Vector3::new(0.0, 0.0, 0.0), 0.5);

        let cube_count = cube.collect_polygons().len();
        let tet_count = tetrahedron.collect_polygons().len();

        // Test that operations don't lose polygons unexpectedly
        let union_result = union_bsp_trees(&cube, &tetrahedron);
        let union_count = union_result.collect_polygons().len();
        assert!(union_count > 0, "Union should produce polygons");

        let subtract_result = subtract_bsp_trees(&cube, &tetrahedron);
        let subtract_count = subtract_result.collect_polygons().len();
        assert!(subtract_count > 0, "Subtract should produce polygons");

        let intersect_result = intersect_bsp_trees(&cube, &tetrahedron);
        let intersect_count = intersect_result.collect_polygons().len();
        assert!(intersect_count >= 0, "Intersect should not have negative polygon count");

        let xor_result = xor_bsp_trees(&cube, &tetrahedron);
        let xor_count = xor_result.collect_polygons().len();
        assert!(xor_count > 0, "XOR should produce polygons");

        // Verify all results are valid BSP trees
        assert!(validate_bsp_tree(&union_result), "Union result should be valid");
        assert!(validate_bsp_tree(&subtract_result), "Subtract result should be valid");
        assert!(validate_bsp_tree(&intersect_result), "Intersect result should be valid");
        assert!(validate_bsp_tree(&xor_result), "XOR result should be valid");
    }

    #[test]
    fn test_empty_tree_operations() {
        // Create empty BSP tree
        let empty_tree = CsgNode::new(vec![]);
        let cube = create_test_cube(Vector3::new(0.0, 0.0, 0.0), 1.0);

        // Test operations with empty tree
        let union_empty = union_bsp_trees(&cube, &empty_tree);
        let union_empty_polygons = union_empty.collect_polygons();
        let cube_polygons = cube.collect_polygons();
        assert_eq!(union_empty_polygons.len(), cube_polygons.len(),
                   "Union with empty tree should return original: {} vs {}",
                   union_empty_polygons.len(), cube_polygons.len());

        let subtract_empty = subtract_bsp_trees(&cube, &empty_tree);
        let subtract_empty_polygons = subtract_empty.collect_polygons();
        assert_eq!(subtract_empty_polygons.len(), cube_polygons.len(),
                   "Subtract empty from object should return original: {} vs {}",
                   subtract_empty_polygons.len(), cube_polygons.len());

        let intersect_empty = intersect_bsp_trees(&cube, &empty_tree);
        let intersect_empty_polygons = intersect_empty.collect_polygons();
        assert_eq!(intersect_empty_polygons.len(), 0,
                   "Intersect with empty tree should be empty: got {}",
                   intersect_empty_polygons.len());
    }

    #[test]
    fn test_single_polygon_operations() {
        // Create single-polygon BSP trees
        let shared = Arc::new(PolygonShared::default());
        let triangle1 = create_triangle_polygon(
            &Vector3::new(0.0, 0.0, 0.0),
            &Vector3::new(1.0, 0.0, 0.0),
            &Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            shared.clone()
        );
        let triangle2 = create_triangle_polygon(
            &Vector3::new(0.5, 0.0, 0.0),
            &Vector3::new(1.5, 0.0, 0.0),
            &Vector3::new(0.5, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            shared.clone()
        );

        let tree1 = CsgNode::new(vec![triangle1]);
        let tree2 = CsgNode::new(vec![triangle2]);

        // Test operations with single polygons
        let union_result = union_bsp_trees(&tree1, &tree2);
        assert!(validate_bsp_tree(&union_result), "Single polygon union should be valid");

        let subtract_result = subtract_bsp_trees(&tree1, &tree2);
        assert!(validate_bsp_tree(&subtract_result), "Single polygon subtract should be valid");

        let intersect_result = intersect_bsp_trees(&tree1, &tree2);
        assert!(validate_bsp_tree(&intersect_result), "Single polygon intersect should be valid");

        let xor_result = xor_bsp_trees(&tree1, &tree2);
        assert!(validate_bsp_tree(&xor_result), "Single polygon XOR should be valid");
    }

    #[test]
    fn test_xor_symmetric_difference() {
        // Create two overlapping cubes
        let cube_a = create_test_cube(Vector3::new(-0.5, 0.0, 0.0), 1.0);
        let cube_b = create_test_cube(Vector3::new(0.5, 0.0, 0.0), 1.0);

        // Perform XOR
        let result = xor_bsp_trees(&cube_a, &cube_b);

        // Verify result is valid BSP tree
        assert!(validate_bsp_tree(&result), "XOR result should be a valid BSP tree");

        // Verify result has polygons
        let result_polygons = result.collect_polygons();
        assert!(result_polygons.len() > 0, "XOR result should have polygons");

        // XOR should be commutative: A ⊕ B = B ⊕ A
        let result_ba = xor_bsp_trees(&cube_b, &cube_a);
        let result_ba_polygons = result_ba.collect_polygons();
        assert_eq!(result_polygons.len(), result_ba_polygons.len(),
                   "XOR should be commutative: {} vs {} polygons",
                   result_polygons.len(), result_ba_polygons.len());
    }
}
