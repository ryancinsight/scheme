//! src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("The simulation's linear system could not be solved. This may be due to disconnected channels or other geometry issues.")]
    LinearSystemError,
} 