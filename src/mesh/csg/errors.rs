//! src/mesh/csg/errors.rs
//! 
//! CSG Error Types - The Immune System for the CSG Chapel
//! 
//! This module defines error types specific to CSG operations, providing
//! clear diagnostics for geometric and computational failures.

use thiserror::Error;

/// Errors that can occur during CSG operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CsgError {
    /// Invalid polygon with insufficient vertices
    #[error("Invalid polygon: requires at least 3 vertices, got {vertex_count}")]
    InvalidPolygon { vertex_count: usize },

    /// Degenerate triangle with zero or near-zero area
    #[error("Degenerate triangle: area is below threshold")]
    DegenerateTriangle,

    /// Coplanar vertices that cannot form a valid plane
    #[error("Cannot create plane from coplanar points")]
    CoplanarPoints,

    /// Numerical instability in geometric calculations
    #[error("Numerical instability detected in {operation}")]
    NumericalInstability { operation: String },

    /// BSP tree construction failed
    #[error("BSP tree construction failed: {reason}")]
    BspTreeError { reason: String },

    /// Invalid normal vector (zero length or NaN)
    #[error("Invalid normal vector: {reason}")]
    InvalidNormal { reason: String },
}

impl CsgError {
    /// Create an invalid polygon error
    pub fn invalid_polygon(vertex_count: usize) -> Self {
        Self::InvalidPolygon { vertex_count }
    }

    /// Create a numerical instability error
    pub fn numerical_instability(operation: impl Into<String>) -> Self {
        Self::NumericalInstability {
            operation: operation.into(),
        }
    }

    /// Create a BSP tree error
    pub fn bsp_tree_error(reason: impl Into<String>) -> Self {
        Self::BspTreeError {
            reason: reason.into(),
        }
    }

    /// Create an invalid normal error
    pub fn invalid_normal(reason: impl Into<String>) -> Self {
        Self::InvalidNormal {
            reason: reason.into(),
        }
    }
}
