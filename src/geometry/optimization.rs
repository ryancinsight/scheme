//! Optimization utilities for serpentine channel generation
//!
//! This module provides utilities for optimizing serpentine channel parameters
//! to maximize channel length while maintaining proper wall clearance and
//! multi-channel compatibility.

use crate::geometry::types::Point2D;
use crate::config::{GeometryConfig, SerpentineConfig, OptimizationProfile};
use crate::config_constants::ConstantsRegistry;

/// Optimization algorithm constants
mod constants {
    /// Minimum path length threshold for valid optimization results
    pub const MIN_PATH_LENGTH_THRESHOLD: f64 = 0.0;

    /// Penalty multiplier for constraint violations
    pub const CONSTRAINT_PENALTY_MULTIPLIER: f64 = 1000.0;

    /// Small penalty tolerance for validity checking
    pub const PENALTY_TOLERANCE: f64 = 1.0;

    /// Nelder-Mead algorithm coefficients
    pub const REFLECTION_COEFFICIENT: f64 = 1.0;
    pub const EXPANSION_COEFFICIENT: f64 = 2.0;
    pub const CONTRACTION_COEFFICIENT: f64 = 0.5;
    pub const SHRINK_COEFFICIENT: f64 = 0.5;

    /// Parameter bounds for optimization
    pub const MIN_WAVELENGTH_FACTOR: f64 = 0.5;
    pub const MAX_WAVELENGTH_FACTOR: f64 = 5.0;
    pub const MIN_WAVE_DENSITY_FACTOR: f64 = 0.5;
    pub const MAX_WAVE_DENSITY_FACTOR: f64 = 5.0;
    pub const MIN_FILL_FACTOR: f64 = 0.1;
    pub const MAX_FILL_FACTOR: f64 = 0.95;

    /// Simplex initialization perturbation factors
    pub const WAVELENGTH_PERTURBATION: f64 = 1.1;
    pub const WAVE_DENSITY_PERTURBATION: f64 = 1.1;
    pub const FILL_FACTOR_PERTURBATION: f64 = 1.05;

    /// Wave shape parameters
    pub const SQUARE_WAVE_SHARPNESS: f64 = 5.0;

    /// Envelope calculation constants
    pub const SMOOTHSTEP_COEFFICIENT_1: f64 = 3.0;
    pub const SMOOTHSTEP_COEFFICIENT_2: f64 = 2.0;
    pub const GAUSSIAN_CENTER: f64 = 0.5;
    pub const GAUSSIAN_EXPONENT_COEFFICIENT: f64 = -0.5;
    pub const GAUSSIAN_POWER: i32 = 2;

    /// Distance normalization bounds
    pub const MIN_DISTANCE_NORMALIZATION: f64 = 0.1;
    pub const MAX_DISTANCE_NORMALIZATION: f64 = 1.0;
}

/// Calculate the total path length of a serpentine channel
///
/// # Arguments
/// * `path` - Vector of points defining the serpentine path
///
/// # Returns
/// Total length of the path by summing Euclidean distances between consecutive points
#[must_use]
pub fn calculate_path_length(path: &[Point2D]) -> f64 {
    if path.len() < 2 {
        return 0.0;
    }
    
    path.windows(2)
        .map(|window| {
            let (p1, p2) = (window[0], window[1]);
            let dx = p2.0 - p1.0;
            let dy = p2.1 - p1.1;
            dx.hypot(dy)
        })
        .sum()
}

/// Calculate the minimum distance from a path to the box boundaries
///
/// # Arguments
/// * `path` - Vector of points defining the channel path
/// * `box_dims` - Box dimensions (width, height)
/// * `channel_width` - Width of the channel (for clearance calculation)
///
/// # Returns
/// Minimum distance from any path point to the nearest wall, considering channel width
#[must_use]
pub fn calculate_min_wall_distance(
    path: &[Point2D], 
    box_dims: (f64, f64),
    channel_width: f64
) -> f64 {
    let (box_width, box_height) = box_dims;
    let half_channel_width = channel_width / 2.0;
    
    path.iter()
        .map(|&(x, y)| {
            // Distance to each wall, accounting for channel width
            let dist_to_left = x - half_channel_width;
            let dist_to_right = box_width - x - half_channel_width;
            let dist_to_bottom = y - half_channel_width;
            let dist_to_top = box_height - y - half_channel_width;
            
            // Return minimum distance to any wall
            dist_to_left.min(dist_to_right).min(dist_to_bottom).min(dist_to_top)
        })
        .fold(f64::INFINITY, f64::min)
}

