# Scheme - 2D Microfluidic Schematic Design Library

A focused Rust library for designing 2D microfluidic schematics with support for bifurcation and trifurcation patterns, channel layout algorithms, and schematic visualization.

## Features

- **Bifurcation Patterns**: Generate single to five-level bifurcation channel layouts
- **Trifurcation Patterns**: Create single to five-level trifurcation designs  
- **Mixed Patterns**: Combine bifurcation and trifurcation in complex layouts
- **2D Visualization**: Export schematics as PNG images using plotters
- **Configurable Geometry**: Customize channel dimensions and wall clearances
- **Channel Types**: Support for straight and serpentine (S-shaped) channels

## Channel Types

Scheme supports different channel geometries as part of its DAG architecture with precise endpoint alignment:

- **Straight Channels**: Traditional linear connections between nodes
- **Serpentine Channels**: S-shaped channels with configurable amplitude, wavelength, and periods

### Serpentine Channel Features

- **Precise Endpoint Alignment**: Serpentine channels start and end exactly at node positions
- **Improved Gaussian Envelope**: Advanced envelope system with distance-based normalization and middle section detection
- **Distance-Based Normalization**: Shorter channels get more aggressive tapering to prevent node intersection
- **Middle Section Detection**: Horizontal channels (no directional change) maintain more amplitude in the center
- **Plateau Effect**: Middle sections have a flat region for full amplitude where appropriate
- **Directional Change Handling**: Nodes with direction changes get full Gaussian tapering for smooth transitions
- **Linear Near Nodes**: Channels remain linear near connection points, becoming serpentine in the middle
- **Smooth Channel Separation**: Allows channels to separate before becoming serpentine
- **Dynamic Wavelength Adaptation**: Wavelength automatically adapts to channel length for optimal appearance
- **Length Optimization**: Automatically optimize parameters to maximize channel length while maintaining wall clearance
- **Configurable Parameters**: Control amplitude factor, wavelength factor, periods, and Gaussian width

Channel types can be configured using the `ChannelTypeConfig` enum:

```rust
use scheme::config::{ChannelTypeConfig, SerpentineConfig};

// All straight channels
let config = ChannelTypeConfig::AllStraight;

// All serpentine channels with custom parameters
let config = ChannelTypeConfig::AllSerpentine(SerpentineConfig {
    fill_factor: 0.8,           // Fraction of available vertical space to fill (0.0 to 1.0)
    wavelength_factor: 4.0,     // Multiplier for channel width to determine wavelength
    gaussian_width_factor: 6.0, // Controls width of Gaussian envelope (sigma = length / gaussian_width_factor)
    wave_density_factor: 1.5,   // Controls how many waves appear relative to channel length (higher = more waves)
});

// Mixed channels based on position (default)
let config = ChannelTypeConfig::MixedByPosition {
    middle_zone_fraction: 0.4,  // Fraction of box width for serpentine zone
    serpentine_config: SerpentineConfig::default(),
};

// Custom channel type function with length-based selection
let config = ChannelTypeConfig::Custom(|from, to, box_dims| {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let length = (dx * dx + dy * dy).sqrt();
    
    if length > 8.0 {
        ChannelType::Serpentine {
            amplitude: length * 0.15,   // Scale amplitude with length
            wavelength: length * 0.25,  // Scale wavelength with length
            periods: 4.0,
            gaussian_width_factor: 8.0, // Narrower Gaussian for more linear behavior
        }
    } else {
        ChannelType::Straight
    }
});
```

## Serpentine Length Optimization

The library includes an optimization system that automatically adjusts serpentine channel parameters to maximize channel length while maintaining proper wall clearance and multi-channel compatibility.

### Enabling Optimization

```rust
use scheme::config::{SerpentineConfig, presets};

// Create optimized configuration
let optimized_config = SerpentineConfig::new_with_optimization(
    0.8,  // fill_factor
    3.0,  // wavelength_factor
    6.0,  // gaussian_width_factor
    2.0,  // wave_density_factor
    0.95, // target_fill_ratio (95% of maximum possible length)
)?;

// Or use the preset
let optimized_config = presets::optimized_serpentine();
```

