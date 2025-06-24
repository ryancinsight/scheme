//! src/mesh/operations/errors.rs
//! 
//! Operation Error Types - The Immune System for the Operations Portico
//! 
//! This module defines error types specific to high-level mesh operations,
//! providing clear diagnostics for conversion and operation failures.

use thiserror::Error;
use crate::mesh::csg::CsgError;

/// Errors that can occur during mesh operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OperationError {
    /// Error during triangle to polygon conversion
    #[error("Triangle conversion failed: {reason}")]
    ConversionError { reason: String },

    /// Error during CSG operation
    #[error("CSG operation failed: {source}")]
    CsgError {
        #[from]
        source: CsgError,
    },

    /// Empty mesh provided where non-empty mesh expected
    #[error("Empty mesh provided for operation: {operation}")]
    EmptyMesh { operation: String },

    /// Invalid triangle data
    #[error("Invalid triangle: {reason}")]
    InvalidTriangle { reason: String },

    /// Operation not supported
    #[error("Operation not supported: {operation}")]
    UnsupportedOperation { operation: String },
}

impl OperationError {
    /// Create a conversion error
    pub fn conversion_error(reason: impl Into<String>) -> Self {
        Self::ConversionError {
            reason: reason.into(),
        }
    }

    /// Create an empty mesh error
    pub fn empty_mesh(operation: impl Into<String>) -> Self {
        Self::EmptyMesh {
            operation: operation.into(),
        }
    }

    /// Create an invalid triangle error
    pub fn invalid_triangle(reason: impl Into<String>) -> Self {
        Self::InvalidTriangle {
            reason: reason.into(),
        }
    }

    /// Create an unsupported operation error
    pub fn unsupported_operation(operation: impl Into<String>) -> Self {
        Self::UnsupportedOperation {
            operation: operation.into(),
        }
    }
}