/// Calculate the minimum distance between a path and neighboring channels
///
/// # Arguments
/// * `path` - Vector of points defining the channel path
/// * `neighbor_positions` - Y-coordinates of neighboring channels
/// * `channel_width` - Width of the channel
///
/// # Returns
/// Minimum distance to any neighboring channel, considering channel width
#[must_use]
pub fn calculate_min_neighbor_distance(
    path: &[Point2D],
    neighbor_positions: &[f64],
    channel_width: f64
) -> f64 {
    if neighbor_positions.is_empty() {
        return f64::INFINITY;
    }
    
    let _half_channel_width = channel_width / 2.0;
    
    path.iter()
        .map(|&(_, y)| {
            neighbor_positions.iter()
                .map(|&neighbor_y| (y - neighbor_y).abs() - channel_width)
                .fold(f64::INFINITY, f64::min)
        })
        .fold(f64::INFINITY, f64::min)
}

/// Optimization parameters for serpentine channel generation
///
/// These parameters control the shape and density of serpentine channels
/// during the optimization process.
#[derive(Debug, Clone)]
pub struct OptimizationParams {
    /// Multiplier for channel width to determine wavelength (1.0 to 10.0)
    pub wavelength_factor: f64,
    /// Controls how many waves appear relative to channel length (0.5 to 5.0)
    pub wave_density_factor: f64,
    /// Fraction of available vertical space to fill (0.1 to 0.95)
    pub fill_factor: f64,
}

/// Result of serpentine optimization
///
/// Contains the optimized parameters and performance metrics from
/// the optimization process.
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// The optimized parameters that produced the best result
    pub params: OptimizationParams,
    /// Total length of the optimized serpentine path
    pub path_length: f64,
    /// Minimum distance to any wall boundary
    pub min_wall_distance: f64,
    /// Minimum distance to any neighboring channel
    pub min_neighbor_distance: f64,
    /// Whether the optimization result meets all constraints
    pub is_valid: bool,
    /// Number of optimization iterations performed
    pub iterations: usize,
    /// Total time spent on optimization
    pub optimization_time: std::time::Duration,
}

/// Optimization statistics for monitoring performance
///
/// Provides detailed metrics about the optimization process for
/// performance analysis and debugging.
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Total number of parameter evaluations performed
    pub total_evaluations: usize,
    /// Number of cache hits during optimization
    pub cache_hits: usize,
    /// Number of cache misses during optimization
    pub cache_misses: usize,
    /// Best objective function score achieved
    pub best_score: f64,
    /// Number of iterations until convergence
    pub convergence_iterations: usize,
}

/// Parameter cache for optimization
// Future enhancement: Parameter caching for optimization
// type ParameterCache = HashMap<String, (f64, f64, f64)>; // key -> (length, wall_dist, neighbor_dist)
/// Optimize serpentine parameters to maximize channel length using advanced algorithms
///
/// # Arguments
/// * `p1` - Start point of the channel
/// * `p2` - End point of the channel
/// * `geometry_config` - Geometry configuration
/// * `serpentine_config` - Serpentine configuration
/// * `box_dims` - Box dimensions
/// * `neighbor_info` - Optional neighbor channel positions
///
/// # Returns
/// Optimized parameters that maximize channel length while maintaining constraints
#[must_use]
pub fn optimize_serpentine_parameters(
    p1: Point2D,
    p2: Point2D,
    geometry_config: &GeometryConfig,
    serpentine_config: &SerpentineConfig,
    box_dims: (f64, f64),
    neighbor_info: Option<&[f64]>,
) -> OptimizationResult {
    let start_time = std::time::Instant::now();

    match serpentine_config.optimization_profile {
        OptimizationProfile::Fast => optimize_fast(
            p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info, start_time
        ),
        OptimizationProfile::Balanced => optimize_nelder_mead(
            p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info, start_time
        ),
        OptimizationProfile::Thorough => optimize_thorough(
            p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info, start_time
        ),
    }
}