### Optimization Features

- **Automatic Parameter Tuning**: Optimizes wavelength_factor, wave_density_factor, and fill_factor
- **Multiple Optimization Profiles**: Fast, Balanced, and Thorough optimization modes
- **Advanced Algorithms**: Uses Nelder-Mead simplex optimization for intelligent parameter search
- **Constraint Satisfaction**: Maintains wall clearance and neighbor spacing requirements
- **Multi-Channel Compatibility**: Works with complex split patterns and multiple channels
- **Bilateral Symmetry Preservation**: Maintains perfect mirror symmetry during optimization
- **Configurable Target**: Set target fill ratio (80-99% of theoretical maximum length)

### Optimization Profiles

- **Fast Profile**: Limited parameter exploration (5-10x slower than standard)
- **Balanced Profile**: Moderate exploration using Nelder-Mead algorithm (20-50x slower)
- **Thorough Profile**: Extensive multi-start optimization (100-500x slower)

## Extensible Metadata System

The library features a comprehensive metadata system that allows you to attach arbitrary tracking data to channels and nodes without breaking existing functionality.

### Key Features

- **Type-Safe Storage**: Metadata is stored and retrieved using Rust's type system
- **Zero-Cost Abstraction**: No performance impact when metadata is not used
- **Backward Compatibility**: All existing code continues to work unchanged
- **Extensible Design**: Easy to add new metadata types for any domain
- **Builder Patterns**: Convenient APIs for creating channels and nodes with metadata

### Built-in Metadata Types

- **FlowMetadata**: Flow rates, pressure drops, Reynolds numbers, velocities
- **ThermalMetadata**: Temperature, heat transfer coefficients, thermal conductivity
- **ManufacturingMetadata**: Tolerances, surface roughness, manufacturing methods
- **OptimizationMetadata**: Optimization history, improvements, iteration counts
- **PerformanceMetadata**: Generation times, memory usage, performance metrics

### Performance Considerations

Optimization adds computational overhead (typically 500-1000x slower than standard generation) but provides significant length improvements in many cases. For production use, consider:

- Using optimization during design phase and caching optimal parameters
- Disabling optimization for real-time applications
- Using preset configurations that balance performance and length

## Quick Start

Add Scheme to your `Cargo.toml`:

```toml
[dependencies]
scheme = "0.1.0"
```

Basic usage:

```rust
use scheme::{
    geometry::{generator::create_geometry, SplitType},
    config::{GeometryConfig, ChannelTypeConfig},
    visualizations::schematic::plot_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure geometry parameters
    let config = GeometryConfig::default();
    
    // Create a bifurcation pattern with straight channels
    let system = create_geometry(
        (200.0, 100.0),  // box dimensions (width, height)
        &[SplitType::Bifurcation],  // split pattern
        &config,
        &ChannelTypeConfig::AllStraight,  // channel type configuration
    );
    
    // Generate visualization
    plot_geometry(&system, "schematic.png")?;
    
    Ok(())
}
```

## Examples

The library includes comprehensive examples using the modern API:

### Bifurcation Patterns
- `single_split` - Basic bifurcation
- `double_split` - Two-level bifurcation
- `triple_split` - Three-level bifurcation
- `four_split` - Four-level bifurcation
- `five_split` - Five-level bifurcation

### Trifurcation Patterns
- `single_trifurcation` - Basic trifurcation
- `double_trifurcation` - Two-level trifurcation
- `triple_trifurcation` - Three-level trifurcation
- `four_trifurcation` - Four-level trifurcation
- `five_trifurcation` - Five-level trifurcation

