//! Central parameter registry providing single source of truth (SSOT)
//!
//! This module implements the central parameter registry that manages all
//! parameter managers and provides a unified interface for parameter access
//! and modification across the entire microfluidic design system.

use crate::state_management::{
    managers::{
        ParameterManager, SerpentineParameterManager, ArcParameterManager,
        GeometryParameterManager, CollisionParameterManager, SymmetryParameterManager,
    },
    errors::{RegistryError, StateManagementError, StateManagementResult},
    validation::ValidationRuleSet,
};
use std::collections::HashMap;

/// Updates to be applied to parameters
#[derive(Debug)]
pub struct ParameterUpdates {
    /// Updates organized by domain and parameter name
    pub updates: HashMap<String, HashMap<String, ParameterUpdate>>,
    
    /// Reason for the updates
    pub reason: String,
}

/// Individual parameter update
#[derive(Debug)]
pub struct ParameterUpdate {
    /// New value for the parameter
    pub value: Box<dyn std::any::Any + Send + Sync>,
    
    /// Whether to validate this update
    pub validate: bool,
}

impl ParameterUpdates {
    /// Create a new parameter updates collection
    pub fn new(reason: &str) -> Self {
        Self {
            updates: HashMap::new(),
            reason: reason.to_string(),
        }
    }
    
    /// Add an update for a specific domain and parameter
    pub fn add_update<T: 'static + Send + Sync>(
        mut self,
        domain: &str,
        parameter: &str,
        value: T,
        validate: bool,
    ) -> Self {
        let domain_updates = self.updates.entry(domain.to_string()).or_insert_with(HashMap::new);
        domain_updates.insert(
            parameter.to_string(),
            ParameterUpdate {
                value: Box::new(value),
                validate,
            },
        );
        self
    }
    
    /// Get all domains that have updates
    pub fn domains(&self) -> Vec<&String> {
        self.updates.keys().collect()
    }
    
    /// Get updates for a specific domain
    pub fn domain_updates(&self, domain: &str) -> Option<&HashMap<String, ParameterUpdate>> {
        self.updates.get(domain)
    }
}

/// Central registry for all design parameters
#[derive(Debug)]
pub struct ParameterRegistry {
    /// Serpentine parameter manager
    serpentine_manager: SerpentineParameterManager,
    
    /// Arc parameter manager
    arc_manager: ArcParameterManager,
    
    /// Geometry parameter manager
    geometry_manager: GeometryParameterManager,
    
    /// Collision parameter manager
    collision_manager: CollisionParameterManager,
    
    /// Symmetry parameter manager
    symmetry_manager: SymmetryParameterManager,
    
    /// Global validation rules
    global_validation_rules: ValidationRuleSet,
    
    /// Whether validation is enabled
    validation_enabled: bool,
    
    /// Registry lock state
    is_locked: bool,
    
    /// Registry version for change tracking
    version: u64,
}

impl ParameterRegistry {
    /// Create a new parameter registry with default managers
    pub fn new() -> StateManagementResult<Self> {
        Ok(Self {
            serpentine_manager: SerpentineParameterManager::new(),
            arc_manager: ArcParameterManager::new(),
            geometry_manager: GeometryParameterManager::new(),
            collision_manager: CollisionParameterManager::new(),
            symmetry_manager: SymmetryParameterManager::new(),
            global_validation_rules: ValidationRuleSet::new(),
            validation_enabled: true,
            is_locked: false,
            version: 0,
        })
    }
    
    /// Create a registry with default configuration
    pub fn with_defaults() -> StateManagementResult<Self> {
        let mut registry = Self::new()?;
        registry.initialize_default_validation_rules();
        Ok(registry)
    }
    
    /// Initialize default validation rules
    fn initialize_default_validation_rules(&mut self) {
        // Add cross-domain validation rules here if needed
        // For now, each manager handles its own validation
    }
    
    /// Get the serpentine parameter manager
    pub fn serpentine(&self) -> &SerpentineParameterManager {
        &self.serpentine_manager
    }
    
    /// Get mutable serpentine parameter manager
    pub fn serpentine_mut(&mut self) -> StateManagementResult<&mut SerpentineParameterManager> {
        self.check_not_locked()?;
        Ok(&mut self.serpentine_manager)
    }
    
    /// Get the arc parameter manager
    pub fn arc(&self) -> &ArcParameterManager {
        &self.arc_manager
    }
    
    /// Get mutable arc parameter manager
    pub fn arc_mut(&mut self) -> StateManagementResult<&mut ArcParameterManager> {
        self.check_not_locked()?;
        Ok(&mut self.arc_manager)
    }
    
    /// Get the geometry parameter manager
    pub fn geometry(&self) -> &GeometryParameterManager {
        &self.geometry_manager
    }
    
    /// Get mutable geometry parameter manager
    pub fn geometry_mut(&mut self) -> StateManagementResult<&mut GeometryParameterManager> {
        self.check_not_locked()?;
        Ok(&mut self.geometry_manager)
    }
    
    /// Get the collision parameter manager
    pub fn collision(&self) -> &CollisionParameterManager {
        &self.collision_manager
    }
    
