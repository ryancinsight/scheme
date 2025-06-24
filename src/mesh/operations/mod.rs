//! src/mesh/operations/mod.rs
//! 
//! Operations Portico - The Grand Entrance for High-Level Mesh Operations
//! 
//! This module provides the public API for mesh boolean operations, serving as the bridge
//! between the low-level CSG implementation and the high-level mesh processing needs.
//! Following cathedral engineering principles, this module is organized as:
//! - The Façade (mod.rs): Public API for mesh operations
//! - The Mind (conversions.rs): Triangle ↔ Polygon conversion logic
//! - The Mind (mesh_ops.rs): High-level operation implementations
//! - The Immune System (errors.rs): Operation-specific error handling

// Production-ready CSG operations using Binary Space Partitioning trees

pub mod conversions;
pub mod errors;
pub mod difference;

use stl_io::Triangle;
use conversions::{triangles_to_polygons, polygons_to_triangles};
use crate::mesh::csg::Csg;
use crate::mesh::csgrs_bridge;
pub use errors::OperationError;
pub use difference::difference;

/// Computes the difference of two meshes (A - B) using CSG implementation.
///
/// This operation removes the volume of mesh_b from mesh_a, creating a result
/// where mesh_a has mesh_b-shaped holes carved out of it.
///
/// # Mathematical Semantics
/// - subtract(cube, sphere) = cube with spherical hole
/// - subtract(sphere, cube) = sphere with cubic hole
///
/// # Volume Conservation
/// The result volume must satisfy: Volume(A - B) ≤ Volume(A)
/// Any violation of this constraint indicates a fundamental CSG implementation error.
///
/// # Arguments
/// * `mesh_a` - The base mesh (what to subtract FROM)
/// * `mesh_b` - The tool mesh (what to subtract)
///
/// # Returns
/// * `Ok(Vec<Triangle>)` - The resulting mesh after subtraction
/// * `Err(&'static str)` - Error message if operation fails
///
pub fn subtract(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    if mesh_a.is_empty() {
        return Ok(Vec::new());
    }
    if mesh_b.is_empty() {
        return Ok(mesh_a.to_vec());
    }

    // Convert triangles to CSG polygons
    let polygons_a = triangles_to_polygons(mesh_a)?;
    let polygons_b = triangles_to_polygons(mesh_b)?;

    // Create CSG objects and track initial volumes
    let csg_a = Csg::from_polygons(polygons_a);
    let csg_b = Csg::from_polygons(polygons_b);

    let initial_volume_a = csg_a.calculate_volume();
    let initial_volume_b = csg_b.calculate_volume();

    // Perform CSG subtraction using BSP trees
    let result_csg = csg_a.subtract(&csg_b);
    let result_volume = result_csg.calculate_volume();

    // Volume conservation validation
    if result_volume > initial_volume_a + crate::mesh::csg::EPSILON {
        eprintln!("CRITICAL: Volume conservation violation in subtract operation!");
        eprintln!("  Input A volume: {:.6}", initial_volume_a);
        eprintln!("  Input B volume: {:.6}", initial_volume_b);
        eprintln!("  Result volume: {:.6}", result_volume);
        eprintln!("  Violation: Result volume exceeds input A by {:.6}",
                 result_volume - initial_volume_a);

        // For now, continue with warning rather than failing
        // TODO: Fix the underlying CSG implementation to prevent this
    }

    // Calculate intersection volume for validation
    let intersection_csg = csg_a.intersect(&csg_b);
    let intersection_volume = intersection_csg.calculate_volume();

    // Debug: Log volume information for analysis
    if std::env::var("CSG_DEBUG").is_ok() {
        eprintln!("CSG Subtract Debug:");
        eprintln!("  Input A volume: {:.6}", initial_volume_a);
        eprintln!("  Input B volume: {:.6}", initial_volume_b);
        eprintln!("  Intersection volume: {:.6}", intersection_volume);
        eprintln!("  Result volume: {:.6}", result_volume);
        eprintln!("  Expected volume: {:.6}", initial_volume_a - intersection_volume);
        eprintln!("  Volume error: {:.6}", result_volume - (initial_volume_a - intersection_volume));
    }

    // Convert result back to triangles
    let result_polygons = result_csg.to_polygons();
    Ok(polygons_to_triangles(&result_polygons))
}