/// Fast optimization using limited grid search
fn optimize_fast(
    p1: Point2D,
    p2: Point2D,
    geometry_config: &GeometryConfig,
    serpentine_config: &SerpentineConfig,
    box_dims: (f64, f64),
    neighbor_info: Option<&[f64]>,
    start_time: std::time::Instant,
) -> OptimizationResult {
    // Fast optimization with limited parameter exploration
    let min_clearance = geometry_config.wall_clearance;
    let channel_width = geometry_config.channel_width;

    // Get configurable parameter search ranges from constants registry
    let constants = ConstantsRegistry::new();
    let wavelength_factors = constants.get_fast_wavelength_factors();
    let wave_density_factors = constants.get_fast_wave_density_factors();
    let fill_factors = constants.get_fast_fill_factors();
    
    let mut best_result = OptimizationResult {
        params: OptimizationParams {
            wavelength_factor: serpentine_config.wavelength_factor,
            wave_density_factor: serpentine_config.wave_density_factor,
            fill_factor: serpentine_config.fill_factor,
        },
        path_length: 0.0,
        min_wall_distance: 0.0,
        min_neighbor_distance: 0.0,
        is_valid: false,
        iterations: 0,
        optimization_time: std::time::Duration::from_secs(0),
    };

    let mut iterations = 0;
    
    // Grid search over parameter combinations
    for &wavelength_factor in wavelength_factors {
        for &wave_density_factor in wave_density_factors {
            for &fill_factor in fill_factors {
                iterations += 1;

                // Create test configuration without cloning the entire config
                let test_config = SerpentineConfig {
                    wavelength_factor,
                    wave_density_factor,
                    fill_factor,
                    gaussian_width_factor: serpentine_config.gaussian_width_factor,
                    wave_phase_direction: serpentine_config.wave_phase_direction,
                    wave_shape: serpentine_config.wave_shape,
                    optimization_enabled: false, // Disable nested optimization
                    target_fill_ratio: serpentine_config.target_fill_ratio,
                    optimization_profile: serpentine_config.optimization_profile,
                    adaptive_config: serpentine_config.adaptive_config,
                };

                // Generate test path using simplified serpentine generation logic
                let test_path = generate_simplified_serpentine_path(
                    p1, p2, geometry_config, &test_config, box_dims, neighbor_info
                );

                // Calculate metrics
                let path_length = calculate_path_length(&test_path);
                let min_wall_distance = calculate_min_wall_distance(&test_path, box_dims, channel_width);
                let min_neighbor_distance = neighbor_info.map_or(f64::INFINITY, |neighbors| calculate_min_neighbor_distance(&test_path, neighbors, channel_width));

                // Use penalty-based constraint handling for better optimization
                let penalty = calculate_constraint_penalty(min_wall_distance, min_neighbor_distance, min_clearance);
                let objective_score = path_length - penalty;

                // Update best result if this is better
                if objective_score > best_result.path_length {
                    let is_valid = penalty < constants::PENALTY_TOLERANCE;
                    best_result = OptimizationResult {
                        params: OptimizationParams {
                            wavelength_factor,
                            wave_density_factor,
                            fill_factor,
                        },
                        path_length: objective_score,
                        min_wall_distance,
                        min_neighbor_distance,
                        is_valid,
                        iterations,
                        optimization_time: start_time.elapsed(),
                    };
                }
            }
        }
    }
    
    // If no improvement found, return original parameters
    if best_result.path_length <= constants::MIN_PATH_LENGTH_THRESHOLD {
        let original_path = generate_simplified_serpentine_path(
            p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info
        );

        best_result = OptimizationResult {
            params: OptimizationParams {
                wavelength_factor: serpentine_config.wavelength_factor,
                wave_density_factor: serpentine_config.wave_density_factor,
                fill_factor: serpentine_config.fill_factor,
            },
            path_length: calculate_path_length(&original_path),
            min_wall_distance: calculate_min_wall_distance(&original_path, box_dims, channel_width),
            min_neighbor_distance: neighbor_info.map_or(f64::INFINITY, |neighbors| calculate_min_neighbor_distance(&original_path, neighbors, channel_width)),
            is_valid: true, // Assume original parameters are valid
            iterations,
            optimization_time: start_time.elapsed(),
        };
    }

    best_result
}