    /// Get the symmetry parameter manager
    pub fn symmetry(&self) -> &SymmetryParameterManager {
        &self.symmetry_manager
    }
    
    /// Get parameter manager by domain name
    pub fn get_manager(&self, domain: &str) -> StateManagementResult<&dyn ParameterManager> {
        match domain {
            "serpentine" => Ok(&self.serpentine_manager),
            "arc" => Ok(&self.arc_manager),
            "geometry" => Ok(&self.geometry_manager),
            "collision" => Ok(&self.collision_manager),
            "symmetry" => Ok(&self.symmetry_manager),
            _ => Err(StateManagementError::Registry(
                RegistryError::ManagerNotFound {
                    domain: domain.to_string(),
                }
            )),
        }
    }
    
    /// Get mutable parameter manager by domain name
    pub fn get_manager_mut(&mut self, domain: &str) -> StateManagementResult<&mut dyn ParameterManager> {
        self.check_not_locked()?;
        match domain {
            "serpentine" => Ok(&mut self.serpentine_manager),
            "arc" => Ok(&mut self.arc_manager),
            "geometry" => Ok(&mut self.geometry_manager),
            "collision" => Ok(&mut self.collision_manager),
            "symmetry" => Ok(&mut self.symmetry_manager),
            _ => Err(StateManagementError::Registry(
                RegistryError::ManagerNotFound {
                    domain: domain.to_string(),
                }
            )),
        }
    }
    
    /// Validate all parameters across all managers
    pub fn validate_all(&self) -> StateManagementResult<()> {
        if !self.validation_enabled {
            return Ok(());
        }
        
        // Validate each manager
        self.serpentine_manager.validate_all()
            .map_err(StateManagementError::Parameter)?;
        self.arc_manager.validate_all()
            .map_err(StateManagementError::Parameter)?;
        self.geometry_manager.validate_all()
            .map_err(StateManagementError::Parameter)?;
        self.collision_manager.validate_all()
            .map_err(StateManagementError::Parameter)?;
        self.symmetry_manager.validate_all()
            .map_err(StateManagementError::Parameter)?;
        
        // Validate global rules
        // TODO: Implement cross-manager validation
        
        Ok(())
    }
    
    /// Update parameters using the provided updates
    pub fn update_parameters(&mut self, updates: ParameterUpdates) -> StateManagementResult<()> {
        self.check_not_locked()?;
        
        // Apply updates to each domain
        for (domain, domain_updates) in updates.updates {
            let manager = self.get_manager_mut(&domain)?;
            
            for (parameter, update) in domain_updates {
                manager.set_parameter(&parameter, update.value, &updates.reason)
                    .map_err(StateManagementError::Parameter)?;
            }
        }
        
        // Validate if requested
        if self.validation_enabled {
            self.validate_all()?;
        }
        
        // Increment version
        self.version += 1;
        
        Ok(())
    }
    
    /// Enable or disable validation
    pub fn enable_validation(&mut self, enabled: bool) {
        self.validation_enabled = enabled;
    }
    
    /// Check if validation is enabled
    pub fn is_validation_enabled(&self) -> bool {
        self.validation_enabled
    }
    
    /// Lock the registry to prevent modifications
    pub fn lock(&mut self) {
        self.is_locked = true;
    }
    
    /// Unlock the registry to allow modifications
    pub fn unlock(&mut self) {
        self.is_locked = false;
    }
    
    /// Check if the registry is locked
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }
    
    /// Get the current registry version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Get all domain names
    pub fn domain_names(&self) -> Vec<&str> {
        vec!["serpentine", "arc", "geometry", "collision", "symmetry"]
    }
    
    /// Get all parameter names across all domains
    pub fn all_parameter_names(&self) -> HashMap<String, Vec<String>> {
        let mut all_params = HashMap::new();
        
        for domain in self.domain_names() {
            if let Ok(manager) = self.get_manager(domain) {
                all_params.insert(domain.to_string(), manager.parameter_names());
            }
        }
        
        all_params
    }
    
    /// Reset all parameters to defaults
    pub fn reset_all(&mut self, reason: &str) -> StateManagementResult<()> {
        self.check_not_locked()?;
        
        self.serpentine_manager.reset_all(reason)
            .map_err(StateManagementError::Parameter)?;
        self.arc_manager.reset_all(reason)
            .map_err(StateManagementError::Parameter)?;
        self.geometry_manager.reset_all(reason)
            .map_err(StateManagementError::Parameter)?;
        self.collision_manager.reset_all(reason)
            .map_err(StateManagementError::Parameter)?;
        self.symmetry_manager.reset_all(reason)
            .map_err(StateManagementError::Parameter)?;
        
        self.version += 1;
        Ok(())
    }
    
    /// Check if registry is not locked, return error if locked
    fn check_not_locked(&self) -> StateManagementResult<()> {
        if self.is_locked {
            Err(StateManagementError::Registry(RegistryError::RegistryLocked))
        } else {
            Ok(())
        }
    }
}

impl Default for ParameterRegistry {
    fn default() -> Self {
        Self::with_defaults().expect("Failed to create default parameter registry")
    }
}
