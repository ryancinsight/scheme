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
    wave_shape: WaveShape::Sine, // Wave shape: Sine (smooth curves) or Square (angular transitions)
    ..SerpentineConfig::default()
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

### Wave Shape Control

Serpentine channels now support different wave shapes for varied design aesthetics:

- **Sine Wave (Default)**: Smooth, natural flowing curves that provide gentle transitions
- **Square Wave**: Angular transitions with smooth corners for more geometric designs

```rust
use scheme::config::{SerpentineConfig, WaveShape};

// Smooth sine wave serpentines (default)
let sine_config = SerpentineConfig::default().with_sine_wave();

// Angular square wave serpentines
let square_config = SerpentineConfig::default().with_square_wave();

// Or specify directly
let custom_config = SerpentineConfig {
    wave_shape: WaveShape::Square,
    ..SerpentineConfig::default()
};
```

Both wave shapes maintain:

- Perfect bilateral mirror symmetry
- Smooth rendering with 200+ points per channel (configurable)
- Proper envelope functions for smooth endpoint transitions
- Full compatibility with optimization and adaptive features

## 🛡️ **Arc Channel Collision Prevention**

The library includes advanced collision prevention for arc channels to eliminate overlaps in high-curvature scenarios:

### **Proximity Detection**

- **Real-time overlap detection** during arc path generation
- **Configurable minimum separation distance** between channels
- **Neighbor proximity analysis** for adaptive behavior
- **Density-based estimation** when neighbor information is unavailable

### **Adaptive Curvature Control**

- **Automatic curvature reduction** when overlaps are detected
- **Intelligent path adjustment** that maintains visual appeal
- **Configurable reduction limits** to preserve design intent
- **Progressive reduction algorithms** for optimal balance

### **Safety Configuration**

- **Collision prevention toggle** - can be enabled/disabled per configuration
- **Maximum curvature reduction limits** to prevent over-correction
- **Separation distance validation** with configurable thresholds
- **Safety presets** for common scenarios (dense layouts, high curvature, etc.)

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

## Colored Visualization

Scheme now features **colored channel type differentiation** for easy visual identification:

- **🖤 Straight Channels**: Black (Straight, SmoothStraight)
- **🔵 Curved Channels**: Blue (Serpentine, Arc)
- **🔴 Tapered Channels**: Red (Frustum)

### Custom Color Configuration

You can customize colors for different channel types:

```rust
use scheme::visualizations::{RenderConfig, ChannelTypeStyles, LineStyle, Color};

let custom_styles = ChannelTypeStyles {
    straight_style: LineStyle::solid(Color::rgb(100, 100, 100), 1.0), // Gray
    curved_style: LineStyle::solid(Color::rgb(0, 150, 0), 2.0),       // Green
    tapered_style: LineStyle::solid(Color::rgb(255, 100, 0), 3.0),    // Orange
};

let mut config = RenderConfig::default();
config.channel_type_styles = custom_styles;

scheme::visualizations::schematic::plot_geometry_with_config(
    &system, "custom_colors.svg", &config
)?;
```

## Examples

The library includes comprehensive examples organized by functionality:

### 🚀 **Comprehensive Demos** (Recommended Starting Point)

- `comprehensive_split_patterns` - All split patterns (bifurcation, trifurcation, mixed)
- `comprehensive_serpentine_demo` - Complete serpentine channel showcase with wave shapes, phase control, and optimization
- `comprehensive_arc_demo` - Complete arc channel demonstration with curvature control and smart selection
- `comprehensive_arc_collision_prevention_demo` - Arc collision prevention and proximity detection showcase
- `frustum_channel_demo` - Complete frustum (tapered) channel demonstration with different taper profiles
- `colored_channel_demo` - Colored visualization demonstration showing different channel types in distinct colors

### 🔧 **Specialized Examples**

- `configuration_validation_demo` - Configuration validation and error handling examples
- `svg_demo` - SVG output format demonstration
- `unified_generator_demo` - Unified generator API with metadata support

### 🚀 **Optimization & Performance**

- `optimization/basic_optimization` - Basic optimization demonstration
- `optimization/length_comparison` - Length optimization comparison
- `optimization/profile_comparison` - Optimization profile comparison

### 📊 **Metadata & Advanced Features**

- `metadata/basic_metadata_usage` - Basic metadata system usage
- `metadata/custom_metadata_types` - Custom metadata type implementation

### 📁 **Organized Output Structure**

All examples generate outputs in organized directories:

- `outputs/split_patterns/` - Split pattern visualizations
- `outputs/serpentine/` - Serpentine channel outputs with wave shapes, phase directions, configurations, and optimization
- `outputs/arcs/` - Arc channel visualizations with curvature, smoothness, directions, and collision prevention
- `outputs/mixed/` - Mixed channel configurations and smart selection
- `outputs/optimization/` - Optimization demonstrations and comparisons
- `outputs/metadata/` - Metadata system examples
- `outputs/svg/` - SVG format outputs
- `outputs/unified/` - Unified generator examples
- `outputs/configuration_validation/` - Configuration validation examples

### 🧪 **Metadata System**

- `basic_metadata_usage` - Introduction to the extensible metadata system
- `custom_metadata_types` - Creating custom metadata types for specific domains

### 🎯 **Quick Start Recommendation**

For new users, start with the **Comprehensive Demos** section:

1. Run `cargo run --example comprehensive_split_patterns`
2. Run `cargo run --example comprehensive_serpentine_demo`
3. Run `cargo run --example comprehensive_arc_demo`

These will generate organized outputs in the `outputs/` directory showcasing all major features.

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
