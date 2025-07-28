# Centralized Parameter Management System Design

## Overview

This document outlines the design for a unified parameter management system that follows SOLID, CUPID, GRASP, DRY, KISS, and YAGNI principles while providing a single source of truth (SSOT) for all design parameters.

## Design Principles Applied

### SOLID Principles
- **SRP**: Each parameter manager handles one specific domain
- **OCP**: Extensible through traits without modifying existing code
- **LSP**: All parameter managers implement common interfaces
- **ISP**: Focused interfaces for specific parameter types
- **DIP**: Depend on abstractions, not concrete implementations

### CUPID Principles
- **Composable**: Parameter managers can be combined and reused
- **Unix Philosophy**: Each component does one thing well
- **Predictable**: Consistent behavior across all parameter types
- **Idiomatic**: Follows Rust best practices
- **Domain-centric**: Organized around microfluidic design concepts

## Core Architecture

### 1. Parameter Management Traits

```rust
/// Core trait for all parameter managers
pub trait ParameterManager<T> {
    type Error: std::error::Error;
    
    /// Validate parameter values
    fn validate(&self) -> Result<(), Self::Error>;
    
    /// Get parameter value with validation
    fn get(&self, key: &str) -> Result<T, Self::Error>;
    
    /// Set parameter value with validation
    fn set(&mut self, key: &str, value: T) -> Result<(), Self::Error>;
    
    /// Get all parameter names
    fn parameter_names(&self) -> Vec<&'static str>;
    
    /// Check if parameter exists
    fn has_parameter(&self, key: &str) -> bool;
}

/// Trait for parameters that can be adapted based on context
pub trait AdaptiveParameter<T, Context> {
    /// Calculate adaptive value based on context
    fn adapt(&self, base_value: T, context: &Context) -> T;
    
    /// Check if adaptation is enabled
    fn is_adaptive(&self) -> bool;
}

/// Trait for parameters with interdependencies
pub trait DependentParameter<T> {
    type Dependencies;
    
    /// Calculate value based on dependencies
    fn calculate_with_dependencies(&self, deps: &Self::Dependencies) -> T;
    
    /// Get list of parameter dependencies
    fn dependencies(&self) -> Vec<&'static str>;
}
```

### 2. Centralized Parameter Registry

```rust
/// Central registry for all design parameters
pub struct ParameterRegistry {
    serpentine_params: SerpentineParameterManager,
    arc_params: ArcParameterManager,
    geometry_params: GeometryParameterManager,
    collision_params: CollisionParameterManager,
    symmetry_params: SymmetryParameterManager,
    validation_rules: ValidationRuleSet,
}

impl ParameterRegistry {
    /// Create new registry with default parameters
    pub fn new() -> Self;
    
    /// Create registry from configuration
    pub fn from_config(config: &ParameterConfig) -> Result<Self, ParameterError>;
    
    /// Validate all parameters and their relationships
    pub fn validate_all(&self) -> Result<(), ParameterError>;
    
    /// Get parameter manager for specific domain
    pub fn serpentine(&self) -> &SerpentineParameterManager;
    pub fn arc(&self) -> &ArcParameterManager;
    pub fn geometry(&self) -> &GeometryParameterManager;
    pub fn collision(&self) -> &CollisionParameterManager;
    pub fn symmetry(&self) -> &SymmetryParameterManager;
    
    /// Update parameters with validation
    pub fn update_parameters(&mut self, updates: ParameterUpdates) -> Result<(), ParameterError>;
}
```

### 3. Domain-Specific Parameter Managers

#### Serpentine Parameter Manager
```rust
pub struct SerpentineParameterManager {
    wave_params: WaveParameterSet,
    adaptive_params: AdaptiveParameterSet,
    optimization_params: OptimizationParameterSet,
    context: Option<ChannelGenerationContext>,
}

pub struct WaveParameterSet {
    pub amplitude_base: ConfigurableParameter<f64>,
    pub wavelength_factor: ConfigurableParameter<f64>,
    pub frequency_multiplier: ConfigurableParameter<f64>,
    pub phase_offset: ConfigurableParameter<f64>,
    pub wave_shape: ConfigurableParameter<WaveShape>,
    pub gaussian_width_factor: ConfigurableParameter<f64>,
    pub wave_density_factor: ConfigurableParameter<f64>,
}

impl ParameterManager<f64> for SerpentineParameterManager {
    // Implementation with proper validation and error handling
}
```