### Mixed Patterns
- `bifurcation_trifurcation` - Bifurcation followed by trifurcation
- `trifurcation_bifurcation` - Trifurcation followed by bifurcation
- `bifurcation_bifurcation_trifurcation` - Complex three-stage pattern
- `bifurcation_trifurcation_bifurcation` - Alternating pattern
- `trifurcation_bifurcation_trifurcation` - Alternating pattern
- `trifurcation_trifurcation_bifurcation` - Complex three-stage pattern

### Channel Type Examples
- `serpentine_demo` - Demonstrates serpentine channels in a bifurcation pattern
- `mixed_channel_types` - Shows mixed straight and serpentine channels
- `channel_type_demo` - Comprehensive demonstration of all channel type configurations
- `dynamic_serpentine_demo` - Advanced demonstration of precise endpoint alignment and custom configurations
- `gaussian_tone_burst_demo` - Demonstrates Gaussian envelope effects with different width factors
- `improved_gaussian_demo` - Shows the new distance-based normalization and middle section detection
- `unified_generator_demo` - Demonstrates the unified generator API with optional metadata support

### Optimization Examples
- `basic_optimization` - Simple demonstration of serpentine length optimization
- `length_comparison` - Comprehensive comparison of standard vs optimized channel lengths
- `profile_comparison` - Comparison of Fast, Balanced, and Thorough optimization profiles

### Metadata Examples
- `basic_metadata_usage` - Introduction to the extensible metadata system
- `custom_metadata_types` - Creating and using custom metadata types for specific domains

## Performance Benchmarking

The library includes comprehensive benchmarking infrastructure to validate performance characteristics and detect regressions.

### Benchmark Suites

- **Geometry Benchmarks**: Core geometry generation performance across different patterns and complexities
- **Visualization Benchmarks**: PNG and SVG rendering performance for various system complexities
- **Optimization Benchmarks**: Serpentine optimization performance across different profiles and parameters
- **Metadata Benchmarks**: Extensible metadata system performance and overhead analysis

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench geometry_benchmarks
cargo bench --bench optimization_benchmarks
cargo bench --bench metadata_benchmarks
cargo bench --bench visualization_benchmarks

# Run with specific filter
cargo bench -- optimization_profiles
```

### Performance Characteristics

Based on comprehensive benchmarking, the library demonstrates the following performance characteristics:

- **Geometry Generation**:
  - Simple patterns (1-2 levels): < 1ms
  - Complex patterns (3-5 levels): < 10ms
  - Linear scaling with pattern complexity
  - Memory usage: < 1MB for typical patterns

- **Optimization Performance**:
  - Fast profile: 5-10x slower than standard generation
  - Balanced profile: 20-50x slower (recommended for most use cases)
  - Thorough profile: 100-500x slower (for maximum length optimization)
  - Memory overhead: < 5MB during optimization

- **Visualization Rendering**:
  - PNG generation: < 100ms for typical schematics
  - SVG generation: < 200ms for typical schematics
  - Consistent performance across complexity levels
  - Memory usage: < 10MB during rendering

- **Metadata System**:
  - < 1% overhead when unused
  - < 5% overhead when actively used
  - Type-safe with zero runtime cost for unused metadata

Run any example with:

```bash
cargo run --example single_split
```

## API Reference

### Core Types

- `ChannelSystem` - Represents a 2D channel layout
- `Channel` - Individual channel with type and path information
- `ChannelType` - Enum for straight or serpentine channels
- `SplitType` - Enum for bifurcation and trifurcation patterns
- `GeometryConfig` - Configuration for channel dimensions
- `ChannelTypeConfig` - Configuration for channel type selection
- `Point2D` - 2D coordinate type

### Main Functions

- `create_geometry()` - Generate channel layouts with explicit channel type configuration
- `plot_geometry()` - Export schematics as PNG images

## Architecture

Scheme focuses exclusively on 2D schematic design, having removed all 3D mesh generation, CSG operations, and CFD simulation capabilities for simplicity and performance.

## License

MIT License - see LICENSE file for details.