/// Calculate penalty for constraint violations
#[must_use]
fn calculate_constraint_penalty(wall_distance: f64, neighbor_distance: f64, min_clearance: f64) -> f64 {
    let mut penalty = 0.0;

    // Heavy penalty for wall clearance violations
    if wall_distance < min_clearance {
        penalty += (min_clearance - wall_distance) * constants::CONSTRAINT_PENALTY_MULTIPLIER;
    }

    // Heavy penalty for neighbor clearance violations
    if neighbor_distance < min_clearance {
        penalty += (min_clearance - neighbor_distance) * constants::CONSTRAINT_PENALTY_MULTIPLIER;
    }

    penalty
}

/// Balanced optimization using Nelder-Mead simplex algorithm
fn optimize_nelder_mead(
    p1: Point2D,
    p2: Point2D,
    geometry_config: &GeometryConfig,
    serpentine_config: &SerpentineConfig,
    box_dims: (f64, f64),
    neighbor_info: Option<&[f64]>,
    start_time: std::time::Instant,
) -> OptimizationResult {
    // Start with current parameters as initial guess
    let initial_params = [
        serpentine_config.wavelength_factor,
        serpentine_config.wave_density_factor,
        serpentine_config.fill_factor,
    ];

    // Create initial simplex (triangle in 3D parameter space)
    let mut simplex = [
        initial_params,
        [initial_params[0] * constants::WAVELENGTH_PERTURBATION, initial_params[1], initial_params[2]],
        [initial_params[0], initial_params[1] * constants::WAVE_DENSITY_PERTURBATION, initial_params[2]],
        [initial_params[0], initial_params[1], initial_params[2] * constants::FILL_FACTOR_PERTURBATION],
    ];

    // Evaluate initial simplex
    let mut scores: Vec<f64> = simplex.iter()
        .map(|params| evaluate_objective_function(
            *params, p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info
        ))
        .collect();

    let constants = crate::config_constants::ConstantsRegistry::new();
    let max_iterations = constants.get_max_optimization_iterations();
    let tolerance = constants.get_optimization_tolerance();
    let mut iterations = 0;

    // Nelder-Mead algorithm parameters
    let alpha = constants::REFLECTION_COEFFICIENT;
    let gamma = constants::EXPANSION_COEFFICIENT;
    let rho = constants::CONTRACTION_COEFFICIENT;
    let sigma = constants::SHRINK_COEFFICIENT;

    for _ in 0..max_iterations {
        iterations += 1;

        // Sort simplex by scores (best to worst)
        let mut indices: Vec<usize> = (0..simplex.len()).collect();
        indices.sort_by(|&a, &b| {
            scores[b].partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal) // Handle NaN values gracefully
        });

        let best_idx = indices[0];
        let worst_idx = indices[indices.len() - 1];
        let second_worst_idx = indices[indices.len() - 2];

        // Check for convergence
        let score_range = scores[best_idx] - scores[worst_idx];
        if score_range < tolerance {
            break;
        }

        // Calculate centroid of all points except worst
        let mut centroid = [0.0; 3];
        for &idx in &indices[..indices.len()-1] {
            for i in 0..3 {
                centroid[i] += simplex[idx][i];
            }
        }
        for i in 0..3 {
            centroid[i] /= (simplex.len() - 1) as f64;
        }

        // Reflection
        let mut reflected = [0.0; 3];
        for i in 0..3 {
            reflected[i] = centroid[i] + alpha * (centroid[i] - simplex[worst_idx][i]);
        }
        let reflected_score = evaluate_objective_function(
            reflected, p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info
        );

        if reflected_score > scores[second_worst_idx] && reflected_score <= scores[best_idx] {
            // Accept reflection
            simplex[worst_idx] = reflected;
            scores[worst_idx] = reflected_score;
        } else if reflected_score > scores[best_idx] {
            // Try expansion
            let mut expanded = [0.0; 3];
            for i in 0..3 {
                expanded[i] = centroid[i] + gamma * (reflected[i] - centroid[i]);
            }
            let expanded_score = evaluate_objective_function(
                expanded, p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info
            );

            if expanded_score > reflected_score {
                simplex[worst_idx] = expanded;
                scores[worst_idx] = expanded_score;
            } else {
                simplex[worst_idx] = reflected;
                scores[worst_idx] = reflected_score;
            }
        } else {
            // Try contraction
            let mut contracted = [0.0; 3];
            for i in 0..3 {
                contracted[i] = centroid[i] + rho * (simplex[worst_idx][i] - centroid[i]);
            }
            let contracted_score = evaluate_objective_function(
                contracted, p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info
            );

            if contracted_score > scores[worst_idx] {
                simplex[worst_idx] = contracted;
                scores[worst_idx] = contracted_score;
            } else {
                // Shrink simplex
                for i in 1..simplex.len() {
                    for j in 0..3 {
                        simplex[i][j] = simplex[best_idx][j] + sigma * (simplex[i][j] - simplex[best_idx][j]);
                    }
                    scores[i] = evaluate_objective_function(
                        simplex[i], p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info
                    );
                }
            }
        }
    }

    // Find best result
    let best_idx = scores.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) // Handle NaN values gracefully
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0); // Default to first element if no maximum found

    let best_params = simplex[best_idx];
    let best_config = SerpentineConfig {
        wavelength_factor: best_params[0],
        wave_density_factor: best_params[1],
        fill_factor: best_params[2],
        ..*serpentine_config
    };

    // Generate final path and calculate metrics
    let final_path = generate_simplified_serpentine_path(
        p1, p2, geometry_config, &best_config, box_dims, neighbor_info
    );

    let path_length = calculate_path_length(&final_path);
    let min_wall_distance = calculate_min_wall_distance(&final_path, box_dims, geometry_config.channel_width);
    let min_neighbor_distance = if let Some(neighbors) = neighbor_info {
        calculate_min_neighbor_distance(&final_path, neighbors, geometry_config.channel_width)
    } else {
        f64::INFINITY
    };

    OptimizationResult {
        params: OptimizationParams {
            wavelength_factor: best_params[0],
            wave_density_factor: best_params[1],
            fill_factor: best_params[2],
        },
        path_length,
        min_wall_distance,
        min_neighbor_distance,
        is_valid: min_wall_distance >= geometry_config.wall_clearance && min_neighbor_distance >= geometry_config.wall_clearance,
        iterations,
        optimization_time: start_time.elapsed(),
    }
}

