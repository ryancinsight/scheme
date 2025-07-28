//! State Management Module
//!
//! This module provides centralized state management for all design parameters
//! in the millifluidic design system. It implements a unified parameter management
//! system following SOLID, CUPID, GRASP, DRY, KISS, and YAGNI principles.
//!
//! # Architecture
//!
//! The state management system is built around several key components:
//!
//! - **Parameter Managers**: Domain-specific managers for different parameter types
//! - **Parameter Registry**: Central registry providing single source of truth (SSOT)
//! - **Configurable Parameters**: Type-safe parameters with validation and adaptation
//! - **Constraint System**: Flexible validation framework for parameter values
//! - **Adaptive Behaviors**: Context-aware parameter adjustment capabilities
//!
//! # Design Principles
//!
//! - **Single Responsibility**: Each manager handles one specific parameter domain
//! - **Open/Closed**: Extensible through traits without modifying existing code
//! - **Dependency Inversion**: Depends on abstractions, not concrete implementations
//! - **Composable**: Parameter managers can be combined and reused
//! - **Predictable**: Consistent behavior across all parameter types

pub mod bilateral_symmetry;
pub mod constraints;
pub mod context_consolidation;
pub mod error_handling_enhancements;
pub mod parameters;
pub mod registry;
pub mod managers;
pub mod adaptive;
pub mod symmetry_integration;
pub mod validation;
pub mod errors;

#[cfg(test)]
mod integration_test;

pub use self::{
    constraints::ParameterConstraints,
    parameters::{ConfigurableParameter, ParameterMetadata},
    registry::ParameterRegistry,
    managers::{
        ParameterManager, SerpentineParameterManager, ArcParameterManager,
        GeometryParameterManager, CollisionParameterManager, SymmetryParameterManager
    },
    adaptive::{AdaptiveParameter, ChannelGenerationContext},
    validation::{ValidationRuleSet, ValidationRule},
    errors::{ParameterError, StateManagementError, StateManagementResult, ConstraintError},
};

/// Re-export commonly used types for convenience
pub type ParameterResult<T> = Result<T, ParameterError>;
pub type StateResult<T> = Result<T, StateManagementError>;

/// Version information for the state management system
pub const STATE_MANAGEMENT_VERSION: &str = "1.0.0";

/// Default configuration for parameter management
pub struct DefaultParameterConfig;

impl DefaultParameterConfig {
    /// Create default parameter registry
    pub fn create_registry() -> StateResult<ParameterRegistry> {
        ParameterRegistry::with_defaults()
    }
    
    /// Create registry with validation enabled
    pub fn create_validated_registry() -> StateResult<ParameterRegistry> {
        let mut registry = Self::create_registry()?;
        registry.enable_validation(true);
        registry.validate_all()?;
        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_creation() {
        let registry = DefaultParameterConfig::create_registry();
        assert!(registry.is_ok());
    }

    #[test]
    fn test_validated_registry_creation() {
        let registry = DefaultParameterConfig::create_validated_registry();
        assert!(registry.is_ok());
    }
}
