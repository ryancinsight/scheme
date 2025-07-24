//! Optimization utilities for serpentine channel generation
//!
//! This module provides utilities for optimizing serpentine channel parameters
//! to maximize channel length while maintaining proper wall clearance and
//! multi-channel compatibility.

use crate::geometry::types::Point2D;
use crate::config::{GeometryConfig, SerpentineConfig, OptimizationProfile};
// use std::collections::HashMap; // For future parameter caching

/// Calculate the total path length of a serpentine channel
///
/// # Arguments
/// * `path` - Vector of points defining the serpentine path
///
/// # Returns
/// Total length of the path by summing Euclidean distances between consecutive points
pub fn calculate_path_length(path: &[Point2D]) -> f64 {
    if path.len() < 2 {
        return 0.0;
    }
    
    path.windows(2)
        .map(|window| {
            let (p1, p2) = (window[0], window[1]);
            let dx = p2.0 - p1.0;
            let dy = p2.1 - p1.1;
            (dx * dx + dy * dy).sqrt()
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

    // Reduced parameter search ranges for speed
    let wavelength_factors = [1.0, 2.0, 3.0, 4.0];
    let wave_density_factors = [1.0, 2.0, 3.0];
    let fill_factors = [0.7, 0.8, 0.9];
    
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
    for &wavelength_factor in &wavelength_factors {
        for &wave_density_factor in &wave_density_factors {
            for &fill_factor in &fill_factors {
                iterations += 1;

                // Create test configuration
                let test_config = SerpentineConfig {
                    wavelength_factor,
                    wave_density_factor,
                    fill_factor,
                    ..serpentine_config.clone()
                };

                // Generate test path using simplified serpentine generation logic
                let test_path = generate_simplified_serpentine_path(
                    p1, p2, geometry_config, &test_config, box_dims, neighbor_info
                );

                // Calculate metrics
                let path_length = calculate_path_length(&test_path);
                let min_wall_distance = calculate_min_wall_distance(&test_path, box_dims, channel_width);
                let min_neighbor_distance = if let Some(neighbors) = neighbor_info {
                    calculate_min_neighbor_distance(&test_path, neighbors, channel_width)
                } else {
                    f64::INFINITY
                };

                // Use penalty-based constraint handling for better optimization
                let penalty = calculate_constraint_penalty(min_wall_distance, min_neighbor_distance, min_clearance);
                let objective_score = path_length - penalty;

                // Update best result if this is better
                if objective_score > best_result.path_length {
                    let is_valid = penalty < 1.0; // Small penalty tolerance
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
    if best_result.path_length <= 0.0 {
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
            min_neighbor_distance: if let Some(neighbors) = neighbor_info {
                calculate_min_neighbor_distance(&original_path, neighbors, channel_width)
            } else {
                f64::INFINITY
            },
            is_valid: true, // Assume original parameters are valid
            iterations,
            optimization_time: start_time.elapsed(),
        };
    }

    best_result
}

/// Calculate penalty for constraint violations
fn calculate_constraint_penalty(wall_distance: f64, neighbor_distance: f64, min_clearance: f64) -> f64 {
    let mut penalty = 0.0;

    // Heavy penalty for wall clearance violations
    if wall_distance < min_clearance {
        penalty += (min_clearance - wall_distance) * 1000.0;
    }

    // Heavy penalty for neighbor clearance violations
    if neighbor_distance < min_clearance {
        penalty += (min_clearance - neighbor_distance) * 1000.0;
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
    let mut simplex = vec![
        initial_params,
        [initial_params[0] * 1.1, initial_params[1], initial_params[2]],
        [initial_params[0], initial_params[1] * 1.1, initial_params[2]],
        [initial_params[0], initial_params[1], initial_params[2] * 1.05],
    ];

    // Evaluate initial simplex
    let mut scores: Vec<f64> = simplex.iter()
        .map(|params| evaluate_objective_function(
            *params, p1, p2, geometry_config, serpentine_config, box_dims, neighbor_info
        ))
        .collect();

    let max_iterations = 50; // Reasonable limit for balanced optimization
    let tolerance = 1e-6;
    let mut iterations = 0;

    // Nelder-Mead algorithm parameters
    let alpha = 1.0;  // Reflection coefficient
    let gamma = 2.0;  // Expansion coefficient
    let rho = 0.5;    // Contraction coefficient
    let sigma = 0.5;  // Shrink coefficient

    for _ in 0..max_iterations {
        iterations += 1;

        // Sort simplex by scores (best to worst)
        let mut indices: Vec<usize> = (0..simplex.len()).collect();
        indices.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap());

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
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap();

    let best_params = simplex[best_idx];
    let best_config = SerpentineConfig {
        wavelength_factor: best_params[0],
        wave_density_factor: best_params[1],
        fill_factor: best_params[2],
        ..serpentine_config.clone()
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
    let wavelength_factor = params[0].clamp(0.5, 5.0);
    let wave_density_factor = params[1].clamp(0.5, 5.0);
    let fill_factor = params[2].clamp(0.1, 0.95);

    let test_config = SerpentineConfig {
        wavelength_factor,
        wave_density_factor,
        fill_factor,
        ..serpentine_config.clone()
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
        path_length: 0.0,
        min_wall_distance: 0.0,
        min_neighbor_distance: 0.0,
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
            ..serpentine_config.clone()
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
fn generate_simplified_serpentine_path(
    p1: Point2D,
    p2: Point2D,
    geometry_config: &GeometryConfig,
    serpentine_config: &SerpentineConfig,
    box_dims: (f64, f64),
    neighbor_info: Option<&[f64]>,
) -> Vec<Point2D> {
    let n_points = 50; // Fewer points for faster optimization
    let mut path = Vec::with_capacity(n_points);
    
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    let channel_length = (dx * dx + dy * dy).sqrt();
    
    // Calculate amplitude based on available space
    let channel_center_y = (p1.1 + p2.1) / 2.0;
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
    
    // Generate simplified serpentine path
    let base_wavelength = serpentine_config.wavelength_factor * geometry_config.channel_width;
    let periods = (channel_length / base_wavelength) * serpentine_config.wave_density_factor;
    
    for i in 0..n_points {
        let t = i as f64 / (n_points - 1) as f64;

        let base_x = p1.0 + t * dx;
        let base_y = p1.1 + t * dy;

        // Apply improved envelope logic for optimization path as well
        let envelope = calculate_improved_envelope_for_optimization(t, channel_length, dx, dy, serpentine_config);

        let wave_phase = 2.0 * std::f64::consts::PI * periods * t;
        let wave_amplitude = amplitude * envelope * wave_phase.sin();
        
        let perp_x = -dy / channel_length;
        let perp_y = dx / channel_length;
        
        let x = base_x + wave_amplitude * perp_x;
        let y = base_y + wave_amplitude * perp_y;
        
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

/// Calculate improved Gaussian envelope for optimization (helper function)
///
/// This mirrors the logic from SerpentineChannelStrategy but is available
/// for use in the optimization module.
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
    let middle_section_factor = if is_horizontal && horizontal_ratio > 0.8 {
        0.3 + 0.7 * horizontal_ratio
    } else {
        1.0
    };

    // Distance-based normalization
    let distance_normalization = (node_distance / 10.0).min(1.0).max(0.1);

    // Calculate effective sigma
    let base_sigma = channel_length / serpentine_config.gaussian_width_factor;
    let effective_sigma = base_sigma * distance_normalization * middle_section_factor;

    // Center the envelope
    let center = 0.5;

    // Calculate Gaussian envelope
    let gaussian = (-0.5 * ((t - center) / (effective_sigma / channel_length)).powi(2)).exp();

    // For middle sections, add a plateau in the center
    if is_horizontal && horizontal_ratio > 0.8 {
        let plateau_width = 0.4;
        let plateau_start = 0.5 - plateau_width / 2.0;
        let plateau_end = 0.5 + plateau_width / 2.0;

        if t >= plateau_start && t <= plateau_end {
            let plateau_factor = 1.0 - ((t - 0.5).abs() / (plateau_width / 2.0));
            gaussian.max(0.8 + 0.2 * plateau_factor)
        } else {
            gaussian
        }
    } else {
        gaussian
    }
}