/// Evaluate objective function for optimization (length - penalties)
#[must_use]
fn evaluate_objective_function(
    params: [f64; 3],
    p1: Point2D,
    p2: Point2D,
    geometry_config: &GeometryConfig,
    serpentine_config: &SerpentineConfig,
    box_dims: (f64, f64),
    neighbor_info: Option<&[f64]>,
) -> f64 {
    // Clamp parameters to valid ranges
    let wavelength_factor = params[0].clamp(constants::MIN_WAVELENGTH_FACTOR, constants::MAX_WAVELENGTH_FACTOR);
    let wave_density_factor = params[1].clamp(constants::MIN_WAVE_DENSITY_FACTOR, constants::MAX_WAVE_DENSITY_FACTOR);
    let fill_factor = params[2].clamp(constants::MIN_FILL_FACTOR, constants::MAX_FILL_FACTOR);

    let test_config = SerpentineConfig {
        wavelength_factor,
        wave_density_factor,
        fill_factor,
        ..*serpentine_config
    };

    // Generate test path
    let test_path = generate_simplified_serpentine_path(
        p1, p2, geometry_config, &test_config, box_dims, neighbor_info
    );

    // Calculate metrics
    let path_length = calculate_path_length(&test_path);
    let min_wall_distance = calculate_min_wall_distance(&test_path, box_dims, geometry_config.channel_width);
    let min_neighbor_distance = if let Some(neighbors) = neighbor_info {
        calculate_min_neighbor_distance(&test_path, neighbors, geometry_config.channel_width)
    } else {
        f64::INFINITY
    };

    // Calculate penalty
    let penalty = calculate_constraint_penalty(
        min_wall_distance,
        min_neighbor_distance,
        geometry_config.wall_clearance
    );

    // Return objective score (maximize length, minimize penalty)
    path_length - penalty
}