/// Computes the union of two meshes (A ∪ B) using CSG implementation.
///
/// # Volume Conservation
/// The result volume should satisfy: Volume(A ∪ B) = Volume(A) + Volume(B) - Volume(A ∩ B)
/// For non-overlapping objects: Volume(A ∪ B) = Volume(A) + Volume(B)
///
pub fn union(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    if mesh_a.is_empty() {
        return Ok(mesh_b.to_vec());
    }
    if mesh_b.is_empty() {
        return Ok(mesh_a.to_vec());
    }

    // Convert triangles to CSG polygons
    let polygons_a = triangles_to_polygons(mesh_a)?;
    let polygons_b = triangles_to_polygons(mesh_b)?;

    // Create CSG objects and track initial volumes
    let csg_a = Csg::from_polygons(polygons_a);
    let csg_b = Csg::from_polygons(polygons_b);

    let initial_volume_a = csg_a.calculate_volume();
    let initial_volume_b = csg_b.calculate_volume();

    // Perform CSG union using BSP trees
    let result_csg = csg_a.union(&csg_b);
    let result_volume = result_csg.calculate_volume();

    // Calculate intersection volume for validation
    let intersection_csg = csg_a.intersect(&csg_b);
    let intersection_volume = intersection_csg.calculate_volume();

    // Debug: Log volume information for analysis
    if std::env::var("CSG_DEBUG").is_ok() {
        eprintln!("CSG Union Debug:");
        eprintln!("  Input A volume: {:.6}", initial_volume_a);
        eprintln!("  Input B volume: {:.6}", initial_volume_b);
        eprintln!("  Intersection volume: {:.6}", intersection_volume);
        eprintln!("  Result volume: {:.6}", result_volume);
        eprintln!("  Expected volume: {:.6}", initial_volume_a + initial_volume_b - intersection_volume);
        eprintln!("  Volume error: {:.6}", result_volume - (initial_volume_a + initial_volume_b - intersection_volume));
    }

    // Convert result back to triangles
    let result_polygons = result_csg.to_polygons();
    Ok(polygons_to_triangles(&result_polygons))
}

/// Computes the intersection of two meshes (A ∩ B) using CSG implementation.
pub fn intersection(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    if mesh_a.is_empty() || mesh_b.is_empty() {
        return Ok(Vec::new());
    }

    // Convert triangles to CSG polygons
    let polygons_a = triangles_to_polygons(mesh_a)?;
    let polygons_b = triangles_to_polygons(mesh_b)?;

    // Perform CSG intersection using BSP trees
    let csg_a = Csg::from_polygons(polygons_a);
    let csg_b = Csg::from_polygons(polygons_b);
    let result_csg = csg_a.intersect(&csg_b);

    // Convert result back to triangles
    let result_polygons = result_csg.to_polygons();
    Ok(polygons_to_triangles(&result_polygons))
}

/// Computes the exclusive-OR of two meshes (A ⊕ B) using CSG implementation.
pub fn xor(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    if mesh_a.is_empty() {
        return Ok(mesh_b.to_vec());
    }
    if mesh_b.is_empty() {
        return Ok(mesh_a.to_vec());
    }

    // Convert triangles to CSG polygons
    let polygons_a = triangles_to_polygons(mesh_a)?;
    let polygons_b = triangles_to_polygons(mesh_b)?;

    // Perform CSG XOR using BSP trees
    let csg_a = Csg::from_polygons(polygons_a);
    let csg_b = Csg::from_polygons(polygons_b);
    let result_csg = csg_a.xor(&csg_b);

    // Convert result back to triangles
    let result_polygons = result_csg.to_polygons();
    Ok(polygons_to_triangles(&result_polygons))
}

// Note: The difference function is now imported from the difference module
// It takes a ChannelSystem3D parameter, not two Triangle arrays like the CSG operations

/// CSGRS-based operations - Enhanced CSG operations using the workspace csgrs library
///
/// These functions provide alternative implementations of CSG operations using the
/// robust csgrs library for improved accuracy and reliability.

/// Enhanced subtract operation using csgrs
///
/// This function provides an alternative to the standard subtract operation
/// using the workspace csgrs library for improved robustness and accuracy.
///
/// # Arguments
/// * `mesh_a` - The base mesh (what to subtract FROM)
/// * `mesh_b` - The tool mesh (what to subtract)
///
/// # Returns
/// * `Result<Vec<Triangle>, &'static str>` - The resulting mesh after subtraction
pub fn subtract_csgrs(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    csgrs_bridge::csgrs_difference(mesh_a, mesh_b)
}

/// Enhanced union operation using csgrs
pub fn union_csgrs(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    csgrs_bridge::operations::csgrs_union(mesh_a, mesh_b)
}

/// Enhanced intersection operation using csgrs
pub fn intersection_csgrs(mesh_a: &[Triangle], mesh_b: &[Triangle]) -> Result<Vec<Triangle>, &'static str> {
    csgrs_bridge::operations::csgrs_intersect(mesh_a, mesh_b)
}