#### Arc Parameter Manager
```rust
pub struct ArcParameterManager {
    bezier_params: BezierParameterSet,
    curvature_params: CurvatureParameterSet,
    collision_params: CollisionParameterSet,
}

pub struct BezierParameterSet {
    pub control_point_offset: ConfigurableParameter<f64>,
    pub tension_factor: ConfigurableParameter<f64>,
    pub smoothness_points: ConfigurableParameter<usize>,
    pub curve_direction: ConfigurableParameter<f64>,
}

pub struct CurvatureParameterSet {
    pub base_curvature: ConfigurableParameter<f64>,
    pub adaptive_reduction: ConfigurableParameter<f64>,
    pub min_curvature: ConfigurableParameter<f64>,
    pub max_curvature: ConfigurableParameter<f64>,
}
```

### 4. Configurable Parameter Type

```rust
/// Generic configurable parameter with validation and adaptation
pub struct ConfigurableParameter<T> {
    value: T,
    default: T,
    constraints: ParameterConstraints<T>,
    adaptive_behavior: Option<Box<dyn AdaptiveParameter<T, ChannelGenerationContext>>>,
    dependencies: Vec<String>,
    metadata: ParameterMetadata,
}

impl<T> ConfigurableParameter<T> 
where 
    T: Clone + PartialEq + std::fmt::Debug,
{
    pub fn new(value: T, constraints: ParameterConstraints<T>) -> Self;
    pub fn with_adaptive_behavior<A>(mut self, behavior: A) -> Self 
    where A: AdaptiveParameter<T, ChannelGenerationContext> + 'static;
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self;
    
    /// Get current value with context-based adaptation
    pub fn get_value(&self, context: Option<&ChannelGenerationContext>) -> T;
    
    /// Set value with validation
    pub fn set_value(&mut self, value: T) -> Result<(), ParameterError>;
    
    /// Validate current value
    pub fn validate(&self) -> Result<(), ParameterError>;
    
    /// Reset to default value
    pub fn reset(&mut self);
}
```

### 5. Parameter Constraints System

```rust
/// Constraint system for parameter validation
pub enum ParameterConstraints<T> {
    Range { min: T, max: T },
    Set(Vec<T>),
    Custom(Box<dyn Fn(&T) -> Result<(), String>>),
    Composite(Vec<ParameterConstraints<T>>),
}

impl<T> ParameterConstraints<T> 
where 
    T: PartialOrd + Clone,
{
    pub fn validate(&self, value: &T) -> Result<(), ParameterError>;
    pub fn range(min: T, max: T) -> Self;
    pub fn positive() -> Self where T: Default + PartialOrd;
    pub fn non_zero() -> Self where T: Default + PartialEq;
}
```

## Implementation Strategy

### Phase 1: Core Infrastructure
1. Implement base traits and parameter registry
2. Create configurable parameter type with validation
3. Implement constraint system
4. Add comprehensive error handling

### Phase 2: Domain-Specific Managers
1. Implement serpentine parameter manager
2. Implement arc parameter manager
3. Implement geometry parameter manager
4. Add collision detection parameter management

### Phase 3: Integration and Adaptation
1. Integrate with existing strategy pattern
2. Implement adaptive parameter behaviors
3. Add parameter dependency management
4. Ensure backward compatibility

### Phase 4: Advanced Features
1. Add symmetry parameter management
2. Implement parameter optimization
3. Add configuration serialization/deserialization
4. Create parameter validation UI/CLI tools

## Benefits

### Single Source of Truth (SSOT)
- All parameters managed through central registry
- Consistent validation and error handling
- Unified configuration interface

### Improved Maintainability
- Clear separation of concerns
- Extensible through traits
- Reduced code duplication

### Enhanced Configurability
- All hardcoded values become configurable
- Context-aware parameter adaptation
- Flexible constraint system

### Better Error Handling
- Comprehensive parameter validation
- Clear error messages with context
- Graceful error recovery

### Performance Optimization
- Reduced memory allocation through parameter reuse
- Efficient validation caching
- Optimized parameter access patterns

## Migration Strategy

### Backward Compatibility
- Existing configuration structs remain functional
- Gradual migration path for existing code
- Deprecation warnings for old patterns

### Testing Strategy
- Comprehensive unit tests for all parameter managers
- Integration tests for parameter relationships
- Performance benchmarks for parameter access
- Validation tests for all constraint types

This design provides a solid foundation for centralized parameter management while maintaining the flexibility and extensibility required for the millifluidic design system.