/// Thorough optimization using multi-start Nelder-Mead with extensive exploration
fn optimize_thorough(
    p1: Point2D,
    p2: Point2D,
    geometry_config: &GeometryConfig,
    serpentine_config: &SerpentineConfig,
    box_dims: (f64, f64),
    neighbor_info: Option<&[f64]>,
    start_time: std::time::Instant,
) -> OptimizationResult {
    let mut best_result = OptimizationResult {
        params: OptimizationParams {
            wavelength_factor: serpentine_config.wavelength_factor,
            wave_density_factor: serpentine_config.wave_density_factor,
            fill_factor: serpentine_config.fill_factor,
        },
        path_length: constants::MIN_PATH_LENGTH_THRESHOLD,
        min_wall_distance: constants::MIN_PATH_LENGTH_THRESHOLD,
        min_neighbor_distance: constants::MIN_PATH_LENGTH_THRESHOLD,
        is_valid: false,
        iterations: 0,
        optimization_time: std::time::Duration::from_secs(0),
    };

    // Multiple starting points for thorough exploration
    let starting_points = vec![
        [serpentine_config.wavelength_factor, serpentine_config.wave_density_factor, serpentine_config.fill_factor],
        [1.0, 1.0, 0.7],
        [2.0, 2.0, 0.8],
        [3.0, 3.0, 0.9],
        [4.0, 1.5, 0.85],
        [1.5, 4.0, 0.75],
    ];

    let mut total_iterations = 0;

    for start_point in starting_points {
        // Create temporary config for this starting point
        let temp_config = SerpentineConfig {
            wavelength_factor: start_point[0],
            wave_density_factor: start_point[1],
            fill_factor: start_point[2],
            ..*serpentine_config
        };

        // Run Nelder-Mead from this starting point
        let result = optimize_nelder_mead(
            p1, p2, geometry_config, &temp_config, box_dims, neighbor_info, start_time
        );

        total_iterations += result.iterations;

        // Keep the best result
        if result.path_length > best_result.path_length {
            best_result = result;
        }
    }

    // Update total iterations
    best_result.iterations = total_iterations;
    best_result.optimization_time = start_time.elapsed();

    best_result
}



/// Simplified serpentine path generation for optimization (fallback)
#[must_use]
fn generate_simplified_serpentine_path(
    p1: Point2D,
    p2: Point2D,
    geometry_config: &GeometryConfig,
    serpentine_config: &SerpentineConfig,
    box_dims: (f64, f64),
    neighbor_info: Option<&[f64]>,
) -> Vec<Point2D> {
    let n_points = geometry_config.generation.optimization_points;
    let mut path = Vec::with_capacity(n_points);
    
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    let channel_length = dx.hypot(dy);
    
    // Calculate amplitude based on available space
    let channel_center_y = f64::midpoint(p1.1, p2.1);
    let box_height = box_dims.1;
    
    // Calculate available space considering neighbors
    let mut available_space_above = box_height - channel_center_y;
    let mut available_space_below = channel_center_y;
    
    if let Some(neighbors) = neighbor_info {
        for &neighbor_y in neighbors {
            if neighbor_y > channel_center_y {
                available_space_above = available_space_above.min(neighbor_y - channel_center_y);
            } else {
                available_space_below = available_space_below.min(channel_center_y - neighbor_y);
            }
        }
    }
    
    let available_space = available_space_above.min(available_space_below);
    let max_amplitude = (available_space - geometry_config.wall_clearance - geometry_config.channel_width) * serpentine_config.fill_factor;
    let amplitude = max_amplitude.max(0.0);
    
    // Generate simplified serpentine path with smooth endpoints
    let base_wavelength = serpentine_config.wavelength_factor * geometry_config.channel_width;
    let base_periods = (channel_length / base_wavelength) * serpentine_config.wave_density_factor;
    // Round to nearest integer number of half-periods to ensure sin(π*n) = 0 at endpoints
    let half_periods = (base_periods * 2.0).round().max(1.0);
    
    for i in 0..n_points {
        let t = i as f64 / (n_points - 1) as f64;

        let base_x = p1.0 + t * dx;
        let base_y = p1.1 + t * dy;

        // Apply smooth endpoint envelope combined with improved Gaussian envelope
        let smooth_envelope = calculate_smooth_endpoint_envelope_for_optimization(t);
        let gaussian_envelope = calculate_improved_envelope_for_optimization(t, channel_length, dx, dy, serpentine_config);
        let envelope = smooth_envelope * gaussian_envelope;

        let wave_phase = std::f64::consts::PI * half_periods * t;
        // Calculate wave amplitude based on wave shape
        let wave_value = match serpentine_config.wave_shape {
            crate::config::WaveShape::Sine => wave_phase.sin(),
            crate::config::WaveShape::Square => {
                let sine_value = wave_phase.sin();
                let sharpness = constants::SQUARE_WAVE_SHARPNESS;
                (sharpness * sine_value).tanh()
            }
        };
        let wave_amplitude = amplitude * envelope * wave_value;
        
        let perp_x = -dy / channel_length;
        let perp_y = dx / channel_length;
        
        let x = base_x + wave_amplitude * perp_x;
        let y = base_y + wave_amplitude * perp_y;

        // Ensure exact endpoint matching for first and last points to maintain precision
        // The smooth envelope should make wave_amplitude ≈ 0 at endpoints, but we ensure exactness
        if i == 0 {
            path.push(p1);
        } else if i == n_points - 1 {
            path.push(p2);
        } else {
            path.push((x, y));
        }
    }
    
    path
}

/// Calculate smooth endpoint envelope for optimization
///
/// Uses smoothstep function for C¹ continuity at endpoints
#[must_use]
fn calculate_smooth_endpoint_envelope_for_optimization(t: f64) -> f64 {
    // Smoothstep function: t²(3-2t)
    t * t * (constants::SMOOTHSTEP_COEFFICIENT_1 - constants::SMOOTHSTEP_COEFFICIENT_2 * t)
}

/// Calculate improved Gaussian envelope for optimization (helper function)
///
/// This mirrors the logic from SerpentineChannelStrategy but is available
/// for use in the optimization module.
#[must_use]
fn calculate_improved_envelope_for_optimization(
    t: f64,
    channel_length: f64,
    dx: f64,
    dy: f64,
    serpentine_config: &SerpentineConfig
) -> f64 {
    // Calculate the actual distance between nodes
    let node_distance = (dx * dx + dy * dy).sqrt();

    // Determine if this is primarily a horizontal channel (middle section logic)
    let is_horizontal = dx.abs() > dy.abs();
    let horizontal_ratio = dx.abs() / node_distance;

    // For horizontal channels (middle sections), we want less aggressive tapering
    let middle_section_factor = if is_horizontal && horizontal_ratio > serpentine_config.adaptive_config.horizontal_ratio_threshold {
        serpentine_config.adaptive_config.middle_section_amplitude_factor +
        (1.0 - serpentine_config.adaptive_config.middle_section_amplitude_factor) * horizontal_ratio
    } else {
        1.0
    };

    // Distance-based normalization
    let distance_normalization = if serpentine_config.adaptive_config.enable_distance_based_scaling {
        (node_distance / serpentine_config.adaptive_config.node_distance_normalization)
            .min(constants::MAX_DISTANCE_NORMALIZATION)
            .max(constants::MIN_DISTANCE_NORMALIZATION)
    } else {
        constants::MAX_DISTANCE_NORMALIZATION // No distance-based scaling when disabled
    };

    // Calculate effective sigma
    let base_sigma = channel_length / serpentine_config.gaussian_width_factor;
    let effective_sigma = base_sigma * distance_normalization * middle_section_factor;

    // Center the envelope
    let center = constants::GAUSSIAN_CENTER;

    // Calculate Gaussian envelope
    let gaussian = (constants::GAUSSIAN_EXPONENT_COEFFICIENT * ((t - center) / (effective_sigma / channel_length)).powi(constants::GAUSSIAN_POWER)).exp();

    // For middle sections, add a plateau in the center
    if is_horizontal && horizontal_ratio > serpentine_config.adaptive_config.horizontal_ratio_threshold {
        let plateau_width = serpentine_config.adaptive_config.plateau_width_factor;
        let plateau_start = 0.5 - plateau_width / 2.0;
        let plateau_end = 0.5 + plateau_width / 2.0;

        if t >= plateau_start && t <= plateau_end {
            let plateau_factor = 1.0 - ((t - 0.5).abs() / (plateau_width / 2.0));
            gaussian.max(serpentine_config.adaptive_config.plateau_amplitude_factor +
                (1.0 - serpentine_config.adaptive_config.plateau_amplitude_factor) * plateau_factor)
        } else {
            gaussian
        }
    } else {
        gaussian
    }
}
